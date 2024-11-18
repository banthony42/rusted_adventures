use common::authenticator::Authenticator;

use grpc_codegen::rpg_authenticate_server::RpgAuthenticate;
use grpc_codegen::{AuthReply, AuthRequest, EmptyReply, LogoutRequest};
use tonic::Response;

pub mod grpc_codegen {
    include!("../../../common/GRPC_codegen/rpg.package.rs");
}

#[derive(Debug, Default)]
pub struct RpgAuthenticateService {}

#[tonic::async_trait]
impl RpgAuthenticate for RpgAuthenticateService {
    async fn authenticate_user(
        &self,
        request: tonic::Request<AuthRequest>,
    ) -> std::result::Result<tonic::Response<AuthReply>, tonic::Status> {
        let auth_req: AuthRequest = request.into_inner();
        println!("[Server]: [AuthenticateUser] : with: {:?}", auth_req);

        let mut user = Authenticator::new(auth_req.login.clone());
        if !user.authenticate(&auth_req.password) {
            println!("[Server]: [AuthenticateUser] : Error: Invalid login or password.");
            return Err(tonic::Status::invalid_argument("Invalid login or password"));
        }
        // Generate dummy session token TODO: replace by JWST
        let new_token = format!("{}-cafebab", auth_req.login);

        // Store session token in DB for this user
        match user.set_token(&new_token) {
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
