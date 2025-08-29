use diesel::{result::Error, PgConnection};
use diesel_geometry::data_types::PgPoint;

use crate::database::{
    db::Database,
    model::{
        account::Account,
        character::{Character, Classes, CreateCharacter},
        entity::{CreateEntity, Entity},
        location::{CreateLocation, Location},
    },
};

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

    pub fn get_all_player_on_world(&mut self, world: PgPoint) -> Result<Vec<String>, Error> {
        Character::read_all_by_world(&mut self.connection, world)
    }

    pub fn get_character_info(&mut self) -> Result<CharacterInfo, Error> {
        let chars = &self.get_all()?[0];
        let location = Location::read(&mut self.connection, &chars.entity_id)?;

        Ok(CharacterInfo {
            uuid: format!("{}.{}.{}", chars.account_id, chars.id, chars.entity_id),
            eid: chars.entity_id,
            class: chars.class.clone(),
            map: location.map,
            world: location.world,
        })
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
