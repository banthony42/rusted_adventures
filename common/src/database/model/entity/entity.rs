use diesel::{
    dsl::insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper,
};

use crate::database::schema::entities::dsl::*;

use super::{CreateEntity, Entity, UpdateEntitiy};

type Connection = diesel::pg::PgConnection;

impl Entity {
    /// Create an Entity in DB with the given CreateEntity item
    pub fn create(db: &mut Connection, item: &CreateEntity) -> QueryResult<Self> {
        insert_into(entities).values(item).get_result(db)
    }

    /// Return the Entity in DB for the given entity_id
    pub fn read(db: &mut Connection, entity_id: &i32) -> QueryResult<Self> {
        entities.filter(id.eq(entity_id)).first::<Entity>(db)
    }

    /// Update an Entity in DB for the given entity_id according to the given UpdateEntity item.
    pub fn update(db: &mut Connection, entity_id: &i32, item: &UpdateEntitiy) -> QueryResult<Self> {
        diesel::update(entities.filter(id.eq(entity_id)))
            .set(item)
            .returning(Entity::as_returning())
            .get_result(db)
    }

    /// Delete an Entity by entity_id in the DB
    pub fn delete(db: &mut Connection, entity_id: &i32) -> QueryResult<()> {
        diesel::delete(entities.filter(id.eq(entity_id))).execute(db)?;
        Ok(())
    }

    /// Return all Entities from the DB
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        entities.load::<Entity>(db)
    }
}
