use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;

use crate::args::{AccountCommand, AccountSubcommand, CreateAccount, DeleteAccount, UpdateAccount};
use crate::db::establish_connection;
use crate::models::{Account, NewAccount};
use crate::schema::accounts;

use argon2::*;
use password_hash::rand_core::OsRng;
use password_hash::SaltString;

pub fn handle_account(account: AccountCommand) {
    match account.command {
        AccountSubcommand::Create(account) => create_account(account),
        AccountSubcommand::Show => show_account(),
        AccountSubcommand::Delete(account) => delete_account(account),
        AccountSubcommand::Update(account) => update_account(account),
    }
}

fn hash_password(password: String) -> String {
    let error_handler = |e: String| println!("Error while hashing password: {:?}", e);

    let argon2_owasp_params = match Params::new(19 * 1024, 2, 1, Some(Params::DEFAULT_OUTPUT_LEN)) {
        Ok(p) => p,
        Err(e) => {
            error_handler(e.to_string());
            std::process::exit(1);
        }
    };

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::default(), argon2_owasp_params);
    let salt = SaltString::generate(&mut OsRng);
    let hash_pasword = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error_handler(e.to_string());
            std::process::exit(1)
        }
    };
    return hash_pasword;
}

fn authenticate_user(login: &String, password: &String) -> bool {
    let connection = &mut establish_connection();
    // Get the user account in DB
    let account_to_auth = accounts::table
        .find(login)
        .select(Account::as_select())
        .first(connection)
        .expect("Invalid Login or Password.");

    // Import user hashed password and verify it
    let parsed_hash = PasswordHash::new(&account_to_auth.password)
        .expect("Error while importing user password hash.");
    // TODO: Do not use Argon2::default, use explicit parameters see hash_password function
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn create_account(create_account: CreateAccount) {
    use crate::schema::accounts::dsl::*;

    let connection = &mut establish_connection();
    let mut new_account: NewAccount = create_account.into();
    new_account.password = hash_password(new_account.password);

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
    use crate::schema::accounts::dsl::*;

    let connection = &mut establish_connection();

    // ask user old password to authenticate him
    let current_password = rpassword::prompt_password("Old password: ")
        .expect("An error occured user prompt for current password.");

    // Try to authenticate user
    if !authenticate_user(&update_account.login, &current_password) {
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
    let new_hash = hash_password(new_password);
    diesel::update(accounts)
        .filter(login.eq(update_account.login))
        .set(password.eq(new_hash))
        .execute(connection)
        .expect("Error while updating account.");
}

pub fn delete_account(delete_account: DeleteAccount) {
    let connection = &mut establish_connection();
    let account_to_delete = accounts::table.filter(accounts::login.eq(delete_account.login));

    diesel::delete(account_to_delete)
        .execute(connection)
        .expect("Error deleting account.");
}

pub fn show_account() {
    use crate::schema::accounts::dsl::*;

    let connection = &mut establish_connection();
    let results = accounts
        .load::<Account>(connection)
        .expect("Error loading accounts");

    for account in results {
        println!("{:?}", account)
    }
}
