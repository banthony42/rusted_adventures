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

mod rpc_extensions;

pub mod authenticate;
pub mod chat;
pub mod entities;
