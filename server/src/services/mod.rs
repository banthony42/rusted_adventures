mod constants {
    pub const AUTHENTICATE_USER_WITH: &str = "Server: AuthenticateUser: with: ";
    pub const AUTHENTICATE_USER_SUCCESS: &str = "Server: AuthenticateUser: Success: ";
    pub const AUTHENTICATE_USER_ERROR: &str = "Server: AuthenticateUser: Error: ";

    pub const LOGOUT_USER_WITH: &str = "Server: LogoutUser: with: ";
    pub const LOGOUT_USER_ERROR: &str = "Server: LogoutUser: Error: ";
    pub const LOGOUT_USER_SUCCESS: &str = "Server: LogoutUser: Success: ";

    pub const INVALID_LOGIN_PASSWORD: &str = "Invalid login or password";

    pub const CHARACTER_CREATION: &str = "Server: AuthenticateUser: CreateCharacter: ";
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
