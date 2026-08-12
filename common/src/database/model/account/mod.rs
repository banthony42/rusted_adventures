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
}
