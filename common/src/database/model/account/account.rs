use diesel::{
    dsl::insert_into, Connection as _, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl,
    SelectableHelper,
};

use crate::database::schema::accounts::dsl::*;

use super::{Account, CreateAccount, UpdateAccount};

type Connection = diesel::pg::PgConnection;

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

    /// Execute `operation` in a transaction after atomically reading and locking
    /// the account row identified by `user_login`.
    ///
    /// If the operation returns `Ok(_)`, the transaction is committed.
    ///
    /// If the operation returns `Err(_)`, the transaction is rolled back and
    /// the error is returned to the caller.
    pub fn atomic<F, R>(db: &mut Connection, user_login: &str, operation: F) -> QueryResult<R>
    where
        F: FnOnce(&mut Connection, Account) -> QueryResult<R>,
    {
        db.transaction(|conn| {
            let account = accounts
                .filter(login.eq(user_login))
                .for_update()
                .first::<Account>(conn)?;

            operation(conn, account)
        })
    }
}

impl From<Account> for UpdateAccount {
    fn from(value: Account) -> Self {
        Self {
            login: Some(value.login.clone()),
            password: Some(value.password.clone()),
            login_failure_count: Some(value.login_failure_count),
            login_window_started_at: Some(value.login_window_started_at),
            locked_until: Some(value.locked_until),
            lockout_count: Some(value.lockout_count),
        }
    }
}
