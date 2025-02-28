use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::runtime::Builder;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status, Streaming};

use super::{ChatCmd, GrpcCommand, GrpcSubcommand};

use common::grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use common::grpc_codegen::rpg_chat_client::RpgChatClient;
use common::grpc_codegen::server_chat_event::Event;
use common::grpc_codegen::{AuthReply, AuthRequest};
use common::grpc_codegen::{ChatEventType, ClientChatEvent};
use common::grpc_codegen::{ServerChatEvent, ServerEventType};
use std::error::Error;

async fn authenticate_user(
    login: String,
    password: String,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut client = RpgAuthenticateClient::connect("http://127.0.0.1:2121").await?;

    let request = tonic::Request::new(AuthRequest {
        login: login.clone(),
        password: password.clone(),
    });

    let response: tonic::Response<AuthReply> = client.authenticate_user(request).await?;
    let token = response.into_inner().token.clone();
    Ok(token)
}

fn create_interceptor(
    login: String,
    token: String,
) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    return move |mut req: Request<()>| -> Result<Request<()>, Status> {
        let login_md: MetadataValue<_> = login.parse().unwrap();
        let token_md: MetadataValue<_> = token.parse().unwrap();

        req.metadata_mut().insert("login", login_md);
        req.metadata_mut().insert("authorization", token_md);
        Ok(req)
    };
}

type SenderResponse = (
    Sender<ClientChatEvent>,
    Response<Streaming<ServerChatEvent>>,
);

async fn connect_to_chat(
    login: String,
    token: String,
) -> Result<SenderResponse, Box<dyn Error + Send + Sync>> {
    let endpoint = Endpoint::from_static("http://127.0.0.1:2121")
        .connect()
        .await?;

    let mut client = RpgChatClient::with_interceptor(endpoint, create_interceptor(login, token));

    // Pass the channel rx therefore we can easily write to the stream using tx
    let (tx, rx) = mpsc::channel::<ClientChatEvent>(10);
    let response = client.chat(ReceiverStream::new(rx)).await?;

    Ok((tx, response))
}

enum UserInputCommand {
    None,
    Message(ClientChatEvent),
    Exit,
}

fn parse_input(line: Option<String>) -> UserInputCommand {
    if let Some(l) = line {
        let cmd: Vec<&str> = l.split(' ').collect();

        match cmd[0] {
            "exit" => return UserInputCommand::Exit,
            "/w" if cmd.len() >= 2 => {
                return UserInputCommand::Message(ClientChatEvent {
                    seq_number: 0,
                    event: ChatEventType::Whisper as i32,
                    text: cmd[2..].join(" "),
                    recipient: Some(cmd[1].to_string()),
                })
            }
            _ => {
                return UserInputCommand::Message(ClientChatEvent {
                    seq_number: 0,
                    event: ChatEventType::Broadcast as i32,
                    text: cmd.join(" "),
                    recipient: None,
                })
            }
        };
    }
    UserInputCommand::None
}

fn handle_receive_message(chat_event: Option<ServerChatEvent>) {
    if let Some(event) = chat_event {
        let sender = event.sender.unwrap_or_default();

        // TODO: The .proto describe that a ServerChatEvent can be a ServerEventType or a ChatEventType
        // Tonic generated code give us enum Event::ServerEvent(i32) / Event::ChatEvent(i32)
        // Unfortunately i didn't find yet the way to use ChatEventType enum within ChatEvent
        // Therefore to use an Event of type ChatEventType::Whisper
        // I have to use Event::ChatEvent(1) instead of Event::ChatEvent(ChatEventType:Whisper)
        let prefix = match event.event {
            Some(Event::ChatEvent(1)) => "mp de ",
            Some(Event::ChatEvent(0)) | Some(Event::ChatEvent(_)) => "General: ",
            Some(Event::ServerEvent(3)) => "SERVER: ACKNOWLEDGEMENT",
            Some(Event::ServerEvent(4)) => "SERVER: UNACKNOWLEDGEMENT",
            Some(Event::ServerEvent(_)) => "SERVER: ",
            None => return,
        };
        println!("{}{}: {}", prefix, sender, event.text);
    }
}

fn run_cli_chat(chat_cmd: ChatCmd) {
    let pass = String::from("1234");
    println!("Welcome to RPG Chat shell client.");
    println!("(password not handle yet, all user will use '{:?}'.)", pass);

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("Fail to invoke async context.");

    let master_task = runtime.spawn(async move {
        let token = authenticate_user(chat_cmd.login.clone(), pass)
            .await
            .expect("User authentication failed.");

        let (tx, response) = connect_to_chat(chat_cmd.login, token)
            .await
            .expect("Chat connection failed.");
        let mut stdin_buffer = BufReader::new(stdin()).lines();
        let mut client_stream = response.into_inner();

        loop {
            select! {
                user_input = stdin_buffer.next_line() => match user_input {
                    Ok(input) => match parse_input(input) {
                            UserInputCommand::Message(event) => tx.send(event).await.expect("rpg-chat-cli: fail to send ChatEvent."),
                            UserInputCommand::Exit => break,
                            _ => {}
                        },
                    Err(e) =>  println!("rpg-chat-cli: user input error: {:?}", e),
                },
                data = client_stream.message() => {
                    if let Ok(msg) = data {
                        handle_receive_message(msg);
                    } else {
                        println!("rpg-chat-cli: receive stream error: {:?}", data);
                    }
                }
            }
        }
        println!("Graceful disconnection.");
    });

    loop {
        if master_task.is_finished() {
            break;
        }
    }
}

pub fn handle_grpc(grpc: GrpcCommand) {
    match grpc.command {
        GrpcSubcommand::Chat(chat_cmd) => run_cli_chat(chat_cmd),
    }
}
