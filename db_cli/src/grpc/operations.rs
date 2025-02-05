use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::runtime::Builder;
use tokio::select;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Status};

use super::{ChatCmd, GrpcCommand, GrpcSubcommand};

pub mod grpc_codegen {
    include!("../../../common/GRPC_codegen/rpg.package.rs");
}

use grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use grpc_codegen::rpg_chat_client::RpgChatClient;
use grpc_codegen::server_chat_event::Event;
use grpc_codegen::{AuthReply, AuthRequest};
use grpc_codegen::{ChatEventType, ClientChatEvent};
use std::error::Error;

async fn connect_user(
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
        req.metadata_mut().insert("token", token_md);
        Ok(req)
    };
}

fn run_cli_chat(chat_cmd: ChatCmd) {
    let hardcode_password = String::from("1234");
    println!("Welcome to RPG Chat shell client.");
    println!(
        "(password not handle yet, all user will use '{:?}'.)",
        hardcode_password
    );

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let master_task = runtime.spawn(async move {
        let token = match connect_user(chat_cmd.login.clone(), hardcode_password).await {
            Ok(t) => t,
            Err(e) => {
                let tonic_error_msg = tonic::Status::from_error(e);
                panic!("CLI Chat error: connect_user: {:?}", tonic_error_msg);
            }
        };

        let endpoint = match Endpoint::from_static("http://127.0.0.1:2121").connect().await {
            Ok(connection) => connection,
            Err(err) => panic!("CLI Chat error: Chat endpoint: {:?}", err),
        };

        let mut client = RpgChatClient::with_interceptor(endpoint, create_interceptor(chat_cmd.login.clone(), token.clone()));

        let (tx, rx) = mpsc::channel::<ClientChatEvent>(10);
        // Send a first data here before listenning on the rx to avoid get blocked.
        let _ = tx
            .send(ClientChatEvent {
                event: ChatEventType::Broadcast as i32,
                text: format!("{} joined.", chat_cmd.login.clone()),
                recipient: None,
            })
            .await
            .unwrap();

        // Pass the channel rx therefore we can easily write to the stream using tx
        let response = client.chat(ReceiverStream::new(rx)).await.unwrap();
        let mut client_stream = response.into_inner();

        let mut stdin_buff = BufReader::new(stdin()).lines();
        loop {
            select! {
                user_input = stdin_buff.next_line() => match user_input {
                    Ok(l) => {
                        let input = l.unwrap();
                        let cmd : Vec<&str> = input.split(' ').collect();

                        let event = match cmd[0] {
                            "exit" => break,
                            "/w" => if cmd.len() >= 2 { ChatEventType::Whisper } else { ChatEventType::Broadcast },
                            _ => ChatEventType::Broadcast
                        };
                        let _ = tx.send(ClientChatEvent {
                            event: event as i32,
                            text: if event == ChatEventType::Whisper { cmd[2..].join(" ") } else { cmd.join(" ") },
                            recipient: if event == ChatEventType::Whisper { Some(cmd[1].to_string()) } else { None },
                        }).await.unwrap();
                    },
                    Err(e) =>  println!("rpg-chat-cli: user input error: {:?}", e),
                },
                data = client_stream.message() => {
                    if let Ok(msg) = data {
                        if let Some(m) = msg {
                            let sender = m.sender.unwrap();
                                let evt = m.event.unwrap();
                                let prefix = match evt {
                                    Event::ChatEvent(1) => "mp de ", // TODO: Find a way to use ChatEventType::Whisper as i32 or something else than raw value
                                    Event::ChatEvent(0)|
                                    Event::ChatEvent(_) => "General: ",
                                    Event::ServerEvent(_) => "SERVER: ",
                                };
                                println!("{}{}: {}",prefix, sender, m.text);
                        }
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
            println!("master task finished");
            break;
        }
    }
}

pub fn handle_grpc(grpc: GrpcCommand) {
    match grpc.command {
        GrpcSubcommand::Chat(chat_cmd) => run_cli_chat(chat_cmd),
    }
}
