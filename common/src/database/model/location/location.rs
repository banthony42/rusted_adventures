use diesel::{
    associations::HasTable, dsl::insert_into, ExpressionMethods, QueryDsl, QueryResult,
    RunQueryDsl, SelectableHelper,
};

use crate::database::{model::location::UpdateLocationDestination, schema::locations::dsl::*};

use super::{CreateLocation, Location, UpdateLocation};

type Connection = diesel::pg::PgConnection;

impl Location {
    /// Create a Location in DB with the given CreateLocation item
    pub fn create(db: &mut Connection, item: &CreateLocation) -> QueryResult<Self> {
        insert_into(locations).values(item).get_result(db)
    }

    /// Return the Location in DB for the given entity id
    pub fn read(db: &mut Connection, e_id: &i32) -> QueryResult<Self> {
        locations.filter(entity_id.eq(e_id)).first::<Location>(db)
    }

    /// Update a Location in DB for the given entity id according to the given UpdateLocation item.
    pub fn update(db: &mut Connection, e_id: &i32, item: &UpdateLocation) -> QueryResult<Self> {
        diesel::update(locations.filter(entity_id.eq(e_id)))
            .set(item)
            .returning(Location::as_returning())
            .get_result(db)
    }

    pub fn update_destination(
        db: &mut Connection,
        e_id: &i32,
        new_destination: UpdateLocationDestination,
    ) -> QueryResult<usize> {
        diesel::update(locations.filter(entity_id.eq(e_id)))
            .set(new_destination)
            .execute(db)
    }

    /// Delete Location in DB of the given entity id
    pub fn delete(db: &mut Connection, e_id: &i32) -> QueryResult<()> {
        diesel::delete(locations.filter(entity_id.eq(e_id))).execute(db)?;
        Ok(())
    }

    /// Return all the Location in DB
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        locations.load::<Location>(db)
    }
}
