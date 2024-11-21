use tokio::runtime::Builder;

use super::{GrpcCommand, GrpcSubcommand};

pub mod grpc_codegen {
    include!("../../../common/GRPC_codegen/rpg.package.rs");
}

use grpc_codegen::rpg_chat_client::RpgChatClient;
use grpc_codegen::{ChatEventType, ClientChatEvent, ServerChatEvent};

pub fn handle_grpc(grpc: GrpcCommand) {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    // let (tx, mut rx) = mpsc::channel::<ServerChatEvent>(10);

    match grpc.command {
        GrpcSubcommand::Chat(chat_cmd) => {
            let master_task = runtime.spawn(async move {
                let mut client = RpgChatClient::connect("http://127.0.0.1:2121")
                    .await
                    .unwrap();

                let evt = match chat_cmd.event {
                    super::ChatCmdEventType::Broadcast => ChatEventType::Broadcast,
                    super::ChatCmdEventType::Whisper => ChatEventType::Whisper,
                };

                let request = tonic::Request::new(tokio_stream::iter(vec![ClientChatEvent {
                    login: chat_cmd.login,
                    token: String::from("sulfurel-cafebab"),
                    event: evt as i32,
                    text: chat_cmd.text,
                    recipient: None,
                }]));

                match client.chat(request).await {
                    Ok(r) => {
                        let mut resp = r.into_inner();

                        while let Ok(Some(ev)) = resp.message().await {
                            println!("====> {:?}", ev);
                        }
                    }
                    Err(e) => println!("something goes wrong: {:?}", e),
                }
            });

            loop {
                if master_task.is_finished() {
                    break;
                }
            }
        }
    }
}
