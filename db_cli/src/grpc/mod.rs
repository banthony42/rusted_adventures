use std::str::FromStr;

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

    /// The Chat event type to use
    pub event: ChatCmdEventType,

    /// The message content to send
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum ChatCmdEventType {
    Broadcast,
    Whisper,
}

impl FromStr for ChatCmdEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Broadcast" => Ok(Self::Broadcast),
            "Whisper" => Ok(Self::Whisper),
            _ => Err(String::from("Unknown variant")),
        }
    }
}
