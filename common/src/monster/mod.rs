use diesel::{result::Error as DieselError, PgConnection};
use diesel_geometry::data_types::PgPoint;

use crate::{
    database::{
        db::Database,
        model::{
            bestiary::{bestiary::Bestiary, BestiaryEntry, PgSpecies},
            entity::{CreateEntity, Entity},
            location::{CreateLocation, Location, UpdateLocation},
            monster::{CreateMonster, Monster},
        },
    },
    grpc_codegen::Location as RpcLocation,
    record::Record,
    CellCoord, WorldCoord,
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
                cell: CellCoord::random().into(),
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

    /// Update the Monster location with `new_loc`
    /// If `new_loc` match the `destination` then `destination` is reset to None
    pub fn update_location(
        &mut self,
        entity_id: i32,
        new_loc: RpcLocation,
    ) -> Result<(), DieselError> {
        let new_w = PgPoint(
            new_loc.world.unwrap().x as f64,
            new_loc.world.unwrap().y as f64,
        );
        let new_m = PgPoint(
            new_loc.cell.unwrap().x as f64,
            new_loc.cell.unwrap().y as f64,
        );

        Location::update_by_entity_id(
            &mut self.connection,
            &entity_id,
            UpdateLocation {
                world: new_w,
                cell: new_m,
            },
        )?;
        Ok(())
    }
}
