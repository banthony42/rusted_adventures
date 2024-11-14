use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    Selectable,
};

mod account;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::accounts, primary_key(login))]
pub struct Account {
    pub login: String,
    pub password: String,
    pub session_token: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::accounts)]
pub struct CreateAccount {
    pub login: String,
    pub password: String,
    pub session_token: Option<String>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::accounts, primary_key(login))]
pub struct UpdateAccount {
    pub login: Option<String>,
    pub password: Option<String>,
    pub session_token: Option<Option<String>>,
}
