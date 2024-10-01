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
        let mut reply = AuthReply::default();

        println!("[RPGAuthenticate] : with: {:?}", auth_req);

        if !Authenticator::new(auth_req.login.clone()).authenticate(&auth_req.password) {
            println!();
            reply.success = false;
            reply.message = String::from("Invalid login or password.");
            return Ok(Response::new(reply));
        }

        // Generate dummy session token TODO: replace by JWST
        let new_token = format!("{}-cafebab", auth_req.login);

        // Store session token in DB for this user
        use common::database::schema::accounts::dsl::*;
        let connection = &mut Database::new().establish_connection();
        diesel::update(accounts)
            .filter(login.eq(auth_req.login))
            .set(session_token.eq(new_token.clone()))
            .execute(connection)
            .expect("Error while updating account.");

        // Reply with success and with session token
        reply.success = true;
        reply.message = String::from("User connected.");
        reply.token = new_token;
        Ok(Response::new(reply))
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
