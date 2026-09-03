use std::env;

use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash, Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    Version,
};

use chrono::{Duration, Local};
use diesel::PgConnection;
use password_hash::SaltString;
use rand::Rng;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::database::db::Database;
use crate::database::model::account::{Account, UpdateAccount};
use crate::database::model::session::{CreateSession, Session, UpdateSession};

const SESSION_TOKEN_EXPIRATION: i64 = 3600 * 2;
const THROTTLING_TIME_WINDOW: i64 = 60 * 15;
const ACCOUNT_LOCK_TIME_1: i64 = 60;

const ACCOUNT_LOCK_TIME_2: Duration = Duration::minutes(5);
const ACCOUNT_LOCK_TIME_3: Duration = Duration::minutes(25);
const ACCOUNT_LOCK_TIME_4: Duration = Duration::minutes(125);
const ACCOUNT_LOCK_TIME_5: Duration = Duration::hours(24);

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Failed to read account in database: {0}")]
    ReadAccount(diesel::result::Error),

    #[error("Failed to update account in database: {0}")]
    UpdateAccount(diesel::result::Error),

    #[error("Failed to create new session: {0}")]
    CreateSession(diesel::result::Error),

    #[error("Failed to parse password hash from database: {0}")]
    ParsePassword(password_hash::errors::Error),

    #[error("Failed to verify password: {0}")]
    VerifyPassword(password_hash::errors::Error),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Error while revoking session account: {0}")]
    RevokeSession(diesel::result::Error),

    #[error("Failed to retrieve couple session, login: {0}")]
    SessionNotFound(diesel::result::Error),

    #[error("Failed to update session: {0}")]
    UpdateSession(diesel::result::Error),

    #[error("Failed to delete session: {0}")]
    DeleteSession(diesel::result::Error),

    #[error("Expired session")]
    SessionExpired,

    #[error("Too many authentication attempt, account locked")]
    AccountLock,
}

impl From<AuthError> for tonic::Status {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::ReadAccount(_) | AuthError::InvalidPassword => {
                Self::unauthenticated("Invalid login or password")
            }
            AuthError::SessionNotFound(_) | AuthError::SessionExpired => {
                Self::unauthenticated("Invalid or expired token")
            }
            AuthError::AccountLock => Self::resource_exhausted("Too many attempts, retry later"),
            _ => Self::internal("Authentication failed"),
        }
    }
}

#[derive(Debug)]
pub struct AuthenticatorConfig {
    pub session_expiration: Duration,
    pub throttling_time_window: Duration,
    pub account_lock_time_1: Duration,
}

impl AuthenticatorConfig {
    pub fn from_env() -> Self {
        Self {
            session_expiration: Duration::seconds(
                env::var("SESSION_TOKEN_EXPIRATION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(SESSION_TOKEN_EXPIRATION),
            ),
            throttling_time_window: Duration::seconds(
                env::var("THROTTLING_TIME_WINDOW")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(THROTTLING_TIME_WINDOW),
            ),
            account_lock_time_1: Duration::seconds(
                env::var("ACCOUNT_LOCK_TIME_1")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(ACCOUNT_LOCK_TIME_1),
            ),
        }
    }
}

pub struct Authenticator {
    login: String,
    account: Option<Account>,
    connection: PgConnection,
    config: AuthenticatorConfig,
}

impl Authenticator {
    fn create_argon2<'a>() -> Argon2<'a> {
        Argon2::new(
            Algorithm::Argon2id,
            Version::default(),
            Params::new(19 * 1024, 2, 1, Some(Params::DEFAULT_OUTPUT_LEN))
                .expect("Error while creating Argon2 context"),
        )
    }

    fn account(&mut self) -> Result<&Account, AuthError> {
        if self.account.is_none() {
            self.account = Some(
                Account::read(&mut self.connection, &self.login).map_err(AuthError::ReadAccount)?,
            );
        }
        Ok(self.account.as_ref().unwrap())
    }

    pub fn new(login: &str) -> Self {
        Authenticator {
            login: login.to_string(),
            account: None,
            connection: Database::new().establish_connection(),
            config: AuthenticatorConfig::from_env(),
        }
    }

    pub fn hash_password(password: String) -> String {
        Self::create_argon2()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .expect("Error while hashing password")
            .to_string()
    }

    pub fn login(&self) -> &String {
        &self.login
    }

