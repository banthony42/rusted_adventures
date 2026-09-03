mod shared;

mod lifecycle {
    use std::time::Duration;

    use common::grpc_codegen::EmptyReply;

    use crate::shared::{
        constants::INVALID_EXPIRED_TOKEN,
        utils::{client_authenticate_user, client_get_player, client_logout_user, GetPlayerError},
    };

    /// Classic connection / disconnection flow (Nominal case)
    #[tokio::test]
    async fn lifecycle_01_nominal_auth_and_logout() {
        // Connect a user
        let response = client_authenticate_user("lifecycle1", "42")
            .await
            .expect("Unexpected error");

        // Ensure token is valid
        let token = response.into_inner().token;
        let result = client_get_player("lifecycle1", &token).await;
        assert!(result.is_ok(), "{result:?}");

        // Logout user
        let result = client_logout_user("lifecycle1", token).await;
        assert!(result.is_ok(), "{result:?}");
        assert!(result.unwrap().into_inner() == EmptyReply {});
    }

    /// Ensure unique sessions policy
    /// New authentication revoke the last generated token
    #[tokio::test]
    async fn lifecycle_02_revoked_token_usage() {
        let response_1 = client_authenticate_user("lifecycle2", "42")
            .await
            .expect("Unexpected error");

        // Cause the first token to be revoke (unique session policy)
        let response_2 = client_authenticate_user("lifecycle2", "42")
            .await
            .expect("Unexpected error");

        let token_1 = response_1.into_inner().token;
        let token_2 = response_2.into_inner().token;

        // Ensure token_1 has been revoked
        match client_get_player("lifecycle2", token_1).await {
            Ok(_) => panic!("Unexpected success, token should be revoked"),
            Err(GetPlayerError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(GetPlayerError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };

        // Ensure token_2 is valid
        let result = client_get_player("lifecycle2", token_2).await;
        assert!(result.is_ok(), "{result:?}")
    }

    /// Assert expired token lock protected services
    #[tokio::test]
    async fn lifecycle_03_session_expiration() {
        // Connect a user to get a token
        let response = client_authenticate_user("lifecycle3", "42")
            .await
            .expect("Unexpected error");

        let token = response.into_inner().token;

        // Ensure token is valid
        assert!(client_get_player("lifecycle3", token.clone()).await.is_ok());

        // Wait the token to expire
        // Token expiration can be customized with SESSION_TOKEN_EXPIRATION=2 env variable for server run
        tokio::time::sleep(Duration::from_millis(2050)).await;

        // Ensure token has expire and is now invalid
        match client_get_player("lifecycle3", token).await {
            Ok(_) => panic!("Unexpected success, token must expire"),
            Err(GetPlayerError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(GetPlayerError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };
    }
}
