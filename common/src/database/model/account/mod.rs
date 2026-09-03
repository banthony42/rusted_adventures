use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    Selectable,
};

pub mod account;

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
