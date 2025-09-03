use diesel::{result::Error, PgConnection};
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
    grpc_codegen::{Bestiary, Coord, Location as RpcLocation},
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

pub struct CharacterInfo {
    pub uuid: String,
    pub eid: i32,
    pub class: Classes,
    pub map: PgPoint,
    pub world: PgPoint,
}
pub struct CharacterAccountHandler {
    login: String,
    pub connection: PgConnection,
}

impl CharacterAccountHandler {
    pub fn new(login: &String) -> Self {
        Self {
            login: login.clone(),
            connection: Database::new().establish_connection(),
        }
    }

    fn get_map_spawn() -> PgPoint {
        PgPoint(5.0, 5.0)
    }

    fn get_world_spawn() -> PgPoint {
        PgPoint(0.0, 0.0)
    }

    pub fn get_all(&mut self) -> Result<Vec<Character>, Error> {
        Character::read_all_by_account_login(&mut self.connection, &self.login)
    }

    pub fn get_character_info(&mut self) -> Result<CharacterInfo, Error> {
        let chars = &self.get_all()?[0];
        let location = Location::read(&mut self.connection, &chars.entity_id)?;

        Ok(CharacterInfo {
            uuid: format!("{}.{}", chars.account_id, chars.entity_id),
            eid: chars.entity_id,
            class: chars.class.clone(),
            map: location.map,
            world: location.world,
        })
    }

    pub fn get_all_player_on_world(&mut self, world: PgPoint) -> Result<Vec<String>, Error> {
        Character::read_all_by_world(&mut self.connection, world)
    }

    pub fn get_players_on_same_world(&mut self) -> Result<Vec<String>, Error> {
        let info = self.get_character_info()?;
        self.get_all_player_on_world(info.world)
    }

    pub fn get_entities_on_same_world(&mut self) -> Result<Vec<(RpcEntity, Option<Coord>)>, Error> {
        let info = self.get_character_info()?;

        let players =
            Character::get_all_players_by_world(&mut self.connection, &self.login, info.world)?;
        let mut players_as_rpcentities: Vec<(RpcEntity, Option<Coord>)> = players
            .iter()
            .map(|ent| {
                (
                    RpcEntity {
                        uuid: ent.0.clone(),
                        name: ent.1.clone(),
                        family: Bestiary::Human as i32,
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
                        family: ent.2.clone() as i32,
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

        if let Ok(info) = self.get_character_info() {
            let ul_result = Location::update(
                &mut self.connection,
                &info.eid,
                &UpdateLocation {
                    world: new_w,
                    map: new_m,
                },
            );

            if let Ok(ul) = ul_result {
                println!(
                    "=====> update_location: comparison: {:?} - {:?}",
                    ul.destination, ul.map
                );
                if ul.destination == Some(ul.map) {
                    let _udresult = Location::update_destination(
                        &mut self.connection,
                        &1,
                        UpdateLocationDestination { destination: None },
                    );
                }
            }
        }
    }

    pub fn create(&mut self, name: &String, class: Classes) -> Result<(), Error> {
        // For now we authorize only one character per account

        let entity_to_create = CreateEntity { name: name.clone() };
        let new_entity = Entity::create(&mut self.connection, &entity_to_create)?;

        // Create and bind character as foreign key
        let user_account = Account::read(&mut self.connection, &self.login)?;
        let new_item = CreateCharacter {
            account_id: user_account.id,
            entity_id: new_entity.id,
            class,
        };
        Character::create(&mut self.connection, &new_item)?;

        let new_loc = CreateLocation {
            entity_id: new_entity.id,
            map: Self::get_map_spawn(),
            world: Self::get_world_spawn(),
            destination: None,
        };

        Location::create(&mut self.connection, &new_loc)?;
        Ok(())
    }
}
