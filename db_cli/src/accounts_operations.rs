use common::database::schema::accounts;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;

use crate::args::{AccountCommand, AccountSubcommand, CreateAccount, DeleteAccount, UpdateAccount};

use common::authenticator::Authenticator;
use common::database::db::Database;
use common::database::models::{Account, NewAccount};

pub fn handle_account(account: AccountCommand) {
    match account.command {
        AccountSubcommand::Create(account) => create_account(account),
        AccountSubcommand::Show => show_account(),
        AccountSubcommand::Delete(account) => delete_account(account),
        AccountSubcommand::Update(account) => update_account(account),
    }
}

pub fn create_account(create_account: CreateAccount) {
    use common::database::schema::accounts::dsl::*;

    let connection = &mut Database::new().establish_connection();
    let mut new_account: NewAccount = create_account.into();
    new_account.password = Authenticator::default().hash_password(new_account.password);

    match diesel::insert_into(accounts)
        .values(&new_account)
        .execute(connection)
    {
        Ok(_) => println!("Account created."),
        Err(e) => match e {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                println!(
                    "Error login `{}` already exist. Please retry with another login.",
                    new_account.login
                )
            }
            _ => println!("Error while creating account : {:?}", e),
        },
    }
}

pub fn update_account(update_account: UpdateAccount) {
    use common::database::schema::accounts::dsl::*;

    let connection = &mut Database::new().establish_connection();

    // ask user old password to authenticate him
    let current_password = rpassword::prompt_password("Old password: ")
        .expect("An error occured user prompt for current password.");

    let mut auth_user = Authenticator::new(update_account.login.clone());
    // Try to authenticate user
    if !auth_user.authenticate(&current_password) {
        println!("Invalid Login or Password.");
        std::process::exit(1)
    }

    // Ask user to confirm new password
    let new_password = rpassword::prompt_password("Confirm new password: ").unwrap();
    if update_account.password.ne(&new_password) {
        println!("New passwords didn't match please retry.");
        std::process::exit(1)
    }

    // Hash and update user Account in DB
    let new_hash = auth_user.hash_password(new_password);
    diesel::update(accounts)
        .filter(login.eq(update_account.login))
        .set(password.eq(new_hash))
        .execute(connection)
        .expect("Error while updating account.");

    if !auth_user.logout(None) {
        println!("Logout failed.");
    }
}

pub fn delete_account(delete_account: DeleteAccount) {
    let connection = &mut Database::new().establish_connection();
    let account_to_delete = accounts::table.filter(accounts::login.eq(delete_account.login));

    diesel::delete(account_to_delete)
        .execute(connection)
        .expect("Error deleting account.");
}

pub fn show_account() {
    use common::database::schema::accounts::dsl::*;

    let connection = &mut Database::new().establish_connection();
    let results = accounts
        .load::<Account>(connection)
        .expect("Error loading accounts");

    for account in results {
        println!("{:?}", account)
    }
}
