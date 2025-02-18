use piston_window::*;

use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

use super::client::ChatClient;
use super::model::{ChatMessage, ChatModel, Target};
use super::view::ChatGraphicView;

use crate::ui::font::Font;

use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::{ChatEventType, ServerEventType};

pub struct ChatController {
    me: String,
    model: ChatModel,
    view: ChatGraphicView,
    tx: Sender<ChatMessage>,
    _runtime: Runtime,
}

const CHAT_CONNEXION: &str = "Connexion au serveur de chat ...";
const CHAT_CONNECTED: &str = "Connecté au serveur de chat.";
const CHAT_CONNEXION_FAILED: &str = "La connexion au serveur de chat à échoué.";
const CHAT_ERROR_FROM_SERVER: &str = "Le serveur de chat à renvoyé une erreur.";
const CHAT_CONNEXION_LOST: &str = "La connextion au serveur à été perdue.";
const CHAT_RECONNECT_TIMER: u64 = 5000;

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

                    let (stream_tx, response) = connexion.into_parts();
                    let mut stream = response.into_inner();
                    loop {
                        select! {
                            data = stream.message() => {
                                match data {
                                    Ok(Some(server_chat_event)) => model.post_from(server_chat_event).await,
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
                            input = controller_rx.recv() => {
                                if let Some(msg) = input {
                                    model.post_message(msg.clone()).await;
                                    if let Ok(cli_event) = msg.try_into() {
                                        if let Err(error) =  stream_tx.send(cli_event).await {
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

    pub fn update(&mut self, delta_ts: u128) {
        self.view.update(delta_ts, self.model.get());
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
