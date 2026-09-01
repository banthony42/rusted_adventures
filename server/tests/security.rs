mod shared;

mod security {
    use std::time::Duration;

    use common::grpc_codegen::{rpg_authenticate_client::RpgAuthenticateClient, AuthRequest};

    use crate::shared::{
        constants::{
            INVALID_EXPIRED_TOKEN, INVALID_LOGIN_PASSWORD, TEST_SERVER_ENDPOINT,
            TOO_MANY_AUTH_ATTEMPT,
        },
        utils::{client_authenticate_user, client_get_player, GetPlayerError},
    };

    /// Ensure valid token from account A can't retrieve protected informations of account B
    #[tokio::test]
    async fn security_01_valid_token_theft() {
        let token_a = client_authenticate_user("security1A", "42")
            .await
            .expect("Unexpected error")
            .into_inner()
            .token;

        // Ensure valid token don't leak data from other account B
        match client_get_player("security1B", &token_a).await {
            Ok(_) => panic!("Unexpected success, token A must not leak information of account B"),
            Err(GetPlayerError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(GetPlayerError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };

        let _ = client_authenticate_user("security1B", "42")
            .await
            .expect("Unexpected error");

        // Ensure valid token don't leak data from other account B
        // even if this account B is connected
        match client_get_player("security1B", token_a).await {
            Ok(_) => panic!("Unexpected success, token A must not leak information of account B"),
            Err(GetPlayerError::Connection(c)) => panic!("Unexpected error: {c}"),
            Err(GetPlayerError::Status(status)) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_EXPIRED_TOKEN);
            }
        };
    }

    /// Check rate limitation against password guessing by bruteforce
    #[tokio::test]
    async fn security_02_password_bruteforce_must_lockout_the_account() {
        let login = "security2".to_string();
        let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
            .await
            .expect("Unexpected error");

        // Reach the rate limitation
        for attempt in 1..5 {
            let request = tonic::Request::new(AuthRequest {
                login: login.clone(),
                password: format!("test-security-02-password-bruteforce-attempt{attempt}"),
            });

            match client.authenticate_user(request).await {
                Ok(_) => panic!("Unexpected success"),
                Err(status) => {
                    assert_eq!(status.code(), tonic::Code::Unauthenticated);
                    assert_eq!(status.message(), INVALID_LOGIN_PASSWORD);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Next request should directly fail
        let request = tonic::Request::new(AuthRequest {
            login: login.clone(),
            password: "test-security-02-password-bruteforce-last-attempt".into(),
        });

        match client.authenticate_user(request).await {
            Ok(_) => panic!("Unexpected success"),
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::ResourceExhausted);
                assert_eq!(status.message(), TOO_MANY_AUTH_ATTEMPT);
            }
        }
    }

    /// Trigger login throttling mecanism by password bruteforce to lock the account
    /// and ensure the account is available after the lockout time.
    #[tokio::test]
    async fn security_03_recover_after_account_lockout_expired() {
        let login = "security3".to_string();
        let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
            .await
            .expect("Unexpected error");

        // Reach the rate limitation
        for attempt in 1..5 {
            let request = tonic::Request::new(AuthRequest {
                login: login.clone(),
                password: format!(
                    "test-security-03-recover-after-account-lockout-expired-attempt{attempt}"
                ),
            });

            let _ = client.authenticate_user(request).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let request = tonic::Request::new(AuthRequest {
            login: login.clone(),
            password: "test-security-03-recover-after-account-lockout-expired-locked-out".into(),
        });

        // Ensure account is Lockout
        match client.authenticate_user(request).await {
            Ok(_) => panic!("Unexpected success"),
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::ResourceExhausted);
                assert_eq!(status.message(), TOO_MANY_AUTH_ATTEMPT);
            }
        }

        // Wait account lockout is over
        // First lockout can be customized with ACCOUNT_LOCK_TIME_1=1 env variable
        tokio::time::sleep(Duration::from_secs(2)).await;

        let request = tonic::Request::new(AuthRequest {
            login,
            password: "42".into(),
        });

        // Ensure account is back available
        client
            .authenticate_user(request)
            .await
            .expect("unexpected error should success since account is not locked anymore");
    }

    /// Ensure successful authentication reset the failure counter that lead to account lockout
    #[tokio::test]
    async fn security_04_recover_after_auth_succeed() {
        let login = "security4".to_string();
        let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
            .await
            .expect("Unexpected error");

        // Reach 4 failure on 5 authorized
        for attempt in 1..=4 {
            let request = tonic::Request::new(AuthRequest {
                login: login.clone(),
                password: format!("test-security-04-recover-after-auth-suceed-attempt{attempt}"),
            });

            let _ = client.authenticate_user(request).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // auth succeed (should reset failure counter in server)
        let request = tonic::Request::new(AuthRequest {
            login: login.clone(),
            password: "42".into(),
        });
        client
            .authenticate_user(request)
            .await
            .expect("Unexpected error");

        // auth fail 2 times more should both return unauthenticated and not resource exhausted
        for attempt in 1..=2 {
            let request = tonic::Request::new(AuthRequest {
                login: login.clone(),
                password: format!("test-security-04-recover-after-auth-suceed-attempt{attempt}"),
            });

            match client.authenticate_user(request).await {
                Ok(_) => panic!("Unexpected success"),
                Err(status) => {
                    assert_eq!(status.code(), tonic::Code::Unauthenticated);
                    assert_eq!(status.message(), INVALID_LOGIN_PASSWORD);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Ensure time window reset failures
    #[tokio::test]
    async fn security_05_throttling_time_window_reset_failures() {
        let login = "security5".to_string();
        let mut client = RpgAuthenticateClient::connect(TEST_SERVER_ENDPOINT)
            .await
            .expect("Unexpected error");

        // Only one failure within time window 1:
        let request = tonic::Request::new(AuthRequest {
            login: login.clone(),
            password: "test-security-05-time-window-reset-failures-attempt1-window1".into(),
        });

        match client.authenticate_user(request).await {
            Ok(_) => panic!("Unexpected success"),
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated);
                assert_eq!(status.message(), INVALID_LOGIN_PASSWORD);
            }
        }
        // Need server to be run with THROTTLING_TIME_WINDOW=2
        tokio::time::sleep(Duration::from_millis(2050)).await;

        // Trigger 4 failures within time window 2 should not trigger account lock (5 failures needed within a window)
        for attempt in 1..=4 {
            let request = tonic::Request::new(AuthRequest {
                login: login.clone(),
                password: format!(
                    "test-security-05-time-window-reset-failures-attempt{attempt}-window2"
                ),
            });

            match client.authenticate_user(request).await {
                Ok(_) => panic!("Unexpected success"),
                Err(status) => {
                    assert_eq!(status.code(), tonic::Code::Unauthenticated);
                    assert_eq!(status.message(), INVALID_LOGIN_PASSWORD);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let request = tonic::Request::new(AuthRequest {
            login,
            password: "test-security-05-time-window-reset-failures-locked-out".into(),
        });

        // Ensure account is Lockout
        match client.authenticate_user(request).await {
            Ok(_) => panic!("Unexpected success"),
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::ResourceExhausted);
                assert_eq!(status.message(), TOO_MANY_AUTH_ATTEMPT);
            }
        }
    }
}
