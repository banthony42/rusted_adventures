use diesel::{
    dsl::insert_into,
    prelude::{AsChangeset, Insertable, Queryable},
    ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl, Selectable, SelectableHelper,
};

use crate::database::db::DBConnection;
use crate::database::schema::sessions::dsl::*;
use crate::database::schema::{accounts, sessions};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::sessions)]
pub struct Session {
    pub id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub token_hash: String,
    pub created_at: chrono::NaiveDateTime,
    pub expires_at: chrono::NaiveDateTime,
    pub last_used_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::sessions)]
pub struct CreateSession {
    pub account_id: uuid::Uuid,
    pub token_hash: String,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::sessions)]
pub struct UpdateSession {
    pub token_hash: Option<String>,
    pub last_used_at: chrono::NaiveDateTime,
}

impl Session {
    pub fn create(db: &mut DBConnection, item: CreateSession) -> QueryResult<Self> {
        insert_into(sessions).values(item).get_result(db)
    }

    pub fn read_by_login(db: &mut DBConnection, login: &String) -> QueryResult<Self> {
        let r = sessions
            .inner_join(accounts::table.on(accounts::id.eq(sessions::account_id)))
            .filter(accounts::login.eq(login))
            .select(Session::as_select())
            .get_result(db);
        tracing::info!("read_by_login: {login}: {r:?}");
        r
    }

    pub fn read_by_token_if_owned_by(
        db: &mut DBConnection,
        hash: &String,
        login: &String,
    ) -> QueryResult<Self> {
        let r = sessions
            .inner_join(accounts::table.on(accounts::id.eq(sessions::account_id)))
            .filter(sessions::token_hash.eq(hash.clone()))
            .filter(accounts::login.eq(login))
            .select(Session::as_select())
            .get_result(db);
        tracing::info!("read_by_token_if_owned_by: {hash}: {r:?}");
        r
    }

    pub fn update(
        db: &mut DBConnection,
        session_id: &uuid::Uuid,
        item: UpdateSession,
    ) -> QueryResult<Self> {
        diesel::update(sessions.filter(id.eq(session_id)))
            .set(item)
            .returning(Session::as_returning())
            .get_result(db)
    }

    pub fn delete(db: &mut DBConnection, session_id: &uuid::Uuid) -> QueryResult<()> {
        diesel::delete(sessions.filter(id.eq(session_id))).execute(db)?;
        Ok(())
    }

    pub fn delete_by_account_login(db: &mut DBConnection, login: &String) -> QueryResult<()> {
        diesel::delete(
            sessions::table.filter(
                sessions::account_id.eq_any(
                    accounts::table
                        .filter(accounts::login.eq(login))
                        .select(accounts::id),
                ),
            ),
        )
        .execute(db)?;
        Ok(())
    }

    pub fn delete_by_token_if_owned_by(
        db: &mut DBConnection,
        hash: &String,
        login: &String,
    ) -> QueryResult<()> {
        diesel::delete(
            sessions::table
                .filter(
                    sessions::account_id.eq_any(
                        accounts::table
                            .filter(accounts::login.eq(login))
                            .select(accounts::id),
                    ),
                )
                .filter(sessions::token_hash.eq(hash)),
        )
        .returning(sessions::id)
        .get_result::<uuid::Uuid>(db)?;

        Ok(())
    }
}
