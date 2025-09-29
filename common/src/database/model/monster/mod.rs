use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};

pub mod monster;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::database::schema::monsters)]
pub struct Monster {
    pub id: i32,
    pub bestiary_id: i32,
    pub entity_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::monsters)]
pub struct CreateMonster {
    pub bestiary_id: i32,
    pub entity_id: i32,
}

// No Update for monsters
// Because for now, it contains only foreign keys
