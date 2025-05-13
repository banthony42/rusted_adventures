use diesel::{
    dsl::insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper,
};

use crate::database::schema::characters::dsl::*;

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
