use std::io::Write;

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{AsChangeset, Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    Selectable,
};
use rand::distr::{Distribution, StandardUniform};

use crate::{
    database::schema::sql_types::Pgclass, grpc_codegen::entity::Family,
    grpc_codegen::Classes as RpcClasses,
};

pub mod character;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgclass)]
pub enum PgClasses {
    Warrior,
    Mage,
}

impl Into<Family> for PgClasses {
    fn into(self) -> Family {
        match self {
            PgClasses::Warrior => Family::Class(RpcClasses::Warrior.into()),
            PgClasses::Mage => Family::Class(RpcClasses::Mage.into()),
        }
    }
}

impl ToSql<Pgclass, Pg> for PgClasses {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PgClasses::Warrior => out.write_all(b"Warrior")?,
            PgClasses::Mage => out.write_all(b"Mage")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Pgclass, Pg> for PgClasses {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Warrior" => Ok(PgClasses::Warrior),
            b"Mage" => Ok(PgClasses::Mage),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl Distribution<PgClasses> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PgClasses {
        match rng.random_range(0..=1) {
            0 => PgClasses::Warrior,
            _ => PgClasses::Mage,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct Character {
    pub id: i32,
    pub account_id: uuid::Uuid,
    pub entity_id: i32,
    pub name: String,
    pub class: PgClasses,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct CreateCharacter {
    pub account_id: uuid::Uuid,
    pub entity_id: i32,
    pub name: String,
    pub class: PgClasses,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct UpdateCharacter {
    pub class: PgClasses,
    pub name: String,
}
