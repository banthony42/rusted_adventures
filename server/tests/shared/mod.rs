#[allow(dead_code)]
pub mod constants {
    pub const TEST_SERVER_ENDPOINT: &str = "http://localhost:2121";
    pub const INVALID_LOGIN_PASSWORD: &str = "Invalid login or password";
    pub const INVALID_EXPIRED_TOKEN: &str = "Invalid or expired token";
    pub const TOO_MANY_AUTH_ATTEMPT: &str = "Too many attempts, retry later";
    pub const MAX_LOGIN_LENGTH: usize = 32;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
}

pub mod utils {
    use common::grpc_codegen::{
        rpg_authenticate_client::RpgAuthenticateClient, rpg_entity_client::RpgEntityClient,
        AuthReply, AuthRequest, EmptyReply, EmptyRequest, LogoutRequest, PlayerData,
    };
    use tonic::{metadata::MetadataValue, transport::Endpoint, Request, Status};

    use crate::shared::constants::TEST_SERVER_ENDPOINT;

    #[allow(dead_code)]
    #[derive(Debug)]
    pub enum TestAuthError {
        Connection(tonic::transport::Error),
        Status(tonic::Status),
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    pub enum GetPlayerError {
        Connection(tonic::transport::Error),
        Status(tonic::Status),
    }

    pub async fn client_authenticate_user(
        login: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<tonic::Response<AuthReply>, TestAuthError> {
        let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
            .await
            .map_err(TestAuthError::Connection)?;

        let request = tonic::Request::new(AuthRequest {
            login: login.into(),
            password: password.into(),
        });

        client
            .authenticate_user(request)
            .await
            .map_err(TestAuthError::Status)
    }

    #[allow(dead_code)]
    pub async fn client_logout_user(
        login: impl Into<String>,
        token: impl Into<String>,
    ) -> Result<tonic::Response<EmptyReply>, TestAuthError> {
        let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
            .await
            .map_err(TestAuthError::Connection)?;

        client
            .logout(tonic::Request::new(LogoutRequest {
                login: login.into(),
                token: token.into(),
            }))
            .await
            .map_err(TestAuthError::Status)
    }

    #[cfg(test)]
    pub fn auth_interceptor(
        login: String,
        token: String,
    ) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let login_md: MetadataValue<_> = login
                .parse()
                .map_err(|err| Status::invalid_argument(format!("Login: {}", err)))?;

            let token_md: MetadataValue<_> = token
                .parse()
                .map_err(|err| Status::invalid_argument(format!("Token: {}", err)))?;

            req.metadata_mut().insert("login", login_md);
            req.metadata_mut().insert("authorization", token_md);
            Ok(req)
        }
    }

    #[allow(dead_code)]
    pub async fn client_get_player(
        login: impl Into<String>,
        token: impl Into<String>,
    ) -> Result<tonic::Response<PlayerData>, GetPlayerError> {
        let endpoint = Endpoint::from_static(TEST_SERVER_ENDPOINT)
            .connect()
            .await
            .map_err(GetPlayerError::Connection)?;

        let mut client = RpgEntityClient::with_interceptor(
            endpoint,
            auth_interceptor(login.into(), token.into()),
        );

        let request = tonic::Request::new(EmptyRequest {});

        client
            .get_player(request)
            .await
            .map_err(GetPlayerError::Status)
    }
}
