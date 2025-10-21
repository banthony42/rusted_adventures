use std::collections::HashMap;

use common::record::Record;
use common::utils::SequenceNumber;
use piston_window::*;

use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

use super::client::ChatClient;
use super::model::{ChatMessage, ChatModel, Target};
use super::view::ChatGraphicView;

use crate::entities::model::UIEntityModel;
use crate::ui::font::Font;

use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::{ChatEventType, ClientChatEvent, ServerEventType};

pub struct ChatController {
    me: String,
    model: ChatModel,
    view: ChatGraphicView,
    tx: Sender<ChatMessage>,
    _runtime: Runtime,
}

const CHAT_RECONNECT_TIMER: u64 = 5000;
const CHAT_REQUEST_TTL: u32 = 5000;
const CHAT_CONNEXION: &str = "Connexion au serveur de chat ...";
const CHAT_CONNECTED: &str = "Connecté au serveur de chat.";
const CHAT_CONNEXION_FAILED: &str = "La connexion au serveur de chat à échoué.";
const CHAT_ERROR_FROM_SERVER: &str = "Le serveur de chat à renvoyé une erreur.";
const CHAT_CONNEXION_LOST: &str = "La connexion au serveur à été perdue.";
const CHAT_SERVER_RESPONSE_TIMEOUT: &str = "Le serveur a mis trop de temps à répondre.";
const CHAT_USAGE: &str = "/help Affiche ce message d'aide. /w [destinataire] [text ...] Envoie un message privé au destinataire.";
const CHAT_WHISPER_USAGE: &str = "chuchotement: nécéssite un destinataire et un contenu.";

