use common::database::model::account::{Account, CreateAccount, UpdateAccount};
use diesel::result::DatabaseErrorKind;

use common::authenticator::Authenticator;
use common::database::db::Database;

use super::{
    AccountCommand, AccountSubcommand, CreateAccountCmd, DeleteAccountCmd, UpdateAccountCmd,
};

pub fn handle_account(account: AccountCommand) {
    match account.command {
        AccountSubcommand::Create(account) => create_account(account),
        AccountSubcommand::Update(account) => update_account(account),
        AccountSubcommand::Delete(account) => delete_account(account),
        AccountSubcommand::Show => show_account(),
    }
}

fn create_account(create_account: CreateAccountCmd) {
    let connection = &mut Database::new().establish_connection();

    let mut new_account: CreateAccount = create_account.into();

    // Ask user to confirm new password
    let new_password = rpassword::prompt_password("Confirm new password: ").unwrap();
    if new_account.password.ne(&new_password) {
        println!("Passwords didn't match please retry.");
        std::process::exit(1)
    }

    new_account.password = Authenticator::default().hash_password(new_account.password);

    match Account::create(connection, &new_account) {
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

fn update_account(update_account: UpdateAccountCmd) {
    let connection = &mut Database::new().establish_connection();

    // ask user old password to authenticate him
    let current_password = rpassword::prompt_password("Old password: ")
        .expect("An error occured user prompt for current password.");

    // Try to authenticate user
    let mut auth_user = Authenticator::new(&update_account.login);
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

    // Create the update item with the new Hash for the user
    let update_item = UpdateAccount {
        login: Some(update_account.login.clone()),
        password: Some(auth_user.hash_password(new_password)),
        session_token: None,
    };

    match Account::update(connection, &update_account.login, &update_item) {
        Ok(_) => {}
        Err(e) => println!("Error updating accounts: {:?}", e),
    }

    if let Err(e) = auth_user.logout(None) {
        println!("Logout failed: {}", e);
    };
}

fn delete_account(delete_account: DeleteAccountCmd) {
    let connection = &mut Database::new().establish_connection();
    match Account::delete(connection, &delete_account.login) {
        Ok(_) => {}
        Err(e) => println!("Error deleting accounts: {:?}", e),
    };
}

fn show_account() {
    let connection = &mut Database::new().establish_connection();
    match Account::read_all(connection) {
        Ok(accounts) => accounts
            .iter()
            .map(|account| println!("{:?}", account))
            .collect(),
        Err(e) => println!("Error reading all accounts: {:?}", e),
    };
}
