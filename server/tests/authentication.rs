mod shared;

mod authentication {
    use crate::shared::{
        constants::*,
        utils::{client_authenticate_user, AuthError},
    };
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

    /// Ensure token are unique and different over sessions
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
    }
}
