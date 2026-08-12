use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    Selectable,
};

pub mod session;

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
