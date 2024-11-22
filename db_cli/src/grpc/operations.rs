use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::runtime::Builder;
use tokio::select;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::{ChatCmd, GrpcCommand, GrpcSubcommand};

pub mod grpc_codegen {
    include!("../../../common/GRPC_codegen/rpg.package.rs");
}

use grpc_codegen::rpg_chat_client::RpgChatClient;
use grpc_codegen::server_chat_event::Event;
use grpc_codegen::{ChatEventType, ClientChatEvent};

fn run_cli_chat(chat_cmd: ChatCmd) {
    println!("Welcome to RPG Chat shell client.");

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let master_task = runtime.spawn(async move {
        let mut stdin_buff = BufReader::new(stdin()).lines();
        let (tx, rx) = mpsc::channel::<ClientChatEvent>(10);

        let mut client = RpgChatClient::connect("http://127.0.0.1:2121")
            .await
            .unwrap();

        // Send a first data here before listenning on the rx to avoid get blocked.
        let _ = tx
            .send(ClientChatEvent {
                login: chat_cmd.login.clone(),
                token: String::from("sulfurel-cafebab"),
                event: ChatEventType::Broadcast as i32,
                text: format!("{} joined.", chat_cmd.login.clone()),
                recipient: None,
            })
            .await
            .unwrap();

        // Pass the channel rx therefore we can easily write to the stream using tx
        let response = client.chat(ReceiverStream::new(rx)).await.unwrap();
        let mut client_stream = response.into_inner();

        loop {
            select! {
                user_input = stdin_buff.next_line() => match user_input {
                    Ok(l) => {
                        let input = l.unwrap();
                        let cmd : Vec<&str> = input.split(' ').collect();

                        let event = match cmd[0] {
                            "/w" => if cmd.len() >= 2 { ChatEventType::Whisper } else { ChatEventType::Broadcast },
                            _ => ChatEventType::Broadcast
                        };
                        let _ = tx.send(ClientChatEvent {
                            login: chat_cmd.login.clone(),
                            token: String::from("sulfurel-cafebab"),
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
                            if chat_cmd.login.ne(&sender) {
                                let evt = m.event.unwrap();
                                let prefix = match evt {
                                    Event::ChatEvent(1) => "mp de ", // TODO: Find a way to use ChatEventType::Broadcast as i32 or something else than raw value
                                    Event::ChatEvent(0)|
                                    Event::ChatEvent(_) => "General: ",
                                    Event::ServerEvent(_) => "SERVER: ",
                                };
                                println!("{}{}: {}",prefix, sender, m.text);
                            }
                        }
                    } else {
                        println!("rpg-chat-cli: receive stream error: {:?}", data);
                    }
                }
            }
        }
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
