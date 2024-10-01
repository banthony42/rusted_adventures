use diesel::prelude::*;

use argon2::*;
use password_hash::rand_core::OsRng;
use password_hash::SaltString;

use crate::database::db::Database;
use crate::database::models::Account;
use crate::database::schema::accounts;

pub struct Authenticator<'a> {
    login: String,
    argon2: Argon2<'a>,
}

fn create_argon2_context() -> Params {
    match Params::new(19 * 1024, 2, 1, Some(Params::DEFAULT_OUTPUT_LEN)) {
        Ok(p) => p,
        Err(e) => {
            println!("Error while hashing password: {:?}", e);
            std::process::exit(1);
        }
    }
}

impl Default for Authenticator<'_> {
    fn default() -> Self {
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::default(),
            create_argon2_context(),
        );

        Self {
            login: Default::default(),
            argon2: argon2,
        }
    }
}

impl Authenticator<'_> {
    pub fn new(login: String) -> Self {
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::default(),
            create_argon2_context(),
        );

        Authenticator {
            argon2: argon2,
            login: login,
        }
    }

    pub fn hash_password(&self, password: String) -> String {
        let salt = SaltString::generate(&mut OsRng);

        let hash_pasword = match self.argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(e) => {
                println!("Error while hashing password: {:?}", e);
                std::process::exit(1)
            }
        };
        return hash_pasword;
    }

    pub fn authenticate(&self, password: &String) -> bool {
        let connection = &mut Database::new().establish_connection();
        // Get the user account in DB
        let account_to_auth = match accounts::table
            .find(&self.login)
            .select(Account::as_select())
            .first(connection)
        {
            Ok(account) => account,
            Err(err) => return false,
        };

        // Import user hashed password and verify it
        let parsed_hash = PasswordHash::new(&account_to_auth.password)
            .expect("Error while importing user password hash.");

        match self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
