use diesel::{result::Error as DieselError, PgConnection};

use crate::{
    database::{
        db::Database,
        model::{
            bestiary::{bestiary::Bestiary, BestiaryEntry, PgSpecies},
            entity::{CreateEntity, Entity},
            location::{CreateLocation, Location},
            monster::{CreateMonster, Monster},
        },
    },
    record::Record,
    MapCoord, WorldCoord,
};

pub struct MonsterHandler {
    pub connection: PgConnection,
    cache: Record<BestiaryEntry>,
    ttl: Option<u32>,
}

impl MonsterHandler {
    pub fn new() -> Self {
        Self {
            connection: Database::new().establish_connection(),
            cache: Record::new(),
            ttl: Some(900),
        }
    }

    fn get_bestiary_entry(&mut self, species: &PgSpecies) -> Result<BestiaryEntry, DieselError> {
        if let Some(cache) = self.cache.get(&species.to_string()) {
            return Ok(cache);
        }
        let entry = Bestiary::read_by_species(&mut self.connection, &species)?;
        self.cache.set(species.to_string(), entry.clone(), self.ttl);
        return Ok(entry);
    }

    pub fn create(
        &mut self,
        species: &PgSpecies,
        world: WorldCoord,
    ) -> Result<Monster, DieselError> {
        let location = Location::create(
            &mut self.connection,
            &CreateLocation {
                // Impl. random (require to load each map collider grid to avoid spawning on collider)
                map: MapCoord::random().into(),
                world: world.into(),
                destination: None,
            },
        )?;

        let entity = Entity::create(
            &mut self.connection,
            &CreateEntity {
                location_id: location.id,
            },
        )?;

        let bestiary = self.get_bestiary_entry(species)?;

        Monster::create(
            &mut self.connection,
            &CreateMonster {
                bestiary_id: bestiary.id,
                entity_id: entity.id,
            },
        )
    }
}
