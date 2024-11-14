use std::io::Write;

use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::serialize::{IsNull, ToSql};

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(treat_none_as_null = true)]
pub struct NewAccount {
    pub login: String,
    pub password: String,
    pub session_token: Option<String>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::accounts)]
#[diesel(treat_none_as_null = true)]
pub struct Account {
    pub login: String,
    pub password: String,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, FromSqlRow, AsExpression, PartialEq, Eq)]
#[diesel(sql_type = crate::database::schema::sql_types::Races)]
pub enum Races {
    Player,
    Bouftou,
}

impl ToSql<crate::database::schema::sql_types::Races, diesel::pg::Pg> for Races {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            Races::Bouftou => out.write_all(b"Bouftou")?,
            Races::Player => out.write_all(b"Player")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::database::schema::sql_types::Races, diesel::pg::Pg> for Races {
    fn from_sql(
        bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Bouftou" => Ok(Races::Bouftou),
            b"Player" => Ok(Races::Player),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression)]
#[diesel(sql_type = crate::database::schema::sql_types::Classes)]
pub enum Classes {
    Warrior,
}

impl ToSql<crate::database::schema::sql_types::Classes, diesel::pg::Pg> for Classes {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            Classes::Warrior => out.write_all(b"Warrior")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::database::schema::sql_types::Classes, diesel::pg::Pg> for Classes {
    fn from_sql(
        bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Warrior" => Ok(Classes::Warrior),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::database::schema::player)]
pub struct Player {
    pub name: String,
    pub race: Races,
    pub class: Classes,
}
