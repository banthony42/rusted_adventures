use std::io::Write;

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{AsChangeset, Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    Selectable,
};

use crate::{database::schema::sql_types::Pgclass, grpc_codegen::entity::Family, grpc_codegen::Classes as RpcClasses};

pub mod character;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgclass)]
pub enum Classes {
    Warrior,
    Mage,
}

impl Into<Family> for Classes {
    fn into(self) -> Family {
        match self {
            Classes::Warrior => Family::Class(RpcClasses::Warrior.into()),
            Classes::Mage => Family::Class(RpcClasses::Mage.into()),
        }
    }
}

impl ToSql<Pgclass, Pg> for Classes {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Classes::Warrior => out.write_all(b"Warrior")?,
            Classes::Mage => out.write_all(b"Mage")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Pgclass, Pg> for Classes {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Warrior" => Ok(Classes::Warrior),
            b"Mage" => Ok(Classes::Mage),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct Character {
    pub id: i32,
    pub account_id: uuid::Uuid,
    pub entity_id: i32,
    pub class: Classes,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct CreateCharacter {
    pub account_id: uuid::Uuid,
    pub entity_id: i32,
    pub class: Classes,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct UpdateCharacter {
    pub class: Classes,
}
