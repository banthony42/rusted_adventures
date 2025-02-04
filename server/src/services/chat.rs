use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use super::grpc_codegen::rpg_chat_server::RpgChat;
use super::grpc_codegen::server_chat_event::Event;
use super::grpc_codegen::{ChatEventType, ClientChatEvent, ServerChatEvent};

type ArcMutexHashMapClient = Arc<Mutex<HashMap<String, Sender<Result<ServerChatEvent, Status>>>>>;
#[derive(Debug)]
pub struct RpgChatService {
    clients: ArcMutexHashMapClient,
    chat_event_tx: Sender<ClientChatEvent>,
}

async fn broadcast(event: ClientChatEvent, clients: ArcMutexHashMapClient) {
    println!("{}: Broadcast: {}", event.login, event.text);
    // Should find all player on the same map as the sender
    // and send them the msg

    // TODO: real implem needed.
    let serv_data = ServerChatEvent {
        text: event.text,
        sender: Some(event.login),
        event: Some(Event::ChatEvent(ChatEventType::Broadcast as i32)),
    };
    let registered_clients = clients.lock().await;
    for (name, s) in registered_clients.iter() {
        match registered_clients[name].send(Ok(serv_data.clone())).await {
            Ok(_) => {} /* Data transmit to client with success */
            Err(err) => {
                // Handle here client disconnection:
                // Example: one player broadcast all other players
                // One player disconnect at the same time
                // The server will detect the broken pipe
                // Then it will free it's data, therefore drop it's rx dedicated channel
                // (This is done by RPC generated code, the rx was returned as Stream by the RpgChatService.chat method)
                // Therefore this send will fail
                println!("Error: chat broadcast: {:?}", err);
            }
        }
    }
}

async fn whisper(event: ClientChatEvent, clients: ArcMutexHashMapClient) {
    println!("{}: Whisper: {}", event.login, event.text);
    // Should find the player in the DB and ensure he is connected
    // If it's the case send him the msg
    if event.recipient.is_none() {
        return;
    }
    // TODO: real implem needed.
    let serv_data = ServerChatEvent {
        text: event.text,
        sender: Some(event.login),
        event: Some(Event::ChatEvent(ChatEventType::Whisper as i32)),
    };

    let registered_clients = clients.lock().await;
    match registered_clients[&event.recipient.unwrap()]
        .send(Ok(serv_data))
        .await
    {
        Ok(_) => {} /* Data transmit to client with success */
        Err(err) => {
            // Handle here client disconnection:
            // Example: one player whisper another player
            // The targeted player disconnect at the same time
            // The server will detect the broken pipe
            // Then it will free it's data, therefore drop it's rx dedicated channel
            // (This is done by RPC generated code, the rx was returned as Stream by the RpgChatService.chat method)
            // Therefore this send will fail
            println!("Error: chat whisper: {:?}", err)
        }
    }
}

impl RpgChatService {
    pub fn new() -> Self {
        let (chat_event_tx, mut chat_event_rx) = mpsc::channel::<ClientChatEvent>(10);

        let clients = Arc::new(Mutex::new(HashMap::<
            String,
            Sender<Result<ServerChatEvent, Status>>,
        >::new()));

        let clts_clone = clients.clone();
        // This task loop on the ChatEvent receive channel to handle
        // ChatEvent for all connected clients.
        tokio::spawn(async move {
            while let Some(receive) = chat_event_rx.recv().await {
                match receive.event() {
                    ChatEventType::Broadcast => broadcast(receive, clts_clone.clone()).await,
                    ChatEventType::Whisper => whisper(receive, clts_clone.clone()).await,
                };
            }
        });

        return Self {
            clients,
            chat_event_tx,
        };
    }
}

#[tonic::async_trait]
impl RpgChat for RpgChatService {
    type ChatStream = ReceiverStream<Result<ServerChatEvent, Status>>;

    async fn chat(
        &self,
        request: Request<Streaming<ClientChatEvent>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        let mut client_stream = request.into_inner();

        let (server_event_tx, server_event_rx) =
            mpsc::channel::<Result<ServerChatEvent, Status>>(10);

        // For now we have to process the first message from the client
        // to get back it's name ... TODO: rework this
        // The ServerEvent tx channel for this client will be used later
        // to send to this client some ServerChatEvent
        let mut client_name = String::default();
        if let Some(msg) = client_stream.message().await.unwrap() {
            client_name = msg.login.clone();
            println!("Client chat connection: {:?}", msg);
            self.clients
                .lock()
                .await
                .insert(msg.login.clone(), server_event_tx);
            match self.chat_event_tx.send(msg).await {
                // TODO: understand why sending msg here, is it necessary ?
                Ok(_) => {} /* Event from client, sent to the server processing task with success */
                Err(_) => {} /* Should never append since receiver is never close */
            }
        }

        // For each ClientChatEvent request receive from the stream,
        // send it through the ChatEvent channel
        // Therefore it will be process by the ChatEvent receive task
        let chat_event_tx = self.chat_event_tx.clone();
        let cl = self.clients.clone();
        tokio::spawn(async move {
            while let Some(chat_event) = client_stream.next().await {
                match chat_event {
                    Ok(e) => match chat_event_tx.send(e).await {
                        Ok(_) => {}      /* Event from client send to the server processing task with success */
                        Err(_) => break, /* Should never append since receiver is never close */
                    },
                    Err(status) => {
                        // Tonic report the client disconnection within Error.source
                        // because it's a broken pipe raised by the h2 crate
                        // Thereis a pending issue on github to have better integrated way to detect
                        // client disconnection / broken pipe.
                        // For now i have just redo the tricks from the tonic/examples/src/streaming/server.rs
                        if let Some(io_err) = match_for_io_error(&status) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                println!("RpgChatService client: {:?} : broken pipe", client_name);
                                break;
                            }
                        }
                        // Maybe some other Errors have to be handled here
                        // For now break the loop and end the task for the client
                        // and print an error message.
                        println!("RpgChatService: client {:?} : {:?}", client_name, status);
                        break;
                    }
                }
            }
            println!("RpgChatService: client: {:?} disconnected", client_name);
            cl.lock().await.remove(&client_name);
        });

        // The ServerChatEvent rx channel is passed therefore
        // any data send through tx will be received by the gRPC codegen
        // and transmit to the client through gRPC request
        Ok(Response::new(ReceiverStream::new(server_event_rx)))
    }
}

/*
** From tonic repository to handle IO error, broken pipe
** to match client disconnections.
*/
fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;
    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }
        // h2::Error do not expose std::io::Error with `source()`
        // https://github.com/hyperium/h2/pull/462
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }
        err = err.source()?;
    }
}
