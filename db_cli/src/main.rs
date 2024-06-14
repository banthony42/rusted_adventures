#[macro_use]
mod schema;
mod db;
mod models;
mod accounts_operations;
mod args;

use clap::Parser;

use args::DBCliArgs;
use args::Commands;
use accounts_operations::handle_account;

fn main() {
    let args = DBCliArgs::parse();

    match args.commands {
        Commands::Account(account) => handle_account(account),
    }
}
