use diesel::prelude::*;

use crate::models::{Account, NewAccount};
use crate::db::establish_connection;

pub fn create_account(new_login: String, new_password: String) {
    println!("Creating account: login:{:?} password:{:?}", new_login, new_password);
    use crate::schema::accounts::dsl::*;

    let connection = &mut establish_connection();
    let new_account = NewAccount {
        login: new_login,
        password: new_password
    };

    diesel::insert_into(accounts)
        .values(&new_account)
        .execute(connection)
        .expect("Error creating new account."); // TODO: properly warn and do nothing when user already exist
    println!("==> TODO: properly warn and do nothing when user already exist");
}

pub fn update_account() {
    todo!()
}

pub fn delete_account() {
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