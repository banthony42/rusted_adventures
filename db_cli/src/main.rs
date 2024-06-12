use accounts_operations::{create_account, show_account};

#[macro_use]
mod schema;
mod db;
mod models;
mod accounts_operations;

fn main() {
    println!("==> TODO: Implem. args");

    create_account(String::from("-smirnof-"), String::from("biere"));
    create_account(String::from("fealhach"), String::from("fuckthesystem"));
    create_account(String::from("sulfurel"), String::from("boulanger"));

    show_account();
}
