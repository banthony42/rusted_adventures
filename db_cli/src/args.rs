use crate::character::CharacterCommand;

use super::account::AccountCommand;
use super::grpc::GrpcCommand;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct DBCliArgs {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create, update, delete or show accounts
    Account(AccountCommand),

    /// Test GRPc services
    Grpc(GrpcCommand),

    /// Create, delete or show characters
    Character(CharacterCommand),
}
