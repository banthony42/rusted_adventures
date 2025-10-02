use std::io::Write;

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    Selectable,
};

use crate::database::schema::sql_types::Pgspecies;

pub mod bestiary;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgspecies)]
pub enum PgSpecies {
    Bouftou,
    Crabedoeuf,
}

impl ToSql<Pgspecies, Pg> for PgSpecies {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PgSpecies::Bouftou => out.write_all(b"Bouftou")?,
            PgSpecies::Crabedoeuf => out.write_all(b"Crabedoeuf")?,
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

impl ToString for PgSpecies {
    fn to_string(&self) -> String {
        match self {
            PgSpecies::Bouftou => String::from("Bouftou"),
            PgSpecies::Crabedoeuf => String::from("Cradeboeuf"),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::database::schema::bestiary)]
pub struct BestiaryEntry {
    pub id: i32,
    pub species: PgSpecies,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::bestiary)]
pub struct CreateBestiaryEntry {
    pub id: i32,
    pub species: PgSpecies,
    pub name: String,
}
