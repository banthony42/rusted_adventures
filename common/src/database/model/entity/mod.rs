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
    database::schema::sql_types::Pgbestiary,
    grpc_codegen::{entity::Family, Species},
};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgbestiary)]
pub enum Bestiary {
    Bouftou,
    Crabedoeuf,
}

impl Into<Family> for Bestiary {
    fn into(self) -> Family {
        match self {
            Bestiary::Bouftou => Family::Species(Species::Bouftou.into()),
            Bestiary::Crabedoeuf => Family::Species(Species::Crabedoeuf.into()),
        }
    }
}

impl ToSql<Pgbestiary, Pg> for Bestiary {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Bestiary::Crabedoeuf => out.write_all(b"Crabedoeuf")?,
            Bestiary::Bouftou => out.write_all(b"Bouftou")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Pgbestiary, Pg> for Bestiary {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Bouftou" => Ok(Bestiary::Bouftou),
            b"Crabedoeuf" => Ok(Bestiary::Crabedoeuf),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::entities)]
pub struct Entity {
    pub id: i32,
    pub uuid: uuid::Uuid,
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
