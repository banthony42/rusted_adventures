use common::authenticator::Authenticator;
use common::database::model::character::PgClasses;
use tracing::instrument;

use crate::services::constants::*;
use common::character::{CharacterHandler, CharacterHandlerError};
use common::grpc_codegen::rpg_authenticate_server::RpgAuthenticate;
use common::grpc_codegen::{AuthReply, AuthRequest, EmptyReply, LogoutRequest};
use tonic::Response;

#[derive(Debug, Default)]
pub struct RpgAuthenticateService {}

#[tonic::async_trait]
impl RpgAuthenticate for RpgAuthenticateService {
    #[instrument(level = "debug")]
    async fn authenticate_user(
        &self,
        request: tonic::Request<AuthRequest>,
    ) -> Result<tonic::Response<AuthReply>, tonic::Status> {
        let auth_req: AuthRequest = request.into_inner();
        tracing::info!("{AUTHENTICATE_USER_WITH} login: {}", auth_req.login);

        if auth_req.login.is_empty() || auth_req.login.len() > MAX_LOGIN_LENGTH {
            return Err(tonic::Status::invalid_argument(INVALID_LOGIN_PASSWORD));
        }

        if auth_req.password.is_empty() || auth_req.password.len() > MAX_PASSWORD_LENGTH {
            return Err(tonic::Status::invalid_argument(INVALID_LOGIN_PASSWORD));
        }

        let mut user = Authenticator::new(&auth_req.login);

        user.authenticate(&auth_req.password)?;

        // Session ID Token best practices from OWASP:
        // https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
        // I tried to list, applicable web app best practices that could be implemented in our game server:
        //
        // _ : session inactivity timeout, to logout inactive users (server-side)
        // _ : TLS to avoid disclose the token (related to gRPC server or reverse proxy)
        // _ : Client can implement a maximum retry counter, with a timer when the maximum is reached
        // _ : Client can logout the user at window close event
        // _ : Bind session id token with user properties (ip address, OS, etc ...)
        // Advanced measure:
        // _ : password / token bruteforce detection with single IP address

        // Session ID Token best practices from OWASP:
        // https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
        // _ : Simultaneous login ideally should prompt user to choose what should be done:
        //     (Keep old session logout new session, Keep new and logout old)
        //
        // Here i choose unique session per user,
        // Therefore revoke any existing sessions for this user before creating another one
        user.revoke_session(None)?;

        let session_id_token = user.create_session()?;

        tracing::info!(
            "{AUTHENTICATE_USER_SUCCESS} {} session_id: {}...",
            user.login(),
            &session_id_token[..7]
        );

        // Temporary automatic characters creation, waiting for character creation client/server
        // Automate characters creation : for now only one character is allowed per account
        let success = Response::new(AuthReply {
            token: session_id_token,
        });
        match CharacterHandler::new(&auth_req.login) {
            Ok(_) => {
                tracing::info!(
                    "{CHARACTER_CREATION} A character already exist for {}",
                    auth_req.login
                );
                Ok(success)
            }
            Err(CharacterHandlerError::NoCharacterForAccount) => {
                if let Err(e) = CharacterHandler::create(
                    &auth_req.login,
                    &auth_req.login,
                    rand::random::<PgClasses>(),
                ) {
                    tracing::error!("{CHARACTER_CREATION}failed with: {e:?}");
                    Err(tonic::Status::internal(e.to_string()))
                } else {
                    tracing::info!("{CHARACTER_CREATION}succeed.");
                    Ok(success)
                }
            }
            Err(e) => {
                tracing::error!(
                    "{AUTHENTICATE_USER_ERROR}while retrieving Character for {}: {e}",
                    auth_req.login
                );
                Err(tonic::Status::internal(e.to_string()))
            }
        }
    }

    #[instrument(level = "debug")]
    async fn logout(
        &self,
        request: tonic::Request<LogoutRequest>,
    ) -> Result<tonic::Response<EmptyReply>, tonic::Status> {
        let req: LogoutRequest = request.into_inner();
        tracing::info!("{LOGOUT_USER_WITH} login: {}", req.login);

        if req.token.is_empty() || req.login.is_empty() {
            return Err(tonic::Status::invalid_argument(INVALID_EXPIRED_TOKEN));
        }

        Authenticator::new(&req.login).revoke_session(Some(req.token))?;

        tracing::info!("{LOGOUT_USER_SUCCESS}");
        Ok(Response::new(EmptyReply {}))
    }
}
