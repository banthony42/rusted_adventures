use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    Selectable,
};

use diesel_geometry::data_types::PgPoint;

pub mod location;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct Location {
    pub id: i32,
    pub map: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct CreateLocation {
    pub map: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct UpdateLocation {
    pub map: PgPoint,
    pub cell: PgPoint,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
#[diesel(treat_none_as_null = true)]
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
