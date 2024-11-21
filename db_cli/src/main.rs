#[macro_use]
mod account;
mod args;
mod grpc;

use args::Commands;
use args::DBCliArgs;
use clap::Parser;

fn main() {
    let args = DBCliArgs::parse();

    match args.commands {
        Commands::Account(account) => account::operations::handle_account(account),
        Commands::Grpc(grpc) => grpc::operations::handle_grpc(grpc),
    }
}
