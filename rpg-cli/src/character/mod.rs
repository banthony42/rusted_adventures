use clap::{Args, Subcommand, ValueEnum};

pub mod operations;

#[derive(Debug, Args)]
pub struct CharacterCommand {
    #[clap(subcommand)]
    pub command: CharacterSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CharacterSubcommand {
    /// Create an new character
    Create(CreateCharacterCmd),

    /// Delete a character
    Delete(DeleteCharacterCmd),

    /// Show all characters of a given account (by login)
    Show,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum CharacterClass {
    Warrior,
    Mage,
}

#[derive(Debug, Args)]
pub struct CreateCharacterCmd {
    /// The account login, this character is created for
    pub login: String,

    /// The name of the character
    pub name: String,

    /// The class of the character
    pub class: CharacterClass,
}

#[derive(Debug, Args)]
pub struct DeleteCharacterCmd {
    /// The id of the character to delete
    pub id: i32,
}
