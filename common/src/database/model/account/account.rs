use std::error::Error;
use std::fmt::Display;

use diesel::{
    dsl::insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper,
};

use crate::database::schema::accounts::dsl::*;

use super::{Account, CreateAccount, UpdateAccount};

type Connection = diesel::pg::PgConnection;

#[derive(Debug)]
pub enum AccountError {
    NotConnected,
}

impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AccountError::NotConnected => write!(f, "NotConnected"),
        }
    }
}

impl Error for AccountError {}

impl Account {
    /// Create an Account in DB with the given CreateAccount item
    pub fn create(db: &mut Connection, item: &CreateAccount) -> QueryResult<Self> {
        insert_into(accounts).values(item).get_result(db)
    }

    /// Return the Account in DB for the given user_login
    pub fn read(db: &mut Connection, user_login: &String) -> QueryResult<Self> {
        accounts.filter(login.eq(user_login)).first::<Account>(db)
    }

    /// Return all the Account in DB for the given user_login
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        accounts.load::<Account>(db)
    }

    /// Update an Account in DB for the given user_login according to the given UpdateAccount item.
    pub fn update(
        db: &mut Connection,
        user_login: &String,
        item: &UpdateAccount,
    ) -> QueryResult<Self> {
        diesel::update(accounts.filter(login.eq(user_login)))
            .set(item)
            .returning(Account::as_returning())
            .get_result(db)
    }

    /// Delete Account in DB of the given user_login
    pub fn delete(db: &mut Connection, user_login: &String) -> QueryResult<()> {
        diesel::delete(accounts.filter(login.eq(user_login))).execute(db)?;
        Ok(())
    }

    /// Set session_token with the given token for the given user_login in DB
    pub fn set_token(
        db: &mut Connection,
        user_login: &String,
        token: &String,
    ) -> QueryResult<Self> {
        Self::update(
            db,
            user_login,
            &UpdateAccount {
                login: None,
                password: None,
                session_token: Some(Some(token.clone())),
            },
        )
    }

    pub fn is_connected(db: &mut Connection, user_login: &String) -> Result<(), AccountError> {
        if let Ok(account) = Self::read(db, user_login) {
            if let Some(_) = account.session_token {
                return Ok(());
            }
        }
        Err(AccountError::NotConnected)
    }

    // Set the Account as logged out in DB for the given user_login
    pub fn logout(
        db: &mut Connection,
        user_login: &String,
        token: Option<String>,
    ) -> QueryResult<()> {
        diesel::update(
            accounts
                .filter(login.eq(user_login))
                .filter(session_token.eq(token)),
        )
        .set(session_token.eq(Option::<String>::None))
        .execute(db)?;
        Ok(())
    }
}
