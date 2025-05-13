use common::database::{
    db::Database,
    model::{
        account::Account,
        character::{Character, Classes, CreateCharacter},
        entity::{CreateEntity, Entity},
        location::{Coord, CreateLocation, Location},
    },
};

use super::{
    CharacterClass, CharacterCommand, CharacterSubcommand, CreateCharacterCmd, DeleteCharacterCmd,
};

pub fn handle_character(character: CharacterCommand) {
    match character.command {
        CharacterSubcommand::Create(character) => create_character(character),
        CharacterSubcommand::Delete(character) => delete_character(character),
        CharacterSubcommand::Show => show_characters(),
    }
}

fn create_character(character: CreateCharacterCmd) {
    let connection = &mut Database::new().establish_connection();

    let entity_to_create = CreateEntity {
        name: character.name,
    };

    if let Ok(new_entity) = Entity::create(connection, &entity_to_create) {
        println!("Entity created.");

        // Create and bind character as foreign key
        let user_account = Account::read(connection, &character.login)
            .expect("Error while searching after user account.");
        let new_item = match character.class {
            CharacterClass::Warrior => CreateCharacter {
                account_id: user_account.id,
                entity_id: new_entity.id,
                class: Classes::Warrior,
            },
            CharacterClass::Witcher => CreateCharacter {
                account_id: user_account.id,
                entity_id: new_entity.id,
                class: Classes::Witcher,
            },
        };
        match Character::create(connection, &new_item) {
            Ok(_) => println!("Character created."),
            Err(e) => println!("Error while creating character : {:?}", e),
        }

        // Create and bind location as foreign key
        let spawn = Coord { x: 400.0, y: 400.0 };
        let new_loc = CreateLocation {
            entity_id: new_entity.id,
            world: spawn,
        };
        match Location::create(connection, &new_loc) {
            Ok(_) => println!("Location created."),
            Err(e) => println!("Error while creating location : {:?}", e),
        }
    } else {
        println!("Error while creating entity.");
    }
}

fn delete_character(character: DeleteCharacterCmd) {
    let connection = &mut Database::new().establish_connection();
    match Character::delete(connection, &character.id) {
        Ok(_) => println!("Character deleted."),
        Err(e) => println!("Error while deleting character : {:?}", e),
    }
}

fn show_characters() {
    let connection = &mut Database::new().establish_connection();
    match Character::read_all(connection) {
        Ok(characters) => characters
            .iter()
            .map(|character| println!("{:?}", character))
            .collect(),
        Err(e) => println!("Error reading all accounts: {:?}", e),
    };
}
