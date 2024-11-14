#[macro_use]
mod account;
mod args;

use args::Commands;
use args::DBCliArgs;
use clap::Parser;

fn main() {
    let args = DBCliArgs::parse();

    match args.commands {
        Commands::Account(account) => account::operations::handle_account(account),
    }
}
