use clap::{Args, Parser, Subcommand};

use common::database::models::NewAccount;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct DBCliArgs {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create, update, delete or show accounts
    Account(AccountCommand),
    /// Test commands to test things during dev
    Test(TestCommand),
}

#[derive(Debug, Args)]
pub struct TestCommand {
    #[clap(subcommand)]
    pub command: TestSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum TestSubcommand {
    /// Run Diesel / Postgresql custom types playground
    CustomTypes,
}

#[derive(Debug, Args)]
pub struct AccountCommand {
    #[clap(subcommand)]
    pub command: AccountSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountSubcommand {
    /// Create an new account
    Create(CreateAccount),

    /// Update an existing account
    Update(UpdateAccount),

    /// Delete an account
    Delete(DeleteAccount),

    /// Show all accounts
    Show,
}

#[derive(Debug, Args)]
pub struct CreateAccount {
    /// The name of the account
    pub login: String,

    /// The email of the account
    pub password: String,
}

impl Into<NewAccount> for CreateAccount {
    fn into(self) -> NewAccount {
        NewAccount {
            login: self.login,
            password: self.password,
            session_token: None,
        }
    }
}

#[derive(Debug, Args)]
pub struct UpdateAccount {
    /// The name of the account
    pub login: String,

    /// The email of the account
    pub password: String,
}

#[derive(Debug, Args)]
pub struct DeleteAccount {
    /// The name of the account to delete
    pub login: String,
}
