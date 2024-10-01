use common::authenticator::Authenticator;

use authentication::authenticate_server::{Authenticate, AuthenticateServer};
use authentication::{AuthReply, AuthRequest};
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
        println!("[RPGAuthenticate] : with: {:?}", auth_req);

        if !Authenticator::new(auth_req.login.clone()).authenticate(&auth_req.password) {
            println!("Error: Invalid login or password.");
            return Err(tonic::Status::invalid_argument("Invalid login or password"));
        }
        // Generate dummy session token TODO: replace by JWST
        let new_token = format!("{}-cafebab", auth_req.login);

        // Store session token in DB for this user
        use common::database::schema::accounts::dsl::*;
        let connection = &mut Database::new().establish_connection();

        match diesel::update(accounts)
            .filter(login.eq(auth_req.login))
            .set(session_token.eq(new_token.clone()))
            .execute(connection)
        {
            Ok(_) => Ok(Response::new(AuthReply { token: new_token })),
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
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
