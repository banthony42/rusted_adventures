use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};

use crate::database::model::bestiary::BestiaryEntry;
use crate::database::model::bestiary::{CreateBestiaryEntry, PgSpecies};
use crate::database::schema::bestiary;

type Connection = diesel::pg::PgConnection;

pub struct Bestiary;

impl Bestiary {
    /// Create a Bestiary entry in DB with the given CreateBestiaryEntry item
    pub fn create(db: &mut Connection, item: &CreateBestiaryEntry) -> QueryResult<BestiaryEntry> {
        diesel::insert_into(bestiary::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Bestiary entry in DB for the given bestiary id
    pub fn read_by_id(db: &mut Connection, id: &i32) -> QueryResult<BestiaryEntry> {
        bestiary::table
            .filter(bestiary::id.eq(id))
            .first::<BestiaryEntry>(db)
    }

    /// Return the Bestiary entry in DB for the given bestiary species
    pub fn read_by_species(db: &mut Connection, species: &PgSpecies) -> QueryResult<BestiaryEntry> {
        bestiary::table
            .filter(bestiary::species.eq(species))
            .first::<BestiaryEntry>(db)
    }
}
