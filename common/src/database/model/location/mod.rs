use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    Selectable,
};

use diesel_geometry::data_types::PgPoint;

pub mod location;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct Location {
    pub entity_id: i32,
    pub world: PgPoint,
    pub map: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct CreateLocation {
    pub entity_id: i32,
    pub world: PgPoint,
    pub map: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct UpdateLocation {
    pub world: PgPoint,
    pub map: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct UpdateLocationDestination {
    pub destination: Option<PgPoint>,
}

impl UpdateLocationDestination {
    pub fn new(x: f64, y: f64) -> Self {
        UpdateLocationDestination {
            destination: Some(PgPoint(x, y)),
        }
    }
}
