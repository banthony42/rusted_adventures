use common::grpc_codegen::{
    rpg_authenticate_client::RpgAuthenticateClient, rpg_entity_client::RpgEntityClient, AuthReply,
    AuthRequest, EmptyRequest, PlayerData,
};
use tonic::{metadata::MetadataValue, transport::Endpoint, Request, Status};

const TEST_SERVER_ENDPOINT: &str = "http://localhost:2121";
const INVALID_LOGIN_PASSWORD: &str = "Invalid login or password";
const INVALID_EXPIRED_TOKEN: &str = "Invalid or expired token";
const MAX_LOGIN_LENGTH: usize = 32;
const MAX_PASSWORD_LENGTH: usize = 128;

#[derive(Debug)]
enum AuthError {
    Connection(tonic::transport::Error),
    Status(tonic::Status),
}

#[derive(Debug)]
enum GetPlayerError {
    Connection(tonic::transport::Error),
    Status(tonic::Status),
}

async fn client_authenticate_user(
    login: impl Into<String>,
    password: impl Into<String>,
) -> Result<tonic::Response<AuthReply>, AuthError> {
    let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
        .await
        .map_err(AuthError::Connection)?;

    let request = tonic::Request::new(AuthRequest {
        login: login.into(),
        password: password.into(),
    });

    client
        .authenticate_user(request)
        .await
        .map_err(AuthError::Status)
}

fn auth_interceptor(
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

async fn client_get_player(
    login: impl Into<String>,
    token: impl Into<String>,
) -> Result<tonic::Response<PlayerData>, GetPlayerError> {
    let endpoint = Endpoint::from_static(TEST_SERVER_ENDPOINT)
        .connect()
        .await
        .map_err(GetPlayerError::Connection)?;

    let mut client =
        RpgEntityClient::with_interceptor(endpoint, auth_interceptor(login.into(), token.into()));

    let request = tonic::Request::new(EmptyRequest {});

    client
        .get_player(request)
        .await
        .map_err(GetPlayerError::Status)
}

/// Nominal case
#[tokio::test]
async fn auth_01_valid_credentials() {
    let response = client_authenticate_user("arthur", "42")
        .await
        .expect("Unexpected error");

    assert!(!response.into_inner().token.is_empty());
}

/// Should not disclose any account information
#[tokio::test]
async fn auth_02_bad_password() {
    let response = match client_authenticate_user("arthur", "bad password").await {
        // server must be accessible for the test to be perform
        Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
        Err(AuthError::Status(status)) => status,
        Ok(_) => panic!("Unexpected success"),
    };

    assert_eq!(response.code(), tonic::Code::Unauthenticated);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

/// Should not disclose any account information
#[tokio::test]
async fn auth_03_unknow_login() {
    let response = match client_authenticate_user("unknow login", "password").await {
        // server must be accessible for the test to be perform
        Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
        Err(AuthError::Status(status)) => status,
        Ok(_) => panic!("Unexpected success"),
    };

    assert_eq!(response.code(), tonic::Code::Unauthenticated);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

#[tokio::test]
async fn auth_04_empty_login() {
    let response = match client_authenticate_user("", "password").await {
        // server must be accessible for the test to be perform
        Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
        Err(AuthError::Status(status)) => status,
        Ok(_) => panic!("Unexpected success"),
    };

    assert_eq!(response.code(), tonic::Code::InvalidArgument);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

#[tokio::test]
async fn auth_05_empty_password() {
    let response = match client_authenticate_user("arthur", "").await {
        // server must be accessible for the test to be perform
        Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
        Err(AuthError::Status(status)) => status,
        Ok(_) => panic!("Unexpected success"),
    };

    assert_eq!(response.code(), tonic::Code::InvalidArgument);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

#[tokio::test]
async fn auth_06_empty_credentials() {
    let response = match client_authenticate_user("", "").await {
        // server must be accessible for the test to be perform
        Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
        Err(AuthError::Status(status)) => status,
        Ok(_) => panic!("Unexpected success"),
    };

    assert_eq!(response.code(), tonic::Code::InvalidArgument);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

/// Protection against credentials limit abuse
#[tokio::test]
async fn auth_07_exceed_login_limit_size() {
    let response =
        match client_authenticate_user("a".repeat(MAX_LOGIN_LENGTH + 1), "password").await {
            // server must be accessible for the test to be perform
            Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
            Err(AuthError::Status(status)) => status,
            Ok(_) => panic!("Unexpected success"),
        };

    assert_eq!(response.code(), tonic::Code::InvalidArgument);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

/// Protection against credentials limit abuse
#[tokio::test]
async fn auth_08_exceed_password_limit_size() {
    let response =
        match client_authenticate_user("arthur", "a".repeat(MAX_PASSWORD_LENGTH + 1)).await {
            // server must be accessible for the test to be perform
            Err(AuthError::Connection(error)) => panic!("Unexpected error: {error}"),
            Err(AuthError::Status(status)) => status,
            Ok(_) => panic!("Unexpected success"),
        };

    assert_eq!(response.code(), tonic::Code::InvalidArgument);
    assert_eq!(response.message(), INVALID_LOGIN_PASSWORD);
}

/// Check UTF-8 handling
#[tokio::test]
async fn auth_09_utf8_credentials() {
    let response = client_authenticate_user("bastien😀", "21🤘")
        .await
        .expect("Unexpected error");

    assert!(!response.into_inner().token.is_empty());
}

/// Check unique session policy
/// Keep the new session and revoke the old one
#[tokio::test]
async fn auth_10_multiple_authentication() {
    let response_1 = client_authenticate_user("arthur", "42")
        .await
        .expect("Unexpected error");

    let response_2 = client_authenticate_user("arthur", "42")
        .await
        .expect("Unexpected error");

    let token_1 = response_1.into_inner().token;
    let token_2 = response_2.into_inner().token;

    // We must have token for both sessions
    assert!(!token_1.is_empty());
    assert!(!token_2.is_empty());

    // Tokens must be unique accross sessions
    assert_ne!(token_1, token_2);

    // Ensure token_1 has been revoked
    match client_get_player("arthur", token_1).await {
        Ok(_) => panic!("Unexpected success, token should be revoked"),
        Err(GetPlayerError::Connection(c)) => panic!("Unexpected error: {c}"),
        Err(GetPlayerError::Status(status)) => {
            assert_eq!(status.code(), tonic::Code::Unauthenticated);
            assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
        }
    };

    // Ensure token_2 is valid
    assert!(client_get_player("arthur", token_2).await.is_ok())
}
