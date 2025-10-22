use std::fmt::Display;

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
    grpc_codegen::Location as RpcLocation,
    record::Record,
    rpc_extentions::RpcLocationExtension,
    CellCoord, MapCoord,
};

#[derive(Debug)]
pub enum MonsterHandlerError {
    DatabaseError(DieselError),
    GRPCLocationIntoUpdateLocation,
}

impl Display for MonsterHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonsterHandlerError::DatabaseError(err) => {
                write!(f, "DatabaseError: {}", err.to_string())
            }
            MonsterHandlerError::GRPCLocationIntoUpdateLocation => {
                write!(f, "Error while trying to transform gRPC Location into database model UpdateLocation.")
            }
        }
    }
}

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

    fn get_bestiary_entry(
        &mut self,
        species: &PgSpecies,
    ) -> Result<BestiaryEntry, MonsterHandlerError> {
        if let Some(cache) = self.cache.get(&species.to_string()) {
            return Ok(cache);
        }
        let entry = Bestiary::read_by_species(&mut self.connection, &species)
            .map_err(|e| MonsterHandlerError::DatabaseError(e))?;
        self.cache.set(species.to_string(), entry.clone(), self.ttl);
        return Ok(entry);
    }

    pub fn create(
        &mut self,
        species: &PgSpecies,
        map: MapCoord,
        cell: CellCoord,
    ) -> Result<Monster, MonsterHandlerError> {
        let location = Location::create(
            &mut self.connection,
            &CreateLocation {
                cell: cell.into(),
                map: map.into(),
                destination: None,
            },
        )
        .map_err(|e| MonsterHandlerError::DatabaseError(e))?;

        let entity = Entity::create(
            &mut self.connection,
            &CreateEntity {
                location_id: location.id,
            },
        )
        .map_err(|e| MonsterHandlerError::DatabaseError(e))?;

        let bestiary = self.get_bestiary_entry(species)?;

        Monster::create(
            &mut self.connection,
            &CreateMonster {
                bestiary_id: bestiary.id,
                entity_id: entity.id,
            },
        )
        .map_err(|e| MonsterHandlerError::DatabaseError(e))
    }

    /// Update the Monster location with `new_loc`
    /// If `new_loc` match the `destination` then `destination` is reset to None
    pub fn update_location(
        &mut self,
        entity_id: i32,
        new_loc: RpcLocation,
    ) -> Result<(), MonsterHandlerError> {
        let update_location = new_loc
            .into_update_location()
            .ok_or_else(|| MonsterHandlerError::GRPCLocationIntoUpdateLocation)?;

        Location::update_by_entity_id(&mut self.connection, &entity_id, update_location)
            .map_err(|e| MonsterHandlerError::DatabaseError(e))?;

        Ok(())
    }
}
