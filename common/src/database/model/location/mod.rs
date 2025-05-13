use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::{AsChangeset, Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    Selectable,
};

use crate::database::schema::sql_types::Point;

pub mod location;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Point)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl ToSql<Point, Pg> for Coord {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_f64::<NetworkEndian>(self.x)?;
        out.write_f64::<NetworkEndian>(self.y)?;
        Ok(IsNull::No)
    }
}

impl FromSql<Point, Pg> for Coord {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let mut bytes = bytes.as_bytes();
        let x = bytes.read_f64::<NetworkEndian>()?;
        let y = bytes.read_f64::<NetworkEndian>()?;
        Ok(Coord { x, y })
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct Location {
    pub entity_id: i32,
    pub world: Coord,
    pub map: Option<Coord>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct CreateLocation {
    pub entity_id: i32,
    pub world: Coord,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct UpdateLocation {
    pub world: Coord,
}
