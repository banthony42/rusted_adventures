use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper};

use crate::database::{
    model::location::UpdateLocationDestination,
    schema::{entities, locations},
};

use super::{CreateLocation, Location, UpdateLocation};

type Connection = diesel::pg::PgConnection;

impl Location {
    /// Create a Location in DB with the given CreateLocation item
    pub fn create(db: &mut Connection, item: &CreateLocation) -> QueryResult<Self> {
        diesel::insert_into(locations::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Location in DB according to its id
    pub fn read(db: &mut Connection, id: &i32) -> QueryResult<Self> {
        locations::table
            .filter(locations::id.eq(id))
            .first::<Location>(db)
    }

    fn reset_destination_when_reached(
        db: &mut Connection,
        location: Location,
    ) -> QueryResult<Self> {
        if location.destination == Some(location.cell) {
            return Self::update_destination(
                db,
                &location.id,
                UpdateLocationDestination { destination: None },
            );
        }
        Ok(location)
    }

    /// Update a Location in DB for the given id according to the UpdateLocation item.
    pub fn update(db: &mut Connection, id: &i32, item: &UpdateLocation) -> QueryResult<Self> {
        let location = diesel::update(locations::table.filter(locations::id.eq(id)))
            .set(item)
            .returning(Location::as_returning())
            .get_result(db)?;

        Self::reset_destination_when_reached(db, location)
    }

    /// Update a Location in DB associated to the given entity_id according to the UpdateLocation item.
    pub fn update_by_entity_id(
        db: &mut Connection,
        id: &i32,
        item: UpdateLocation,
    ) -> QueryResult<Self> {
        // UPDATE with JOIN seems not handled yet by diesel
        // That's why i have to retrieve the location_id first
        let location_id: i32 = entities::table
            .filter(entities::id.eq(id))
            .select(entities::location_id)
            .first(db)?;

        let location = diesel::update(locations::table)
            .filter(locations::id.eq(location_id))
            .set(item)
            .returning(Location::as_returning())
            .get_result(db)?;

        Self::reset_destination_when_reached(db, location)
    }

    /// Update a destination in Location table, for the given id, according to the UpdateLocationDestination item
    pub fn update_destination(
        db: &mut Connection,
        id: &i32,
        item: UpdateLocationDestination,
    ) -> QueryResult<Self> {
        diesel::update(locations::table.filter(locations::id.eq(id)))
            .set(item)
            .returning(Location::as_returning())
            .get_result(db)
    }

    /// Update a destination in Location table, for the given entity id, according to the UpdateLocationDestination item
    pub fn update_destination_by_entity_id(
        db: &mut Connection,
        id: &i32,
        item: UpdateLocationDestination,
    ) -> QueryResult<Self> {
        // UPDATE with JOIN seems not handled yet by diesel
        // That's why i have to retrieve the location_id first
        let location_id: i32 = entities::table
            .filter(entities::id.eq(id))
            .select(entities::location_id)
            .first(db)?;

        diesel::update(locations::table.filter(locations::id.eq(location_id)))
            .set(item)
            .returning(Location::as_returning())
            .get_result(db)
    }

    /// Delete Location in DB of the given entity id
    pub fn delete(db: &mut Connection, id: &i32) -> QueryResult<()> {
        diesel::delete(locations::table.filter(locations::id.eq(id))).execute(db)?;
        Ok(())
    }

    /// Return all the Location in DB
    pub fn read_all(db: &mut Connection) -> QueryResult<Vec<Self>> {
        locations::table.load::<Location>(db)
    }
}
