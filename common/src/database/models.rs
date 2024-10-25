use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(treat_none_as_null = true)]
pub struct NewAccount {
    pub login: String,
    pub password: String,
    pub session_token: Option<String>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::accounts)]
#[diesel(treat_none_as_null = true)]
pub struct Account {
    pub login: String,
    pub password: String,
    pub session_token: Option<String>,
}
