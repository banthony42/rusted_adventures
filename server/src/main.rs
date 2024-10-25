use common::authenticator::Authenticator;

use authentication::authenticate_server::{Authenticate, AuthenticateServer};
use authentication::{AuthReply, AuthRequest, EmptyReply, LogoutRequest};
use common::database::db::Database;
use diesel::prelude::*;
use tonic::transport::Server;
use tonic::Response;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:2121".parse()?;
    let rpg_authenticate = RPGAuthenticate::default();

    Server::builder()
        .add_service(AuthenticateServer::new(rpg_authenticate))
        .serve(addr)
        .await?;

    Ok(())
}
