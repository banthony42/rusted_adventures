use diesel::{
    allow_columns_to_appear_in_same_group_by_clause, dsl::insert_into, ExpressionMethods,
    JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper,
};
use diesel_geometry::{data_types::PgPoint, prelude::PgSameAsExpressionMethods};
use uuid::Uuid;

use crate::database::{
    model::{character::Classes, entity::Bestiary},
    schema::{accounts, characters, entities, locations, monsters},
};

use super::{Character, CreateCharacter, UpdateCharacter};

type Connection = diesel::pg::PgConnection;

allow_columns_to_appear_in_same_group_by_clause!(entities::name, locations::world);

impl Character {
    /// Create a Character in DB with the given CreateCharacter item
    pub fn create(db: &mut Connection, item: &CreateCharacter) -> QueryResult<Self> {
        insert_into(characters::table).values(item).get_result(db)
    }

    /// Return the Character in DB for the given character id
    pub fn read(db: &mut Connection, char_id: &i32) -> QueryResult<Self> {
        characters::table
            .filter(characters::id.eq(char_id))
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
    pub fn read_by_account_login(
        db: &mut Connection,
        account_login: &String,
    ) -> QueryResult<Option<(Self, String)>> {
        Ok(accounts::table
            .inner_join(characters::table)
            .inner_join(entities::table.on(entities::id.eq(characters::entity_id)))
            .filter(accounts::login.eq(account_login))
            .filter(entities::name.eq(account_login))
            .select((Character::as_select(), entities::name))
            .load(db)?
            .get(0) // For now players have only one character
            .cloned())
    }

    pub fn read_all_on_same_world(db: &mut Connection, eid: i32) -> QueryResult<Vec<String>> {
        locations::table
            .inner_join(entities::table)
            .group_by((locations::world, entities::name))
            .filter(entities::id.eq(eid))
            .select(entities::name)
            .load(db)
    }

    pub fn read_all_by_world(
        db: &mut Connection,
        world_coord: PgPoint,
    ) -> QueryResult<Vec<String>> {
        let entities = locations::table
            .filter(locations::world.same_as(world_coord))
            .inner_join(entities::table)
            .select(entities::name)
            .load(db)?;

        Ok(entities)
    }

    pub fn get_all_monsters_by_world(
        db: &mut Connection,
        world_coord: PgPoint,
    ) -> QueryResult<Vec<(i32, String, Bestiary, PgPoint, Option<PgPoint>)>> {
        let monsters: Vec<(i32, String, Bestiary, PgPoint, Option<PgPoint>)> = entities::table
            .inner_join(locations::table)
            .inner_join(monsters::table)
            .filter(locations::world.same_as(world_coord))
            .select((
                entities::id,
                entities::name,
                monsters::race,
                locations::map,
                locations::destination,
            ))
            .load(db)?;
        Ok(monsters)
    }

    pub fn get_all_players_by_world(
        db: &mut Connection,
        login: &String,
        world_coord: PgPoint,
    ) -> QueryResult<Vec<(String, String, Classes, PgPoint, Option<PgPoint>)>> {
        let players: Vec<(Uuid, i32, String, Classes, PgPoint, Option<PgPoint>)> = entities::table
            .inner_join(locations::table)
            .inner_join(characters::table)
            .inner_join(accounts::table.on(accounts::id.eq(characters::account_id)))
            .select((
                characters::account_id,
                entities::id,
                entities::name,
                characters::class,
                locations::map,
                locations::destination,
            ))
            .filter(accounts::session_token.is_not_null())
            .filter(entities::name.ne(login))
            .filter(locations::world.same_as(world_coord))
            .load(db)?;

        Ok(players
            .iter()
            .map(|d| {
                (
                    format!("{}.{}", d.0, d.1),
                    d.2.clone(),
                    d.3.clone(),
                    d.4,
                    d.5,
                )
            })
            .collect())
    }
}
