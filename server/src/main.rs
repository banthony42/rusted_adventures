use common::authenticator::Authenticator;

use authentication::authenticate_server::{Authenticate, AuthenticateServer};
use authentication::{AuthReply, AuthRequest, EmptyReply, LogoutRequest};
use common::database::db::Database;
use diesel::prelude::*;
use tokio::runtime::{Builder, Runtime};
use tonic::transport::Server;
use tonic::Response;
use world::engine::WorldEngine;

mod world;

pub mod authentication {
    include!("../../common/GRPC_codegen/authentication.rs");
}

#[derive(Debug, Default)]
struct RPGAuthenticate {}

#[tonic::async_trait]
impl Authenticate for RPGAuthenticate {
    async fn authenticate_user(
        &self,
        request: tonic::Request<AuthRequest>,
    ) -> std::result::Result<tonic::Response<AuthReply>, tonic::Status> {
        let auth_req: AuthRequest = request.into_inner();
        println!("[Server]: [AuthenticateUser] : with: {:?}", auth_req);

        if !Authenticator::new(auth_req.login.clone()).authenticate(&auth_req.password) {
            println!("[Server]: [AuthenticateUser] : Error: Invalid login or password.");
            return Err(tonic::Status::invalid_argument("Invalid login or password"));
        }
        // Generate dummy session token TODO: replace by JWST
        let new_token = format!("{}-cafebab", auth_req.login);

        // Store session token in DB for this user - TODO: move this in Authenticator
        use common::database::schema::accounts::dsl::*;
        let connection = &mut Database::new().establish_connection();

        match diesel::update(accounts)
            .filter(login.eq(auth_req.login))
            .set(session_token.eq(Some(new_token.clone())))
            .execute(connection)
        {
            Ok(_) => {
                println!(
                    "[Server]: [AuthenticateUser] : Success: token: {}",
                    new_token
                );
                Ok(Response::new(AuthReply { token: new_token }))
            }
            Err(e) => {
                println!("[Server]: [AuthenticateUser] : Error: {}", e.to_string());
                Err(tonic::Status::internal(e.to_string()))
            }
        }
    }

    async fn logout(
        &self,
        request: tonic::Request<LogoutRequest>,
    ) -> std::result::Result<tonic::Response<EmptyReply>, tonic::Status> {
        let logout_request: LogoutRequest = request.into_inner();
        println!("[Server]: [LogoutUser] : with: {:?}", logout_request);

        let mut authenticator = Authenticator::new(logout_request.login.clone());
        if !authenticator.logout(Some(logout_request.token.clone())) {
            println!("[Server]: [LogoutUser] : Error: internal error");
            return Err(tonic::Status::internal("Logout failed"));
        }

        println!("[Server]: [LogoutUser] : Success");
        Ok(Response::new(EmptyReply {}))
    }
}

fn run_world_engine_on_another_thread() -> Runtime {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("RPG World Engine")
        .enable_all()
        .build()
        .unwrap();

    let world_engine = WorldEngine::new();
    runtime.spawn(async move {
        world_engine.run().await;
    });

    return runtime;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:2121".parse()?;
    let rpg_authenticate = RPGAuthenticate::default();

    // For setup simplicity the WorldEngine coexist within the Grpc Server.
    // I insist on the term 'WorldENGINE' because it's not a server since it not handle connections or requests.
    // We spawn a new thread where the WorldEngine will run (TODO: maybe we don't need the WorldEngine to be async for now)
    // Goal is to share a Mutex on the Database access to ensure DB read/write atomicity
    // Then the WorldEngine should update the world over the time (read/write)
    // and the Grpc server will just answer to client request reading the DB when needed.
    // Therefore with this approach it means that the Game States are stored in the postgresql DB.
    // I know it's not the best solution, maybe a redis should be better for performance
    // But again, the goal of the project is to learn rust, and i want to avoid
    // heavy setup configuration / prerequisite.
    // In addition i already know redis, where i totally discover and learn SQL like DB.
    let _rt = run_world_engine_on_another_thread();

    Server::builder()
        .add_service(AuthenticateServer::new(rpg_authenticate))
        .serve(addr)
        .await?;

    Ok(())
}
