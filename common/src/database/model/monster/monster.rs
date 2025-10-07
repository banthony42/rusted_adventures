use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl};
use diesel_geometry::data_types::PgPoint;
use diesel_geometry::prelude::PgSameAsExpressionMethods;

use super::{CreateMonster, Monster};
use crate::database::model::entity::PgSpecies;
use crate::database::model::EntityIdentifiable;
use crate::database::schema::{bestiary, entities, locations, monsters};

use crate::grpc_codegen::Entity as RpcEntity;
use crate::grpc_codegen::Location as RpcLocation;

type Connection = diesel::pg::PgConnection;

type MonsterInfoData = (
    i32,
    i32,
    String,
    PgSpecies,
    PgPoint,
    PgPoint,
    Option<PgPoint>,
);

impl Into<MonsterInfo> for MonsterInfoData {
    fn into(self) -> MonsterInfo {
        let (id, entity_id, name, species, map, cell, destination) = self;
        MonsterInfo {
            id,
            entity_id,
            name: name,
            species,
            map,
            cell,
            destination,
        }
    }
}

pub struct MonsterInfo {
    pub id: i32,
    pub entity_id: i32,
    pub name: String,
    pub species: PgSpecies,
    pub map: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

impl EntityIdentifiable for MonsterInfo {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl Into<RpcEntity> for MonsterInfo {
    fn into(self) -> RpcEntity {
        RpcEntity {
            uuid: self.identifier(),
            name: self.name,
            family: Some(self.species.into()),
            location: Some(RpcLocation {
                map: Some(self.map.into()),
                cell: Some(self.cell.into()),
            }),
        }
    }
}

impl Monster {
    /// Create a Monster in DB with the given CreateCharacter item
    pub fn create(db: &mut Connection, item: &CreateMonster) -> QueryResult<Self> {
        diesel::insert_into(monsters::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Monster in DB for the given monster id
    pub fn read(db: &mut Connection, id: &i32) -> QueryResult<Self> {
        monsters::table
            .filter(monsters::id.eq(id))
            .first::<Monster>(db)
    }

    /// Return all the Monster in DB
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        monsters::table.load::<Monster>(db)
    }

    pub fn read_info(db: &mut Connection, id: &i32) -> QueryResult<MonsterInfo> {
        let monster_data: MonsterInfoData = entities::table
            .inner_join(locations::table)
            .inner_join(monsters::table)
            .inner_join(bestiary::table.on(bestiary::id.eq(monsters::bestiary_id)))
            .filter(monsters::id.eq(id))
            .select((
                monsters::id,
                monsters::entity_id,
                bestiary::name,
                bestiary::species,
                locations::map,
                locations::cell,
                locations::destination,
            ))
            .get_result(db)?;

        Ok(monster_data.into())
    }
    // No Update function for now since Monster values are only constants.

    /// Delete Character in DB of the given character id
    pub fn delete(db: &mut Connection, id: &i32) -> QueryResult<usize> {
        diesel::delete(monsters::table.filter(monsters::id.eq(id))).execute(db)
    }

    pub fn read_all_by_map(
        db: &mut Connection,
        map_coord: PgPoint,
    ) -> QueryResult<Vec<MonsterInfo>> {
        let data: Vec<MonsterInfoData> = entities::table
            .inner_join(locations::table)
            .inner_join(monsters::table)
            .inner_join(bestiary::table.on(bestiary::id.eq(monsters::bestiary_id)))
            .filter(locations::map.same_as(map_coord))
            .select((
                monsters::id,
                monsters::entity_id,
                bestiary::name,
                bestiary::species,
                locations::map,
                locations::cell,
                locations::destination,
            ))
            .load(db)?;

        Ok(data.iter().map(|data| data.to_owned().into()).collect())
    }
}