impl ChatController {
    pub fn new(login: String, token: String) -> Self {
        let (controller_tx, mut controller_rx) = mpsc::channel::<ChatMessage>(10);
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        let mut model = ChatModel::new();
        let me = login.clone();
        let model_cloned = model.clone();

        runtime.spawn(async move {
            loop {
                model.local_info(CHAT_CONNEXION).await;

                if let Ok(connexion) = ChatClient::connect(login.clone(), token.clone()).await {
                    model.local_info(CHAT_CONNECTED).await;

                    let mut recorder: Record<ChatMessage> = Record::new(); // TEST: what append to previous instance ?
                    let mut seq_number = SequenceNumber::new();
                    let (stream_tx, response) = connexion.into_parts();
                    let mut stream = response.into_inner();
                    loop {
                        select! {
                            // Handle stream reception
                            data = stream.message() => {
                                match data {
                                    Ok(Some(server_chat_event)) => {

                                        if let Some(SEvent::ServerEvent(se)) = server_chat_event.event {
                                            match ServerEventType::try_from(se) {
                                                Ok(ServerEventType::SrvAck) => {
                                                    // A request has been ACK, we should retrieve the request in the cache
                                                    // and post it to the local chat model
                                                    // If the corresponding request is not found, it has expired
                                                    // Therefore do nothing since we already post a message in that case
                                                    // (See the second `select!` case)
                                                    if let Some(request) = recorder.getdel(&server_chat_event.seq_number.to_string()) {
                                                        model.post_message(request).await;
                                                    }
                                                },
                                                Ok(ServerEventType::SrvUnack) => {
                                                    // A request has been UNACK, we should delete the request from the cache
                                                    // Therefore if for any reason we received an ACK/UNACK with the same sequence number
                                                    // It will not conflict with the previous one
                                                    // If the request is successfully deleted post the UNACK message in the chat model.
                                                    // Otherwise do nothing (request has expired) we already post a message in that case
                                                    // (See the second `select!` case)
                                                    if recorder.del(&server_chat_event.seq_number.to_string()) {
                                                        model.post_from(server_chat_event).await;
                                                    }
                                                },
                                                Ok(_) => model.post_from(server_chat_event).await,
                                                Err(err) => println!("Chat failed to parse server event: {:?} with: {:?}", se, err),
                                            }
                                        }
                                        else {
                                            model.post_from(server_chat_event).await;
                                        }
                                    },
                                    Ok(None) => {
                                        model.local_warning(CHAT_CONNEXION_LOST).await;
                                        println!("Chat RPC Stream closed by the server.");
                                        break
                                    },
                                    Err(error) => {
                                        model.local_warning(CHAT_ERROR_FROM_SERVER).await;
                                        println!("Chat receive gRPC error from server: {:?}", error);
                                    }
                                }
                            },
                            // Update request records according to their TTL
                            _ = async {
                                sleep(Duration::from_millis(1000)).await;
                                if recorder.update() {
                                    model.local_danger(CHAT_SERVER_RESPONSE_TIMEOUT).await;
                                }
                            } => {},
                            // Handle player inputs, create ClientChatEvent from inputs, and send them to the stream
                            input = controller_rx.recv() => {
                                if let Some(msg) = input {
                                    let cli_event: Result<ClientChatEvent, _> = msg.clone().try_into();
                                    if let Ok(mut ce) = cli_event {
                                        match ChatEventType::try_from(ce.event) {
                                            Ok(ChatEventType::Whisper) => {
                                                // Server ACK needed: recipient could be unavailable or nonexistent
                                                // Set a sequence_number to follow the response
                                                // Store in the local cache the ChatMessage with the sequence number as key
                                                // Therefore it can be retrieve with the ACK/UNACK response which will have the same sequence_number
                                                ce.seq_number = seq_number.increment();
                                                recorder.set(ce.seq_number.to_string(), msg, Some(CHAT_REQUEST_TTL));
                                            },
                                            // For any kind of broadcast messages we don't need ACK
                                            Ok(_) => model.post_message(msg).await,
                                            Err(_) => println!("`try_from` error unexpected since previous `try_into` has succeed."),
                                        };
                                        if let Err(error) =  stream_tx.send(ce).await {
                                            model.local_warning(CHAT_CONNEXION_LOST).await;
                                            println!("Chat stream tx error, maybe the rx has been dropped (grpc_codegen side): {:?}", error);
                                            break
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    model.local_danger(CHAT_CONNEXION_FAILED).await;
                };
                // The Chat connexion has failed, or has been shutdown
                // Wait some time before trying to reconnect
                sleep(Duration::from_millis(CHAT_RECONNECT_TIMER)).await;
            }
        });

        ChatController {
            me,
            view: ChatGraphicView::new(),
            model: model_cloned,
            tx: controller_tx.clone(),
            _runtime: runtime,
        }
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        self.view.render(evnt, window, font);
    }

    pub fn update(&mut self, delta_ts: u128, ui_entities_model: HashMap<String, UIEntityModel>) {
        self.view
            .update(delta_ts, self.model.get(), ui_entities_model);
    }

    pub fn text_input(&mut self, args: &String, font: &mut Font) {
        self.view.text_input(args, font);
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.view.mouse_cursor_args(args);
    }

    pub fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.view.mouse_scroll_args(args);
    }

    fn parse_input(&self, line: String) -> ChatMessage {
        let cmd: Vec<&str> = line.split(' ').collect();

        match cmd[0] {
            "/help" | "/h" => ChatMessage::new(
                CHAT_USAGE.to_owned(),
                SEvent::ServerEvent(ServerEventType::SrvInfo as i32), // TODO: Create LocalInfo instead of using SrvInfo
                None,
            ),
            "/w" if cmd.len() < 3 => ChatMessage::new(
                CHAT_WHISPER_USAGE.to_owned(),
                SEvent::ServerEvent(ServerEventType::SrvDang as i32),
                None,
            ),
            "/w" if cmd.len() >= 3 => ChatMessage::new(
                cmd[2..].join(" "),
                SEvent::ChatEvent(ChatEventType::Whisper as i32),
                Some(Target::Outbound(cmd[1].to_string())),
            ),
            _ => ChatMessage::new(
                cmd.join(" "),
                SEvent::ChatEvent(ChatEventType::Broadcast as i32),
                Some(Target::Outbound(self.me.clone())),
            ),
        }
    }

    pub fn key_press(&mut self, args: &Button, font: &mut Font) {
        if let Some(user_input) = self.view.key_press(args, font) {
            let msg = self.parse_input(user_input);
            if let Err(error) = self.tx.try_send(msg) {
                println!("Chat controller tx error: {:?}", error);
            }
        };
    }

    pub fn resize(&mut self, margin: &Size) {
        self.view.resize(margin);
    }
}
