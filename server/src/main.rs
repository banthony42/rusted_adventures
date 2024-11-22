use std::collections::HashMap;
use std::sync::Arc;

use services::chat::RpgChatService;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tonic::transport::Server;

use services::authenticate::RpgAuthenticateService;
use services::grpc_codegen::rpg_authenticate_server::RpgAuthenticateServer;
use services::grpc_codegen::rpg_chat_server::RpgChatServer;
use services::grpc_codegen::server_chat_event::Event;
use services::grpc_codegen::{ChatEventType, ClientChatEvent, ServerChatEvent};
use tonic::Status;
use world::engine::WorldEngine;

pub mod proto {
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../common/GRPC_codegen/rpg_services_descriptor.bin");
}

mod services;
mod world;

fn run_world_engine_on_another_thread() -> Runtime {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("RPG World Engine")
        .enable_all()
        .build()
        .unwrap();

    let world_engine = WorldEngine::new();
    runtime.spawn(async move {
        world_engine.run().await;
    });

    return runtime;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:2121".parse()?;
    let rpg_authenticate = RpgAuthenticateService::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let (chat_event_tx, mut chat_event_rx) = mpsc::channel::<ClientChatEvent>(10);

    let shared_clients_a: Arc<Mutex<HashMap<String, Sender<Result<_, Status>>>>> =
        Arc::new(Mutex::new(HashMap::<
            String,
            Sender<Result<ServerChatEvent, Status>>,
        >::new()));

    let shared_clients = shared_clients_a.clone();
    let shared_clients_b = shared_clients_a.clone();
    // ClientChatEvent receiver
    tokio::spawn(async move {
        while let Some(receive) = chat_event_rx.recv().await {
            match receive.event() {
                ChatEventType::Broadcast => {
                    println!("{}: Broadcast: {}", receive.login, receive.text);
                    // Should find all player on the same map as the sender
                    // and send them the msg

                    let serv_data = ServerChatEvent {
                        text: receive.text,
                        sender: Some(receive.login),
                        event: Some(Event::ChatEvent(ChatEventType::Broadcast as i32)),
                    };
                    let registered_clients = shared_clients.lock().await;
                    for (name, s) in registered_clients.iter() {
                        match registered_clients[name].send(Ok(serv_data.clone())).await {
                            Ok(suc) => println!("===> client transmit success: {:?}", suc),
                            Err(err) => println!("===> error transmitting to client: {:?}", err),
                        }
                    }
                }
                ChatEventType::Whisper => {
                    println!("{}: Whisper: {}", receive.login, receive.text);
                    // Should find the player in the DB and ensure he is connected
                    // If it's the case send him the msg
                    if receive.recipient.is_none() {
                        return;
                    }

                    let serv_data = ServerChatEvent {
                        text: receive.text,
                        sender: Some(receive.login),
                        event: Some(Event::ChatEvent(ChatEventType::Whisper as i32)),
                    };

                    let registered_clients = shared_clients.lock().await;
                    match registered_clients[&receive.recipient.unwrap()]
                        .send(Ok(serv_data))
                        .await
                    {
                        Ok(suc) => println!("===> client transmit success: {:?}", suc),
                        Err(err) => println!("===> error transmitting to client: {:?}", err),
                    }
                }
            };
        }
    });

    let rpg_chat = RpgChatService::new(chat_event_tx, shared_clients_b.clone());

    // For setup simplicity the WorldEngine coexist within the Grpc Server.
    // I insist on the term 'WorldENGINE' because it's not a server since it not handle connections or requests.
    // We spawn a new thread where the WorldEngine will run (TODO: maybe we don't need the WorldEngine to be async for now)
    // Goal is to share a Mutex on the Database access to ensure DB read/write atomicity
    // Then the WorldEngine should update the world over the time (read/write)
    // and the Grpc server will just answer to client request reading the DB when needed.
    // Therefore with this approach it means that the Game States are stored in the postgresql DB.
    // I know it's not the best solution, maybe a redis should be better for performance
    // But again, the goal of the project is to learn rust, and i want to avoid
    // heavy setup configuration / prerequisite.
    // In addition i already know redis, where i totally discover and learn SQL like DB.
    let _rt = run_world_engine_on_another_thread();

    Server::builder()
        .add_service(reflection_service)
        .add_service(RpgAuthenticateServer::new(rpg_authenticate))
        .add_service(RpgChatServer::new(rpg_chat))
        .serve(addr)
        .await?;

    Ok(())
}
