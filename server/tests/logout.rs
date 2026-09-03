mod shared;

mod logout {
    use common::grpc_codegen::EmptyReply;

    use crate::shared::{
        constants::INVALID_EXPIRED_TOKEN,
        utils::{
            client_authenticate_user, client_get_player, client_logout_user, GetPlayerError,
            TestAuthError,
        },
    };

    /// Nominal case
    #[tokio::test]
    async fn logout_01_valid_session() {
        let response = client_authenticate_user("logout1", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        // Logout user
        let result = client_logout_user("logout1", token.clone()).await;
        assert!(result.is_ok(), "{result:?}");
        assert!(result.unwrap().into_inner() == EmptyReply {});

        // Ensure token has been revoked
        match client_get_player("logout1", token).await {
            Ok(_) => panic!("Unexpected success, token should be revoked"),
            Err(GetPlayerError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(GetPlayerError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };
    }

    /// Connected user logout with nonexistent/wrong token
    #[tokio::test]
    async fn logout_02_bad_token() {
        let response = client_authenticate_user("logout2", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        match client_logout_user("logout2", "bad_token".to_string()).await {
            Ok(_) => panic!("Unexpected success, user should not be logout"),
            Err(TestAuthError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(TestAuthError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };

        // Ensure token is still valid
        assert!(client_get_player("logout2", token).await.is_ok());
    }

    /// Logout with a valid token that not pertain to an existing login
    #[tokio::test]
    async fn logout_03_bad_login() {
        let response = client_authenticate_user("logout3", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        match client_logout_user("logout1", token.clone()).await {
            Ok(_) => panic!("Unexpected success, user should not be logout"),
            Err(TestAuthError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(TestAuthError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };

        // Ensure token is still valid
        assert!(client_get_player("logout3", token).await.is_ok());
    }

    /// Valid login with empty token
    #[tokio::test]
    async fn logout_04_empty_token() {
        let response = client_authenticate_user("logout4", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        match client_logout_user("logout4", "".to_string()).await {
            Ok(_) => panic!("Unexpected success, user should not be logout"),
            Err(TestAuthError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(TestAuthError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };

        // Ensure token is still valid
        assert!(client_get_player("logout4", token).await.is_ok());
    }

    /// Valid token with empty login
    #[tokio::test]
    async fn logout_05_empty_login() {
        let response = client_authenticate_user("logout5", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        match client_logout_user("".to_string(), token.clone()).await {
            Ok(_) => panic!("Unexpected success, user should not be logout"),
            Err(TestAuthError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(TestAuthError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };

        // Ensure token is still valid
        assert!(client_get_player("logout5", token).await.is_ok());
    }

    /// Both empty login and token
    #[tokio::test]
    async fn logout_06_empty_credentials() {
        match client_logout_user("".to_string(), "".to_string()).await {
            Ok(_) => panic!("Unexpected success, user should not be logout"),
            Err(TestAuthError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(TestAuthError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };
    }

    #[tokio::test]
    async fn logout_07_multiple_disconnection() {
        let response = client_authenticate_user("logout7", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        // Logout user
        let result = client_logout_user("logout7", token.clone()).await;
        assert!(result.is_ok(), "{result:?}");
        assert!(result.unwrap().into_inner() == EmptyReply {});

        // Second logout for this user should fail
        match client_logout_user("logout7", token).await {
            Ok(_) => panic!("Unexpected success, user should not be logout"),
            Err(TestAuthError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(TestAuthError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };
    }
}
