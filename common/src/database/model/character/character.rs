use diesel::{
    associations::HasTable, dsl::insert_into, sql_types::Integer, ExpressionMethods, QueryDsl,
    QueryResult, RunQueryDsl, SelectableHelper,
};
use diesel_geometry::{data_types::PgPoint, prelude::PgSameAsExpressionMethods};
use uuid::Uuid;

use crate::database::{
    model::character::Classes,
    schema::{
        accounts::{self, session_token},
        characters::dsl::*,
        entities,
        locations::dsl::*,
    },
};

use super::{Character, CreateCharacter, UpdateCharacter};

type Connection = diesel::pg::PgConnection;

impl Character {
    /// Create a Character in DB with the given CreateCharacter item
    pub fn create(db: &mut Connection, item: &CreateCharacter) -> QueryResult<Self> {
        insert_into(characters).values(item).get_result(db)
    }

    /// Return the Character in DB for the given character id
    pub fn read(db: &mut Connection, char_id: &i32) -> QueryResult<Self> {
        characters.filter(id.eq(char_id)).first::<Character>(db)
    }

    pub fn read_all_by_account_login(
        db: &mut Connection,
        account_login: &String,
    ) -> QueryResult<Vec<Self>> {
        let _account_id: Uuid = accounts::table
            .filter(accounts::login.eq(account_login))
            .select(accounts::id)
            .get_result(db)?;

        // get all of _account_id's characters
        let chars = characters::table()
            .filter(accounts::id.eq(_account_id))
            .inner_join(accounts::table)
            .select(Character::as_select())
            .load(db)?;

        // Instead of getting all characters
        // select with : where login = entity.name
        Ok(chars)
    }

    pub fn read_all_by_world(
        db: &mut Connection,
        world_coord: PgPoint,
    ) -> QueryResult<Vec<String>> {
        let entities = locations::table()
            .filter(world.same_as(world_coord))
            .inner_join(entities::table)
            .select(entities::name)
            .load(db)?;

        Ok(entities)
    }

    pub fn get_all_entities_by_map(
        db: &mut Connection,
        login: &String,
        world_coord: PgPoint,
    ) -> QueryResult<Vec<(String, String, Classes, PgPoint)>> {
        let data: Vec<(Uuid, i32, String, Classes, PgPoint)> = entities::dsl::entities::table()
            .inner_join(locations)
            .inner_join(characters)
            .select((account_id, entities::id, entities::name, class, map))
            .filter(entities::name.ne(login))
            .filter(world.same_as(world_coord))
            .load(db)?;

        let connected_player: Vec<Uuid> = accounts::table
            .filter(accounts::session_token.is_not_null())
            .select(accounts::id)
            .load(db)?;

        Ok(data
            .iter()
            .filter(|d| connected_player.contains(&d.0))
            .map(|d| (format!("{}.{}", d.0, d.1), d.2.clone(), d.3.clone(), d.4))
            .collect())
    }

    /// Update an Character in DB for the given character id according to the given UpdateCharacter item.
    pub fn update(db: &mut Connection, char_id: &i32, item: &UpdateCharacter) -> QueryResult<Self> {
        diesel::update(characters.filter(id.eq(char_id)))
            .set(item)
            .returning(Character::as_returning())
            .get_result(db)
    }

    /// Delete Character in DB of the given character id
    pub fn delete(db: &mut Connection, char_id: &i32) -> QueryResult<()> {
        diesel::delete(characters.filter(id.eq(char_id))).execute(db)?;
        Ok(())
    }

    /// Return all the Character in DB
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        characters.load::<Character>(db)
    }
}
