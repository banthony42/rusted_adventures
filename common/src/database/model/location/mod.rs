use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, Selectable, SelectableHelper,
};

use diesel_geometry::data_types::PgPoint;

use crate::database::{
    db::DBConnection,
    schema::{entities, locations},
};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct Location {
    pub id: i32,
    pub map: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct CreateLocation {
    pub map: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
pub struct UpdateLocation {
    pub map: PgPoint,
    pub cell: PgPoint,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::locations)]
#[diesel(treat_none_as_null = true)]
pub struct UpdateLocationDestination {
    pub destination: Option<PgPoint>,
}

impl UpdateLocationDestination {
    pub fn new(x: f64, y: f64) -> Self {
        UpdateLocationDestination {
            destination: Some(PgPoint(x, y)),
        }
    }
}

impl Location {
    /// Create a Location in DB with the given CreateLocation item
    pub fn create(db: &mut DBConnection, item: &CreateLocation) -> QueryResult<Self> {
        diesel::insert_into(locations::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Location in DB according to its id
    pub fn read(db: &mut DBConnection, id: &i32) -> QueryResult<Self> {
        locations::table
            .filter(locations::id.eq(id))
            .first::<Location>(db)
    }

    fn reset_destination_when_reached(
        db: &mut DBConnection,
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
    pub fn update(db: &mut DBConnection, id: &i32, item: &UpdateLocation) -> QueryResult<Self> {
        let location = diesel::update(locations::table.filter(locations::id.eq(id)))
            .set(item)
            .returning(Location::as_returning())
            .get_result(db)?;

        Self::reset_destination_when_reached(db, location)
    }

    /// Update a Location in DB associated to the given entity_id according to the UpdateLocation item.
    pub fn update_by_entity_id(
        db: &mut DBConnection,
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
        db: &mut DBConnection,
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
        db: &mut DBConnection,
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
    pub fn delete(db: &mut DBConnection, id: &i32) -> QueryResult<()> {
        diesel::delete(locations::table.filter(locations::id.eq(id))).execute(db)?;
        Ok(())
    }

    /// Return all the Location in DB
    pub fn read_all(db: &mut DBConnection) -> QueryResult<Vec<Self>> {
        locations::table.load::<Location>(db)
    }
}
