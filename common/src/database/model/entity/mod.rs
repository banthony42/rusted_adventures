use std::io::Write;

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{AsChangeset, Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    Selectable,
};

pub mod entity;

use crate::{
    database::schema::sql_types::Pgspecies,
    grpc_codegen::{entity::Family, Species},
};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgspecies)]
pub enum PgSpecies {
    Bouftou,
    Crabedoeuf,
}

impl Into<Family> for PgSpecies {
    fn into(self) -> Family {
        match self {
            PgSpecies::Bouftou => Family::Species(Species::Bouftou.into()),
            PgSpecies::Crabedoeuf => Family::Species(Species::Crabedoeuf.into()),
        }
    }
}

impl ToSql<Pgspecies, Pg> for PgSpecies {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PgSpecies::Crabedoeuf => out.write_all(b"Crabedoeuf")?,
            PgSpecies::Bouftou => out.write_all(b"Bouftou")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Pgspecies, Pg> for PgSpecies {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Bouftou" => Ok(PgSpecies::Bouftou),
            b"Crabedoeuf" => Ok(PgSpecies::Crabedoeuf),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct Entity {
    pub id: i32,
    pub location_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct CreateEntity {
    pub location_id: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct UpdateEntitiy {
    pub location_id: i32,
}
