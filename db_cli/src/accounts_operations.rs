use diesel::prelude::*;

use crate::args::{AccountCommand, AccountSubcommand, CreateAccount, UpdateAccount, DeleteAccount};
use crate::models::{Account, NewAccount};
use crate::db::establish_connection;

pub fn handle_account(account: AccountCommand) {
    match account.command {
        AccountSubcommand::Create(account) => create_account(account),
        AccountSubcommand::Show => show_account(),
        AccountSubcommand::Delete(account) => delete_account(account),
        AccountSubcommand::Update(account) => update_account(account),
    }
}

pub fn create_account(new_account: CreateAccount) {
    println!("Creating account: login:{:?} password:{:?}", new_account.login, new_account.password);
    use crate::schema::accounts::dsl::*;

    let connection = &mut establish_connection();
    let new_account = NewAccount {
        login: new_account.login,
        password: new_account.password
    };

    diesel::insert_into(accounts)
        .values(&new_account)
        .execute(connection)
        .expect("Error creating new account."); // TODO: properly warn and do nothing when user already exist
    println!("==> TODO: properly warn and do nothing when user already exist");
}

pub fn update_account(update_account: UpdateAccount) {
    todo!()
}

pub fn delete_account(delete_account: DeleteAccount) {
    todo!()
}

pub fn show_account() {
    println!("Sowing accounts");
    use crate::schema::accounts::dsl::*;

    let connection = &mut establish_connection();
    let results = accounts
        .load::<Account>(connection)
        .expect("Error loading accounts");

    for account in results {
        println!("==> {:?}", account)
    }
}