    /// Account Lockout guard
    ///
    /// Increment the authentication failure count, then accordingly lock the account after a certain number of failed logins
    /// under a restricted time window.
    /// User can verify account lock state by using `Authenticator::is_allowed`
    ///
    /// https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html#account-lockout
    fn throttling(&mut self) -> Result<(), AuthError> {
        let now = Local::now().naive_local();

        let locked = Account::atomic(&mut self.connection, &self.login, |conn, mut account| {
            if now > account.login_window_started_at + self.config.throttling_time_window {
                account.login_failure_count = 0;
                account.login_window_started_at = now;
            }

            account.login_failure_count += 1;

            let locked = account.login_failure_count >= 5;

            if locked {
                account.locked_until = match account.lockout_count {
                    0 => Some(now + self.config.account_lock_time_1),
                    1 => Some(now + ACCOUNT_LOCK_TIME_2),
                    2 => Some(now + ACCOUNT_LOCK_TIME_3),
                    3 => Some(now + ACCOUNT_LOCK_TIME_4),
                    _ => Some(now + ACCOUNT_LOCK_TIME_5),
                };

                tracing::debug!("account locked until: {:?}", account.locked_until);
                account.lockout_count += 1;
            }

            Account::update(conn, &self.login, &UpdateAccount::from(account))?;

            Ok(locked)
        })
        .map_err(AuthError::UpdateAccount)?;

        if locked {
            Err(AuthError::AccountLock)
        } else {
            Err(AuthError::InvalidPassword)
        }
    }

    pub fn reset_throttling(&mut self) -> Result<(), AuthError> {
        let _ = Account::update(
            &mut self.connection,
            &self.login,
            &UpdateAccount {
                login: None,
                password: None,
                login_failure_count: Some(0),
                login_window_started_at: None,
                locked_until: Some(None),
                lockout_count: Some(0),
            },
        )
        .map_err(AuthError::UpdateAccount)?;

        Ok(())
    }

    pub fn is_allowed(&mut self) -> Result<(), AuthError> {
        match self
            .account()?
            .locked_until
            .is_some_and(|until| Local::now().naive_local() < until)
        {
            true => Err(AuthError::AccountLock),
            false => Ok(()),
        }
    }

    pub fn authenticate(&mut self, password: &String) -> Result<(), AuthError> {
        let account = self.account()?;
        let parsed_hash = PasswordHash::new(&account.password).map_err(AuthError::ParsePassword)?;

        match Self::create_argon2().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(()),
            Err(password_hash::Error::Password) => self.throttling(),
            Err(e) => Err(AuthError::VerifyPassword(e)),
        }
        .inspect_err(|error| tracing::debug!("authenticate: {error:?}"))
    }

    /// Create a new session for the account, following `Session ID Token best practices from OWASP`:
    /// https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
    ///
    /// - Token name should not be extremely descriptive
    /// - Token should use at least 64 bits of entropy / length
    /// - Token content should be meaningless to prevent disclosure
    /// - Token generated by a CSPRNG with size of at least 128 bits
    /// - Token should be generate server side
    pub fn create_session(&mut self) -> Result<String, AuthError> {
        let mut token_bytes = [0u8; 32];
        let mut rng: rand::rngs::StdRng = rand::make_rng();
        rng.fill_bytes(&mut token_bytes);

        let token = hex::encode(token_bytes);
        let token_hash = hex::encode(Sha256::digest(&token));
        let expires_at = Local::now().naive_local() + self.config.session_expiration;
        let account_id = self.account()?.id;

        Session::create(
            &mut self.connection,
            CreateSession {
                token_hash,
                account_id,
                expires_at,
            },
        )
        .map_err(AuthError::CreateSession)
        .inspect_err(|error| tracing::debug!("create session: {error:?}"))?;

        Ok(token)
    }

    /// Assert user has an active and valid session, following `Session ID Token best practices from OWASP`:
    /// https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
    ///
    /// - session token renewal timeout, auto-renew the token without asking re-authentication
    /// - session absolute timeout to force re-authentication
    /// - server should never accept token they have never generated
    pub fn is_connected(&mut self, token: &str) -> Result<(), AuthError> {
        let timenow = Local::now().naive_local();
        let session = Session::read_by_token_if_owned_by(
            &mut self.connection,
            &hex::encode(Sha256::digest(token)),
            &self.login,
        )
        .map_err(AuthError::SessionNotFound)?;

        if timenow > session.expires_at {
            Session::delete(&mut self.connection, &session.id).map_err(AuthError::DeleteSession)?;
            return Err(AuthError::SessionExpired);
        }

        Session::update(
            &mut self.connection,
            &session.id,
            UpdateSession {
                token_hash: None,
                last_used_at: timenow,
            },
        )
        .map_err(AuthError::UpdateSession)?;
        Ok(())
    }

    /// Logout account by revoking its active session, if a `token` is supplied
    /// the database ensure that the given `token` pertain to the `self.login`
    ///
    /// Following `Session ID Token best practices from OWASP`:
    /// https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
    ///
    /// - Client/**Server** should provide manual logout and visible logout button
    /// - Client/**Server** Avoid persistent token, token should be drop at logout
    pub fn revoke_session(&mut self, token: Option<String>) -> Result<(), AuthError> {
        match token {
            None => Session::delete_by_account_login(&mut self.connection, &self.login),
            Some(token) => Session::delete_by_token_if_owned_by(
                &mut self.connection,
                &hex::encode(Sha256::digest(token)),
                &self.login,
            ),
        }
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AuthError::SessionNotFound(e),
            _ => AuthError::RevokeSession(e),
        })
        .inspect_err(|error| tracing::debug!("revoke_session: {error:?}"))
    }
}
