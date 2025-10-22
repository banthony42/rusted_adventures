use common::authenticator::Authenticator;
use common::character::CharacterHandler;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use common::grpc_codegen::rpg_chat_server::RpgChat;
use common::grpc_codegen::server_chat_event::Event;
use common::grpc_codegen::{ChatEventType, ClientChatEvent, ServerChatEvent, ServerEventType};

use crate::generics::match_for_io_error;
use crate::services::utils::login_from_metadata;

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
        Self { sender, event }
    }

    /// Consumes `self` returning the parts of the event.
    fn into_parts(self) -> (String, ClientChatEvent) {
        (self.sender, self.event)
    }
}

async fn broadcast(chat_event: RpgChatEvent, clients: ArcMutexHashMapClient) {
    println!("Server: RpgChatService: {:?}", chat_event);
    let (sender, event) = chat_event.into_parts();

    // Get player on same map than the sender
    // Broadcast only to that list
    let Ok(mut handler) = CharacterHandler::new(&sender) else {
        println!(
            "Server: chat broadcast: Failure while getting char handler for: {}",
            sender
        );
        return;
    };
    let Ok(player_list) = handler.players_on_same_map() else {
        println!(
            "Server: chat broadcast: Failure while getting players on same map for: {}",
            sender
        );
        return;
    };

    let new_event = ServerChatEvent {
        seq_number: 0,
        text: event.text,
        sender: Some(sender.clone()),
        event: Some(Event::ChatEvent(ChatEventType::Broadcast as i32)),
    };

    println!(
        "Server: {} chat broadcast with: {:?} to: {:?}",
        sender, new_event, player_list
    );
    {
        let clts = clients.lock().await;
        // Send the event to each client filtering out the sender to avoid send him it's own event.
        for player in player_list.iter().filter(|name| sender.ne(*name)) {
            if let Some(client_channel) = clts.get(player) {
                if let Err(err) = client_channel.send(Ok(new_event.clone())).await {
                    println!("Server: RpgChatService: Error: chat broadcast: {:?}", err);
                }
            }
        }
    }
}

async fn whisper(chat_event: RpgChatEvent, clients: ArcMutexHashMapClient) {
    println!("Server: RpgChatService: {:?}", chat_event);
    let (sender, event) = chat_event.into_parts();

    if let Some(recipient) = event.recipient {
        let new_event = ServerChatEvent {
            seq_number: 0,
            text: event.text,
            sender: Some(sender.clone()),
            event: Some(Event::ChatEvent(ChatEventType::Whisper as i32)),
        };

        let mut authenticator = Authenticator::new(&recipient);
        let recipient_disconnected = authenticator.is_connected().is_err();

        let clts = clients.lock().await;
        if let Some(sender_event_tx) = clts.get(&sender) {
            if recipient_disconnected {
                let sender_unacknowledgement = ServerChatEvent {
                    seq_number: event.seq_number, // Answer using same SequenceNumber
                    text: format!(
                        "Le joueur [{}] n'existe pas ou n'est pas disponnible.",
                        recipient
                    ),
                    sender: None,
                    event: Some(Event::ServerEvent(ServerEventType::SrvUnack as i32)),
                };
                if let Err(err) = sender_event_tx.send(Ok(sender_unacknowledgement)).await {
                    println!("Server: RpgChatService: Error: chat whisper: {:?}", err)
                }
                return;
            }

            let sender_acknowledgement = ServerChatEvent {
                seq_number: event.seq_number, // Answer using same SequenceNumber
                text: String::default(),
                sender: None,
                event: Some(Event::ServerEvent(ServerEventType::SrvAck as i32)),
            };
            if let Err(err) = sender_event_tx.send(Ok(sender_acknowledgement)).await {
                println!("Server: RpgChatService: Error: chat whisper: {:?}", err)
            }
        }

        if let Some(recipient_event_tx) = clts.get(&recipient) {
            if let Err(err) = recipient_event_tx.send(Ok(new_event)).await {
                println!("Server: RpgChatService: Error: chat whisper: {:?}", err)
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

        let clts = clients.clone();
        // This task loop on the ChatEvent receive channel to handle
        // ChatEvent for all connected clients.
        tokio::spawn(async move {
            while let Some(receive) = event_rx.recv().await {
                match receive.event.event() {
                    ChatEventType::Broadcast => broadcast(receive, clts.clone()).await,
                    ChatEventType::Whisper => whisper(receive, clts.clone()).await,
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
        let login = login_from_metadata(metadata)?;

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
                            println!("Server: RpgChatService client: {:?} : broken pipe", login);
                            break;
                        }
                    }
                    println!("Server: RpgChatService: client {:?} : {:?}", login, status);
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
            println!("Server: RpgChatService: client: {:?} disconnected", login);
            cl.lock().await.remove(&login);
        });

        // The ServerChatEvent rx channel is passed therefore
        // any data send through tx will be received by the gRPC codegen
        // and transmit to the client through gRPC request
        Ok(Response::new(ReceiverStream::new(server_event_rx)))
    }
}
