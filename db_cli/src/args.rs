use crate::account::AccountCommand;
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
}
