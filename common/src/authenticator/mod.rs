use argon2::{
    password_hash, Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    Version,
};

use diesel::PgConnection;
use password_hash::rand_core::OsRng;
use password_hash::SaltString;

use crate::database::db::Database;
use crate::database::model::account::account::AccountError;
use crate::database::model::account::Account;

pub struct Authenticator {
    login: String,
    connection: PgConnection,
}

impl Authenticator {
    pub fn new(login: &str) -> Self {
        Authenticator {
            login: login.to_string(),
            connection: Database::new().establish_connection(),
        }
    }

    fn create_argon2<'a>() -> Argon2<'a> {
        Argon2::new(
            Algorithm::Argon2id,
            Version::default(),
            Params::new(19 * 1024, 2, 1, Some(Params::DEFAULT_OUTPUT_LEN))
                .expect("Error while creating Argon2 context"),
        )
    }

    pub fn hash_password(password: String) -> String {
        Self::create_argon2()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .expect("Error while hashing password")
            .to_string()
    }

    pub fn authenticate(&mut self, password: &String) -> bool {
        // Get the user account in DB
        let Ok(account_to_auth) = Account::read(&mut self.connection, &self.login) else {
            return false;
        };

        // Import user hashed password and verify it
        let Ok(parsed_hash) = PasswordHash::new(&account_to_auth.password) else {
            println!("Error while importing user password hash from database.");
            return false;
        };

        // TODO: we must return false (invalid password) only on password_hash::errors::Error::Password
        if let Err(err) = Self::create_argon2().verify_password(password.as_bytes(), &parsed_hash) {
            println!("Error while while verifying user password: {:?}", err);
            return false;
        }
        true
    }

    pub fn get_token(&mut self) -> Option<String> {
        match Account::read(&mut self.connection, &self.login) {
            Ok(account) => account.session_token,
            Err(_) => {
                println!(
                    "Server: Authenticator: Fail to get token for: {}",
                    self.login
                );
                None
            }
        }
    }

    pub fn set_token(&mut self, token: &String) -> Result<(), diesel::result::Error> {
        Account::set_token(&mut self.connection, &self.login, token)?;
        Ok(())
    }

    pub fn is_connected(&mut self) -> Result<(), AccountError> {
        Account::is_connected(&mut self.connection, &self.login)?;
        Ok(())
    }

    pub fn logout(&mut self, token: Option<String>) -> Result<(), AccountError> {
        let _token = match token {
            Some(token) => Some(token),
            None => self.get_token(),
        };

        Account::logout(&mut self.connection, &self.login, _token)
            .map_err(|_| AccountError::LogoutError)
    }
}
