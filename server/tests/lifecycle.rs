/// Classic connection / disconnection flow (Nominal case)
#[tokio::test]
async fn lifecycle_001_nominal_auth_and_logout() {
    todo!()
}

/// Use revoked token to access authenticated services
#[tokio::test]
async fn lifecycle_002_revoked_token_usage() {
    // Current authenticated services are entity / chat
    // maybe add dummy rpc for authenticate service that require valid token
    // such as :
    // rpc GetCurrentUser (AuthenticatedRequest) returns (CurrentUserReply);
    // message CurrentUserReply {
    //    string login = 1;
    // }
    //
    // with interceptor like chat/entity
    todo!()
}

/// Assert expired token lock protected services
#[tokio::test]
async fn lifecycle_003_session_expiration() {
    todo!()
}

/// Check protection against multiple sessions abuse
#[tokio::test]
async fn lifecycle_004_session_limit() {
    todo!()
}
