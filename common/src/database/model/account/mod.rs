use diesel::{
    dsl::insert_into,
    prelude::{AsChangeset, Insertable, Queryable},
    Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, Selectable,
    SelectableHelper,
};

use crate::database::{db::DBConnection, schema::accounts::dsl::*};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::accounts)]
pub struct Account {
    pub id: uuid::Uuid,
    pub login: String,
    pub password: String,
    pub login_failure_count: i32,
    pub login_window_started_at: chrono::NaiveDateTime,
    pub locked_until: Option<chrono::NaiveDateTime>,
    pub lockout_count: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::accounts)]
pub struct CreateAccount {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::accounts)]
pub struct UpdateAccount {
    pub login: Option<String>,
    pub password: Option<String>,
    pub login_failure_count: Option<i32>,
    pub login_window_started_at: Option<chrono::NaiveDateTime>,
    pub locked_until: Option<Option<chrono::NaiveDateTime>>,
    pub lockout_count: Option<i32>,
}

impl Account {
    /// Create an Account in DB with the given CreateAccount item
    pub fn create(db: &mut DBConnection, item: &CreateAccount) -> QueryResult<Self> {
        insert_into(accounts).values(item).get_result(db)
    }

    /// Return the Account in DB for the given user_login
    pub fn read(db: &mut DBConnection, user_login: &String) -> QueryResult<Self> {
        accounts.filter(login.eq(user_login)).first::<Account>(db)
    }

    /// Return all the Account in DB for the given user_login
    pub fn read_all(db: &mut DBConnection) -> QueryResult<Vec<Self>> {
        accounts.load::<Account>(db)
    }

    /// Update an Account in DB for the given user_login according to the given UpdateAccount item.
    pub fn update(
        db: &mut DBConnection,
        user_login: &String,
        item: &UpdateAccount,
    ) -> QueryResult<Self> {
        diesel::update(accounts.filter(login.eq(user_login)))
            .set(item)
            .returning(Account::as_returning())
            .get_result(db)
    }

    /// Delete Account in DB of the given user_login
    pub fn delete(db: &mut DBConnection, user_login: &String) -> QueryResult<()> {
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
    pub fn atomic<F, R>(db: &mut DBConnection, user_login: &str, operation: F) -> QueryResult<R>
    where
        F: FnOnce(&mut DBConnection, Account) -> QueryResult<R>,
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
