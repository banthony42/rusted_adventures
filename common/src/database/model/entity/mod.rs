use std::io::Write;

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    dsl::insert_into,
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{AsChangeset, Insertable, Queryable},
    serialize::{self, IsNull, Output, ToSql},
    ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, Selectable, SelectableHelper,
};

use crate::{
    database::{
        db::DBConnection,
        schema::{entities::dsl::*, sql_types::Pgspecies},
    },
    grpc_codegen::{entity::Family, Species},
};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgspecies)]
pub enum PgSpecies {
    Bouftou,
    Crabedoeuf,
}

impl From<PgSpecies> for Family {
    fn from(val: PgSpecies) -> Self {
        match val {
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

impl Entity {
    /// Create an Entity in DB with the given CreateEntity item
    pub fn create(db: &mut DBConnection, item: &CreateEntity) -> QueryResult<Self> {
        insert_into(entities).values(item).get_result(db)
    }

    /// Return the Entity in DB for the given entity_id
    pub fn read(db: &mut DBConnection, entity_id: &i32) -> QueryResult<Self> {
        entities.filter(id.eq(entity_id)).first::<Entity>(db)
    }

    /// Update an Entity in DB for the given entity_id according to the given UpdateEntity item.
    pub fn update(
        db: &mut DBConnection,
        entity_id: &i32,
        item: &UpdateEntitiy,
    ) -> QueryResult<Self> {
        diesel::update(entities.filter(id.eq(entity_id)))
            .set(item)
            .returning(Entity::as_returning())
            .get_result(db)
    }

    /// Delete an Entity by entity_id in the DB
    pub fn delete(db: &mut DBConnection, entity_id: &i32) -> QueryResult<()> {
        diesel::delete(entities.filter(id.eq(entity_id))).execute(db)?;
        Ok(())
    }

    /// Return all Entities from the DB
    pub fn read_all(db: &mut DBConnection) -> QueryResult<Vec<Self>> {
        entities.load::<Entity>(db)
    }
}
