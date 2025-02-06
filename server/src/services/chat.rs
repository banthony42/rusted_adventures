use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
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
    event_tx: Sender<RpgChatEvent>,
}

#[derive(Debug)]
struct RpgChatEvent {
    sender: String,
    event: ClientChatEvent,
}

impl RpgChatEvent {
    fn new(sender: String, event: ClientChatEvent) -> Self {
        RpgChatEvent { sender, event }
    }

    /// Consumes `self` returning the parts of the event.
    fn into_parts(self) -> (String, ClientChatEvent) {
        (self.sender, self.event)
    }
}

async fn broadcast(chat_event: RpgChatEvent, clients: ArcMutexHashMapClient) {
    println!("{:?}", chat_event);
    let (sender, event) = chat_event.into_parts();

    let new_event = ServerChatEvent {
        text: event.text,
        sender: Some(sender.clone()),
        event: Some(Event::ChatEvent(ChatEventType::Broadcast as i32)),
    };

    let clts = clients.lock().await;

    // TODO: filter out clients on different map than the sender
    // Send the event to each client filtering out the sender to avoid send him it's own event.
    for (_, server_event_tx) in clts.iter().filter(|(name, _)| name.as_str().ne(&sender)) {
        if let Err(err) = server_event_tx.send(Ok(new_event.clone())).await {
            println!("Error: chat broadcast: {:?}", err);
        }
    }
}

async fn whisper(chat_event: RpgChatEvent, clients: ArcMutexHashMapClient) {
    println!("{:?}", chat_event);
    let (sender, event) = chat_event.into_parts();

    if let Some(recipient) = event.recipient {
        // TODO: find the recipient in the DB and ensure he is connected
        // If it's the case send him the msg

        let new_event = ServerChatEvent {
            text: event.text,
            sender: Some(sender.clone()),
            event: Some(Event::ChatEvent(ChatEventType::Whisper as i32)),
        };

        let clts = clients.lock().await;
        if let Some(server_event_tx) = clts.get(&recipient) {
            if let Err(err) = server_event_tx.send(Ok(new_event)).await {
                println!("Error: chat whisper: {:?}", err)
            }
        }
    }
}

impl RpgChatService {
    pub fn new() -> Self {
        let (event_tx, mut event_rx) = mpsc::channel::<RpgChatEvent>(10);

        let clients = Arc::new(Mutex::new(HashMap::<
            String,
            Sender<Result<ServerChatEvent, Status>>,
        >::new()));

        let clts_clone = clients.clone();
        // This task loop on the ChatEvent receive channel to handle
        // ChatEvent for all connected clients.
        tokio::spawn(async move {
            while let Some(receive) = event_rx.recv().await {
                match receive.event.event() {
                    ChatEventType::Broadcast => broadcast(receive, clts_clone.clone()).await,
                    ChatEventType::Whisper => whisper(receive, clts_clone.clone()).await,
                };
            }
        });

        Self { clients, event_tx }
    }
}

#[tonic::async_trait]
impl RpgChat for RpgChatService {
    type ChatStream = ReceiverStream<Result<ServerChatEvent, Status>>;

    async fn chat(
        &self,
        request: Request<Streaming<ClientChatEvent>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        let (metadata, _, mut client_stream) = request.into_parts();
        let login = metadata.get("login").unwrap().to_str().unwrap().to_string();

        let (server_event_tx, server_event_rx) =
            mpsc::channel::<Result<ServerChatEvent, Status>>(10);

        self.clients
            .lock()
            .await
            .insert(login.clone(), server_event_tx);

        // For each ClientChatEvent request receive from the stream,
        // send it through the RpgChatEvent channel
        // Therefore it will be process by the RpgChatEvent receive task
        let event_tx = self.event_tx.clone();
        let cl = self.clients.clone();
        tokio::spawn(async move {
            while let Some(chat_event) = client_stream.next().await {
                if let Err(status) = chat_event.as_ref() {
                    if let Some(io_err) = match_for_io_error(&status) {
                        if io_err.kind() == ErrorKind::BrokenPipe {
                            println!("RpgChatService client: {:?} : broken pipe", login);
                            break;
                        }
                    }
                    println!("RpgChatService: client {:?} : {:?}", login, status);
                }

                if let Some(event) = chat_event.ok() {
                    if event.text.is_empty() {
                        continue;
                    }
                    if let Err(_) = event_tx.send(RpgChatEvent::new(login.clone(), event)).await {
                        break;
                    }
                }
            }
            println!("RpgChatService: client: {:?} disconnected", login);
            cl.lock().await.remove(&login);
        });

        // The ServerChatEvent rx channel is passed therefore
        // any data send through tx will be received by the gRPC codegen
        // and transmit to the client through gRPC request
        Ok(Response::new(ReceiverStream::new(server_event_rx)))
    }
}

/*
** Tonic report the client disconnection within Error.source
** because it's a broken pipe raised by the h2 crate
** Thereis a pending issue on github to have better integrated way to detect
** client disconnection / broken pipe.
** For now i have just redo the tricks from the tonic/examples/src/streaming/server.rs
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
