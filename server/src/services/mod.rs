mod constants {
    pub const AUTHENTICATE_USER_WITH: &str = "AuthenticateUser: with: ";
    pub const AUTHENTICATE_USER_SUCCESS: &str = "AuthenticateUser: Success: ";
    pub const AUTHENTICATE_USER_ERROR: &str = "AuthenticateUser: Error: ";

    pub const LOGOUT_USER_WITH: &str = "LogoutUser: with: ";
    pub const LOGOUT_USER_ERROR: &str = "LogoutUser: Error: ";
    pub const LOGOUT_USER_SUCCESS: &str = "LogoutUser: Success: ";

    pub const INVALID_LOGIN_PASSWORD: &str = "Invalid login or password";

    pub const CHARACTER_CREATION: &str = "AuthenticateUser: CreateCharacter: ";
}

mod utils {
    use tonic::{metadata::MetadataMap, Status};

    pub fn login_from_metadata(metadata: MetadataMap) -> Result<String, Status> {
        Ok(metadata
            .get("login")
            .ok_or_else(|| Status::unauthenticated(""))?
            .to_str()
            .map_err(|err| Status::unauthenticated(err.to_string()))?
            .to_string())
    }
}

pub mod authenticate;
pub mod chat;
pub mod entities;
