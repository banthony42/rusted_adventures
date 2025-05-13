use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    Selectable,
};

pub mod entity;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct Entity {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct CreateEntity {
    pub name: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct UpdateEntitiy {
    pub name: String,
}
