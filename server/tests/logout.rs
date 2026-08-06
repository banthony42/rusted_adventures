/// Nominal case
#[tokio::test]
async fn logout_001_valid_credentials() {
    todo!()
}

/// Random nonexistent token
#[tokio::test]
async fn logout_002_unauthenticated_session() {
    todo!()
}

/// Valid token with unknow login
#[tokio::test]
async fn logout_003_bad_login() {
    todo!()
}

/// Valid login with empty token
#[tokio::test]
async fn logout_004_empty_token() {
    todo!()
}

/// Valid token with empty login
#[tokio::test]
async fn logout_005_empty_login() {
    todo!()
}

/// Both empty login and token
#[tokio::test]
async fn logout_006_empty_credentials() {
    todo!()
}

#[tokio::test]
async fn logout_007_multiple_disconnection() {
    todo!()
}

/// Assert multiple session are isolated
/// logout from session 1 does not revoke token from session 2
#[tokio::test]
async fn logout_008_isolated_session_revokation() {
    todo!()
}
