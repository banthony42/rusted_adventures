use clap::{Args, Subcommand};
use common::database::model::account::CreateAccount;

pub mod operations;

#[derive(Debug, Args)]
pub struct AccountCommand {
    #[clap(subcommand)]
    pub command: AccountSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountSubcommand {
    /// Create an new account
    Create(CreateAccountCmd),

    /// Update an existing account
    Update(UpdateAccountCmd),

    /// Delete an account
    Delete(DeleteAccountCmd),

    /// Show all accounts
    Show,
}

#[derive(Debug, Args)]
pub struct CreateAccountCmd {
    /// The name of the account
    pub login: String,

    /// The password of the account
    pub password: String,
}

impl Into<CreateAccount> for CreateAccountCmd {
    fn into(self) -> CreateAccount {
        CreateAccount {
            login: self.login,
            password: self.password,
        }
    }
}

#[derive(Debug, Args)]
pub struct UpdateAccountCmd {
    /// The name of the account
    pub login: String,

    /// The password of the account
    pub password: String,
}

#[derive(Debug, Args)]
pub struct DeleteAccountCmd {
    /// The name of the account to delete
    pub login: String,
}
