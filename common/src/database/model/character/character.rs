use diesel::{
    allow_columns_to_appear_in_same_group_by_clause, result::Error as DieselError,
    ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper,
};
use diesel_geometry::{data_types::PgPoint, prelude::PgSameAsExpressionMethods};

use crate::database::{
    model::{character::PgClasses, location::Location, EntityIdentifiable},
    schema::{accounts, characters, entities, locations},
};

use crate::grpc_codegen::Entity as RpcEntity;
use crate::grpc_codegen::Location as RpcLocation;

use super::{Character, CreateCharacter, UpdateCharacter};

type Connection = diesel::pg::PgConnection;

allow_columns_to_appear_in_same_group_by_clause!(characters::name, locations::world);

type CharacterInfoData = (i32, String, PgClasses, PgPoint, PgPoint, Option<PgPoint>);

impl Into<CharacterInfo> for CharacterInfoData {
    fn into(self) -> CharacterInfo {
        let (id, name, class, world, cell, destination) = self;
        CharacterInfo {
            id,
            name,
            class,
            world,
            cell,
            destination,
        }
    }
}

pub struct CharacterInfo {
    pub id: i32,
    pub name: String,
    pub class: PgClasses,
    pub world: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

impl CharacterInfo {
    pub fn from_character(
        db: &mut Connection,
        character: &Character,
    ) -> Result<CharacterInfo, DieselError> {
        let location = Location::read(db, &character.entity_id)?;
        Ok(CharacterInfo {
            id: character.id,
            name: character.name.clone(),
            class: character.class.clone(),
            world: location.world,
            cell: location.cell,
            destination: location.destination,
        })
    }
}

impl EntityIdentifiable for CharacterInfo {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl EntityIdentifiable for Character {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl Into<RpcEntity> for CharacterInfo {
    fn into(self) -> RpcEntity {
        RpcEntity {
            uuid: self.identifier(),
            name: self.name,
            family: Some(self.class.into()),
            location: Some(RpcLocation {
                world: Some(self.world.into()),
                cell: Some(self.cell.into()),
            }),
        }
    }
}

impl Character {
    /// Create a Character in DB with the given CreateCharacter item
    pub fn create(db: &mut Connection, item: &CreateCharacter) -> QueryResult<Self> {
        diesel::insert_into(characters::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Character in DB for the given character id
    pub fn read(db: &mut Connection, id: &i32) -> QueryResult<Self> {
        characters::table
            .filter(characters::id.eq(id))
            .first::<Character>(db)
    }

    /// Return all the Character in DB
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        characters::table.load::<Character>(db)
    }

    /// Update an Character in DB for the given character id according to the given UpdateCharacter item.
    pub fn update(db: &mut Connection, id: &i32, item: &UpdateCharacter) -> QueryResult<Self> {
        diesel::update(characters::table.filter(characters::id.eq(id)))
            .set(item)
            .returning(Character::as_returning())
            .get_result(db)
    }

    /// Delete Character in DB of the given character id
    pub fn delete(db: &mut Connection, id: &i32) -> QueryResult<usize> {
        diesel::delete(characters::table.filter(characters::id.eq(id))).execute(db)
    }

    /// Return the Character in DB for the given account login
    pub fn read_by_account(db: &mut Connection, account_login: &String) -> QueryResult<Vec<Self>> {
        Ok(accounts::table
            .inner_join(characters::table)
            .filter(accounts::login.eq(account_login))
            .select(Character::as_select())
            .load(db)?)
    }

    pub fn characters_names_group_by_world(
        db: &mut Connection,
        eid: i32,
    ) -> QueryResult<Vec<String>> {
        characters::table
            .inner_join(entities::table)
            .inner_join(locations::table.on(locations::id.eq(entities::location_id)))
            .group_by((locations::world, characters::name))
            .filter(entities::id.eq(eid))
            .select(characters::name)
            .load(db)
    }

    pub fn characters_names_at_location(
        db: &mut Connection,
        world_coord: PgPoint,
        exclude: &String,
    ) -> QueryResult<Vec<String>> {
        characters::table
            .inner_join(entities::table)
            .inner_join(locations::table.on(locations::id.eq(entities::location_id)))
            .filter(locations::world.same_as(world_coord))
            .filter(characters::name.ne(exclude))
            .select(characters::name)
            .load(db)
    }

    pub fn read_all_by_world(
        db: &mut Connection,
        world_coord: PgPoint,
    ) -> QueryResult<Vec<CharacterInfo>> {
        let data: Vec<CharacterInfoData> = characters::table
            .inner_join(entities::table)
            .inner_join(locations::table.on(locations::id.eq(entities::location_id)))
            .inner_join(accounts::table.on(characters::account_id.eq(accounts::id)))
            .filter(locations::world.same_as(world_coord))
            .filter(accounts::session_token.is_not_null())
            .select((
                characters::id,
                characters::name,
                characters::class,
                locations::world,
                locations::cell,
                locations::destination,
            ))
            .load(db)?;

        Ok(data.iter().map(|data| data.to_owned().into()).collect())
    }
}
