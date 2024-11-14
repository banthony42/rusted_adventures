#[macro_use]
mod accounts_operations;
mod args;

use args::TestCommand;
use clap::Parser;

use accounts_operations::handle_account;
use args::Commands;
use args::DBCliArgs;
use common::database::db::Database;
use common::database::models::Player;
use diesel::prelude::*;

fn diesel_with_postgresql_custom_type_playground() {
    use common::database::schema::player::dsl::*;
    let connection = &mut Database::new().establish_connection();

    let all_loc = player.load::<Player>(connection);
}

fn handle_playground(cmd: TestCommand) {
    match cmd.command {
        args::TestSubcommand::CustomTypes => diesel_with_postgresql_custom_type_playground(),
    }
}

fn main() {
    let args = DBCliArgs::parse();

    match args.commands {
        Commands::Account(account) => handle_account(account),
        Commands::Test(test_command) => handle_playground(test_command),
    }
}
