use clap::{Args, Subcommand};

pub mod operations;

#[derive(Debug, Args)]
pub struct GrpcCommand {
    #[clap(subcommand)]
    pub command: GrpcSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum GrpcSubcommand {
    Chat(ChatCmd),
}

#[derive(Debug, Args)]
pub struct ChatCmd {
    /// The sender name
    pub login: String,
}
