use diesel::{dsl::insert_into, QueryDsl, QueryResult, RunQueryDsl};
use diesel::{ExpressionMethods, JoinOnDsl, SelectableHelper};

use crate::database::model::session::UpdateSession;
use crate::database::schema::sessions::dsl::*;
use crate::database::schema::{accounts, sessions};

use super::{CreateSession, Session};

type Connection = diesel::pg::PgConnection;

impl Session {
    pub fn create(db: &mut Connection, item: CreateSession) -> QueryResult<Self> {
        insert_into(sessions).values(item).get_result(db)
    }

    pub fn read_by_login(db: &mut Connection, login: &String) -> QueryResult<Self> {
        let r = sessions
            .inner_join(accounts::table.on(accounts::id.eq(sessions::account_id)))
            .filter(accounts::login.eq(login))
            .select(Session::as_select())
            .get_result(db);
        tracing::info!("read_by_login: {login}: {r:?}");
        r
    }

    pub fn read_by_token_if_owned_by(
        db: &mut Connection,
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
        db: &mut Connection,
        session_id: &uuid::Uuid,
        item: UpdateSession,
    ) -> QueryResult<Self> {
        diesel::update(sessions.filter(id.eq(session_id)))
            .set(item)
            .returning(Session::as_returning())
            .get_result(db)
    }

    pub fn delete(db: &mut Connection, session_id: &uuid::Uuid) -> QueryResult<()> {
        diesel::delete(sessions.filter(id.eq(session_id))).execute(db)?;
        Ok(())
    }

    pub fn delete_by_account_login(db: &mut Connection, login: &String) -> QueryResult<()> {
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
        db: &mut Connection,
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
        .execute(db)?;
        Ok(())
    }
}
