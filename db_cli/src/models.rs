use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAccount {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::accounts)]
pub struct Account {
    pub login: String,
    pub password: String,
}
