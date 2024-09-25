#[macro_use]
mod accounts_operations;
mod args;

use clap::Parser;

use accounts_operations::handle_account;
use args::Commands;
use args::DBCliArgs;

fn main() {
    let args = DBCliArgs::parse();

    match args.commands {
        Commands::Account(account) => handle_account(account),
    }
}
