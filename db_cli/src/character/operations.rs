use common::{
    character::{CharacterHandler, CharacterHandlerError},
    database::{
        db::Database,
        model::character::{Character, Classes},
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
    let pg_class = match character.class {
        CharacterClass::Warrior => Classes::Warrior,
        CharacterClass::Mage => Classes::Mage,
    };

    if let Err(e) = CharacterHandler::create(&character.login, &character.name, pg_class) {
        println!("Error while creating character: {:?}", e);
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
