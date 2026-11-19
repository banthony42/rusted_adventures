use std::{fmt::Display, io::Write};

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, Selectable,
};

use crate::database::{
    db::DBConnection,
    schema::{bestiary, sql_types::Pgspecies},
};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone, Copy)]
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

impl Display for PgSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgSpecies::Bouftou => write!(f, "Bouftou"),
            PgSpecies::Crabedoeuf => write!(f, "Crabedoeuf"),
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

pub struct Bestiary;

impl Bestiary {
    /// Create a Bestiary entry in DB with the given CreateBestiaryEntry item
    pub fn create(db: &mut DBConnection, item: &CreateBestiaryEntry) -> QueryResult<BestiaryEntry> {
        diesel::insert_into(bestiary::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Bestiary entry in DB for the given bestiary id
    pub fn read_by_id(db: &mut DBConnection, id: &i32) -> QueryResult<BestiaryEntry> {
        bestiary::table
            .filter(bestiary::id.eq(id))
            .first::<BestiaryEntry>(db)
    }

    /// Return the Bestiary entry in DB for the given bestiary species
    pub fn read_by_species(
        db: &mut DBConnection,
        species: &PgSpecies,
    ) -> QueryResult<BestiaryEntry> {
        bestiary::table
            .filter(bestiary::species.eq(species))
            .first::<BestiaryEntry>(db)
    }
}
