use std::{char, error::Error, fmt::Display};

use diesel::{result::Error as DieselError, PgConnection};
use diesel_geometry::data_types::PgPoint;

use crate::{
    database::{
        db::Database,
        model::{
            account::Account,
            character::{Character, Classes, CreateCharacter},
            entity::{CreateEntity, Entity},
            location::{CreateLocation, Location, UpdateLocation, UpdateLocationDestination},
        },
    },
    grpc_codegen::{Coord, Location as RpcLocation},
    MapCoord, WorldCoord,
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

impl Into<RpcEntity> for CharacterInfo {
    fn into(self) -> RpcEntity {
        RpcEntity {
            uuid: self.uuid,
            name: self.name,
            family: Some(self.class.into()),
            location: Some(RpcLocation {
                world: Some(self.world.into()),
                map: Some(self.map.into()),
            }),
        }
    }
}

#[derive(Debug)]
pub enum CharacterHandlerError {
    DatabaseError(String),
    NoCharacterForAccount,
}

impl Display for CharacterHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CharacterHandlerError::NoCharacterForAccount => {
                write!(f, "This account doesn't have any character.")
            }
            CharacterHandlerError::DatabaseError(_) => {
                write!(f, "DatabaseError")
            }
        }
    }
}

impl Error for CharacterHandlerError {}

#[derive(Clone)]
pub struct CharacterInfo {
    pub uuid: String,
    pub name: String,
    pub eid: i32,
    pub class: Classes,
    pub map: PgPoint,
    pub world: PgPoint,
}

pub struct CharacterHandler {
    login: String,
    character: Character,
    character_name: String,
    pub connection: PgConnection,
}

impl CharacterHandler {
    pub fn new(login: &String) -> Result<Self, CharacterHandlerError> {
        let mut connection = Database::new().establish_connection();
        let (character, character_name) = Character::read_by_account_login(&mut connection, &login)
            .map_err(|e| CharacterHandlerError::DatabaseError(e.to_string()))?
            .ok_or(CharacterHandlerError::NoCharacterForAccount)?;

        Ok(Self {
            character,
            character_name,
            login: login.clone(),
            connection,
        })
    }

    pub fn entity_uuid(&self) -> String {
        format!("{}.{}", self.character.account_id, self.character.entity_id)
    }

    pub fn character_info(&mut self) -> Result<CharacterInfo, DieselError> {
        let location = Location::read(&mut self.connection, &self.character.entity_id)?;

        Ok(CharacterInfo {
            uuid: self.entity_uuid(),
            name: self.character_name.clone(),
            eid: self.character.entity_id,
            class: self.character.class.clone(),
            map: location.map,
            world: location.world,
        })
    }

    pub fn players_on_same_world(&mut self) -> Result<Vec<String>, DieselError> {
        // TODO: issue: groupby clause doesn't work :
        // [941] ERROR:  could not identify an equality operator for type point at character 152
        // [941] STATEMENT:  SELECT "entities"."name" FROM ("locations" INNER JOIN "entities" ON ("locations"."entity_id" = "entities"."id")) WHERE ("entities"."id" = $1) GROUP BY "locations"."world", "entities"."name"
        // Character::read_all_on_same_world(&mut self.connection, self.character.entity_id)

        // TODO: issue 1: the list contain the sender
        // TODO: issue 2: the list contain monsters
        // TODO: issue 3: the list contain several times the same entities name
        // TODO: issue 4: (client side) the client don't check the world coord before accepting the spawn entities
        let location = Location::read(&mut self.connection, &self.character.entity_id)?;
        Character::read_all_by_world(&mut self.connection, location.world)
    }

    pub fn entities_on_same_world(
        &mut self,
    ) -> Result<Vec<(RpcEntity, Option<Coord>)>, DieselError> {
        let info = self.character_info()?;

        let players =
            Character::get_all_players_by_world(&mut self.connection, &self.login, info.world)?;
        let mut players_as_rpcentities: Vec<(RpcEntity, Option<Coord>)> = players
            .iter()
            .map(|ent| {
                (
                    RpcEntity {
                        uuid: ent.0.clone(),
                        name: ent.1.clone(),
                        family: Some(ent.2.clone().into()),
                        location: Some(RpcLocation {
                            world: Some(info.world.into()),
                            map: Some(ent.3.into()),
                        }),
                    },
                    ent.4.map(|d| d.into()),
                )
            })
            .collect();

        let monsters = Character::get_all_monsters_by_world(&mut self.connection, info.world)?;
        let mut monsters_as_rpcentities: Vec<(RpcEntity, Option<Coord>)> = monsters
            .iter()
            .map(|ent| {
                (
                    RpcEntity {
                        uuid: format!("{}.{}", ent.1, ent.0),
                        name: ent.1.clone(),
                        family: Some(ent.2.clone().into()),
                        location: Some(RpcLocation {
                            world: Some(info.world.into()),
                            map: Some(ent.3.into()),
                        }),
                    },
                    ent.4.map(|d| d.into()),
                )
            })
            .collect();
        players_as_rpcentities.append(&mut monsters_as_rpcentities);
        Ok(players_as_rpcentities)
    }

    /// Update the entity location with `new_loc`
    /// If `new_loc` match the `destination` then `destination` is reset to None
    pub fn update_location(&mut self, new_loc: RpcLocation) {
        let new_w = PgPoint(
            new_loc.world.unwrap().x as f64,
            new_loc.world.unwrap().y as f64,
        );
        let new_m = PgPoint(new_loc.map.unwrap().x as f64, new_loc.map.unwrap().y as f64);

        if let Ok(info) = self.character_info() {
            let ul_result = Location::update(
                &mut self.connection,
                &info.eid,
                &UpdateLocation {
                    world: new_w,
                    map: new_m,
                },
            );

            // Could be a pgsql trigger maybe :
            // When map position == destination
            // Player has reached its destination, so reset destination to None
            if let Ok(ul) = ul_result {
                if ul.destination == Some(ul.map) {
                    let _udresult = Location::update_destination(
                        &mut self.connection,
                        &info.eid,
                        UpdateLocationDestination { destination: None },
                    );
                }
            }
        }
    }

    pub fn update_destination(&mut self, new_loc: UpdateLocationDestination) {
        let result =
            Location::update_destination(&mut self.connection, &self.character.entity_id, new_loc);
        if let Err(err) = result {
            println!("Server: update_destination: {:?}", err);
        }
    }

    pub fn create_with_random_class(
        account_login: &String,
        name: &String,
    ) -> Result<(), DieselError> {
        Self::create(account_login, name, rand::random::<Classes>())
    }

    pub fn create(
        account_login: &String,
        name: &String,
        class: Classes,
    ) -> Result<(), DieselError> {
        let mut connection = Database::new().establish_connection();

        // For now we authorize only one character per account
        let entity_to_create = CreateEntity { name: name.clone() };
        let new_entity = Entity::create(&mut connection, &entity_to_create)?;

        // Create and bind character as foreign key
        let user_account = Account::read(&mut connection, account_login)?;
        let new_item = CreateCharacter {
            account_id: user_account.id,
            entity_id: new_entity.id,
            class,
        };
        Character::create(&mut connection, &new_item)?;

        let new_loc = CreateLocation {
            entity_id: new_entity.id,
            map: MapCoord::spawn().into(),
            world: WorldCoord::spawn().into(),
            destination: None,
        };

        Location::create(&mut connection, &new_loc)?;
        Ok(())
    }
}
