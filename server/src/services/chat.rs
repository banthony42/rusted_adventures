use common::authenticator::Authenticator;
use common::character::CharacterHandler;
use common::constants::CHAT_SERVER_INPUT_MAX;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use tracing::instrument;

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

#[instrument(level = "debug")]
async fn broadcast(chat_event: RpgChatEvent, clients: ArcMutexHashMapClient) {
    let (sender, event) = chat_event.into_parts();

    // Get player on same map than the sender
    // Broadcast only to that list
    let mut handler = match CharacterHandler::new(&sender) {
        Ok(handler) => handler,
        Err(e) => {
            tracing::error!("RpgChatService broadcast: Load characters for: {sender}: {e}");
            return;
        }
    };

    let recipients = match handler.players_on_same_map() {
        Ok(list) => list,
        Err(e) => {
            tracing::error!("RpgChatService broadcast: Get recipients of {sender}: {e}");
            return;
        }
    };

    let broadcast = ServerChatEvent {
        seq_number: 0,
        text: event.text,
        sender: Some(sender.clone()),
        event: Some(Event::ChatEvent(ChatEventType::Broadcast as i32)),
    };

    tracing::debug!("RpgChatService: {sender} broadcast {broadcast:?} to {recipients:?}");

    {
        let clts = clients.lock().await;
        // Send the event to each client filtering out the sender to avoid send him it's own event.
        for player in recipients.iter().filter(|name| sender.ne(*name)) {
            if let Some(client_channel) = clts.get(player) {
                if let Err(err) = client_channel.send(Ok(broadcast.clone())).await {
                    tracing::error!("RpgChatService broadcast: Fail to send to {player}: {err}");
                }
            }
        }
    }
}

#[instrument(level = "debug")]
async fn whisper(chat_event: RpgChatEvent, clients: ArcMutexHashMapClient) {
    let (sender, event) = chat_event.into_parts();

    if let Some(recipient) = event.recipient {
        let whisper = ServerChatEvent {
            seq_number: 0,
            text: event.text,
            sender: Some(sender.clone()),
            event: Some(Event::ChatEvent(ChatEventType::Whisper as i32)),
        };

        let mut authenticator = Authenticator::new(&recipient);
        let recipient_disconnected = authenticator.is_connected(None).is_err();

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
                if let Err(e) = sender_event_tx.send(Ok(sender_unacknowledgement)).await {
                    tracing::error!("RpgChatService: whisper: Fail unacknowledge to {sender}: {e}")
                }
                return;
            }

            let sender_acknowledgement = ServerChatEvent {
                seq_number: event.seq_number, // Answer using same SequenceNumber
                text: String::default(),
                sender: None,
                event: Some(Event::ServerEvent(ServerEventType::SrvAck as i32)),
            };
            if let Err(e) = sender_event_tx.send(Ok(sender_acknowledgement)).await {
                tracing::error!("RpgChatService: whisper: Fail acknowledge to {sender}: {e}");
            }
        }

        if let Some(recipient_event_tx) = clts.get(&recipient) {
            tracing::debug!("RpgChatService: {sender} whisper {whisper:?} to {recipient}");
            if let Err(e) = recipient_event_tx.send(Ok(whisper)).await {
                tracing::error!("RpgChatService: whisper: Fail to send to {recipient}: {e}");
                return;
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
            while let Some(mut receive) = event_rx.recv().await {
                // Small protection against huge message sent
                // The channel size should also be considered, and update with care
                receive.event.text.truncate(CHAT_SERVER_INPUT_MAX);
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

    #[instrument(level = "debug")]
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
                            tracing::error!("RpgChatService chat: client {login}: broken pipe");
                            break;
                        }
                    }
                    tracing::info!("RpgChatService chat: client {login}: {status}");
                }

                if let Some(event) = chat_event.ok() {
                    if event.text.is_empty() {
                        continue;
                    }
                    if let Err(e) = event_tx.send(RpgChatEvent::new(login.clone(), event)).await {
                        tracing::error!("RpgChatService chat: client {login}: {e}");
                        break;
                    }
                }
            }
            tracing::info!("RpgChatService chat: client {login} disconnected");
            cl.lock().await.remove(&login);
        });

        // The ServerChatEvent rx channel is passed therefore
        // any data send through tx will be received by the gRPC codegen
        // and transmit to the client through gRPC request
        Ok(Response::new(ReceiverStream::new(server_event_rx)))
    }
}
