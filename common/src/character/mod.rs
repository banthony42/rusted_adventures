use std::{error::Error, fmt::Display};

use diesel::{result::Error as DieselError, PgConnection};
use diesel_geometry::data_types::PgPoint;

use crate::{
    database::{
        db::Database,
        model::{
            account::Account,
            character::{character::CharacterInfo, Character, CreateCharacter, PgClasses},
            entity::{CreateEntity, Entity},
            location::{CreateLocation, Location, UpdateLocationDestination},
            monster::Monster,
            EntityIdentifiable,
        },
    },
    grpc_codegen::{Coord, Location as RpcLocation},
    rpc_extentions::RpcLocationExtension,
    CellCoord,
};

use crate::grpc_codegen::Entity as RpcEntity;

impl Into<Coord> for PgPoint {
    fn into(self) -> Coord {
        Coord {
            x: self.0 as i64,
            y: self.1 as i64,
        }
    }
}

impl From<Coord> for PgPoint {
    fn from(value: Coord) -> Self {
        PgPoint(value.x as f64, value.y as f64)
    }
}

#[derive(Debug)]
pub enum CharacterHandlerError {
    DatabaseError(DieselError),
    NoCharacterForAccount,
    GRPCLocationIntoUpdateLocation,
}

impl Display for CharacterHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharacterHandlerError::NoCharacterForAccount => {
                write!(f, "This account doesn't have any character.")
            }
            CharacterHandlerError::DatabaseError(err) => {
                write!(f, "DatabaseError: {}", err.to_string())
            }
            CharacterHandlerError::GRPCLocationIntoUpdateLocation => {
                write!(f, "Error while trying to transform gRPC Location into database model UpdateLocation.")
            }
        }
    }
}

impl Error for CharacterHandlerError {}

type RpcEntityWithDestination = (RpcEntity, Option<Coord>);

pub struct CharacterHandler {
    character: Character,
    pub connection: PgConnection,
}

impl CharacterHandler {
    /// Load the character of the account, return an handler for it.
    /// (For now an account can have only one character)
    pub fn new(login: &String) -> Result<Self, CharacterHandlerError> {
        let mut connection = Database::new().establish_connection();
        let character = Character::read_by_account(&mut connection, &login)
            .map_err(|e| CharacterHandlerError::DatabaseError(e))?
            .get(0) // For now consider account play with only one character
            .cloned()
            .ok_or(CharacterHandlerError::NoCharacterForAccount)?;

        Ok(Self {
            character,
            connection,
        })
    }

    fn map_spawn() -> PgPoint {
        PgPoint(0.0, 0.0)
    }

    fn cell_spawn() -> PgPoint {
        CellCoord::spawn().into()
    }

    /// Create a character for the account, return an handler for it.
    pub fn create(
        account_login: &String,
        name: &String,
        class: PgClasses,
    ) -> Result<Self, CharacterHandlerError> {
        let mut connection = Database::new().establish_connection();

        let location = Location::create(
            &mut connection,
            &CreateLocation {
                cell: Self::cell_spawn(),
                map: Self::map_spawn(),
                destination: None,
            },
        )
        .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        let entity = Entity::create(
            &mut connection,
            &CreateEntity {
                location_id: location.id,
            },
        )
        .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        // Create and bind character as foreign key
        let user_account = Account::read(&mut connection, account_login)
            .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        let character = Character::create(
            &mut connection,
            &CreateCharacter {
                account_id: user_account.id,
                entity_id: entity.id,
                name: name.to_owned(),
                class,
            },
        )
        .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        Ok(Self {
            character,
            connection,
        })
    }

    pub fn identifier(&self) -> String {
        self.character.identifier()
    }

    pub fn as_rpc_entity(&mut self) -> Result<RpcEntity, CharacterHandlerError> {
        Ok(
            CharacterInfo::from_character(&mut self.connection, &self.character)
                .map_err(|e| CharacterHandlerError::DatabaseError(e))?
                .into(),
        )
    }

    pub fn players_on_same_map(&mut self) -> Result<Vec<String>, CharacterHandlerError> {
        // TODO: issue: groupby clause doesn't work :
        // [941] ERROR:  could not identify an equality operator for type point at character 152
        // [941] STATEMENT:  SELECT "entities"."name" FROM ("locations" INNER JOIN "entities" ON ("locations"."entity_id" = "entities"."id")) WHERE ("entities"."id" = $1) GROUP BY "locations"."map", "entities"."name"

        let location = Location::read(&mut self.connection, &self.character.entity_id)
            .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        Character::read_names_by_map_exclude(
            &mut self.connection,
            location.map,
            &self.character.name,
        )
        .map_err(|e| CharacterHandlerError::DatabaseError(e))
    }

    pub fn entities_on_map(
        &mut self,
    ) -> Result<Vec<RpcEntityWithDestination>, CharacterHandlerError> {
        let location = Location::read(&mut self.connection, &self.character.entity_id)
            .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        let mut players: Vec<RpcEntityWithDestination> =
            Character::read_by_map(&mut self.connection, location.map)
                .map_err(|e| CharacterHandlerError::DatabaseError(e))?
                .into_iter()
                .filter(|e| self.character.name.ne(&e.name))
                .map(|e| {
                    let destination: Option<Coord> = e.destination.map(|d| d.into());
                    (e.into(), destination)
                })
                .collect();

        let mut monsters: Vec<RpcEntityWithDestination> =
            Monster::read_all_by_map(&mut self.connection, location.map)
                .map_err(|e| CharacterHandlerError::DatabaseError(e))?
                .into_iter()
                .map(|e| {
                    let destination: Option<Coord> = e.destination.map(|d| d.into());
                    (e.into(), destination)
                })
                .collect();

        let mut entities: Vec<RpcEntityWithDestination> = Vec::new();
        entities.append(&mut players);
        entities.append(&mut monsters);
        Ok(entities)
    }

    /// Update the Character location with `new_loc`
    /// If `new_loc` match the `destination` then `destination` is reset to None
    pub fn update_location(&mut self, new_loc: RpcLocation) -> Result<(), CharacterHandlerError> {
        let update_location = new_loc
            .into_update_location()
            .ok_or_else(|| CharacterHandlerError::GRPCLocationIntoUpdateLocation)?;

        Location::update_by_entity_id(
            &mut self.connection,
            &self.character.entity_id,
            update_location,
        )
        .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        Ok(())
    }

    pub fn update_destination(
        &mut self,
        new_loc: UpdateLocationDestination,
    ) -> Result<(), CharacterHandlerError> {
        Location::update_destination_by_entity_id(
            &mut self.connection,
            &self.character.entity_id,
            new_loc,
        )
        .map_err(|e| CharacterHandlerError::DatabaseError(e))?;

        Ok(())
    }
}
