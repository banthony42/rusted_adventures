use std::sync::Arc;

use crate::{chat_client::ChatClient, constants::*, ui::text_field::ColoredFormat};
use chrono::Utc;
use common::grpc_codegen::ClientChatEvent;
use piston_window::*;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::ui::{font::Font, input_field::InputField, text_field::TextField};

use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::ChatEventType;
use common::grpc_codegen::ServerChatEvent;
use common::grpc_codegen::ServerEventType;

const CHAT_MAX_MSG: usize = 20;
const CHAT_FONT_SIZE: u32 = 17;
const CHAT_TIME_FORMAT: &str = "%H:%M:%S";

#[derive(Clone)]
enum Recipient {
    /// Entrant, en provenance
    Inbound(String),
    /// Sortant, en partance
    Outbound(String),
}

#[derive(Clone)]
struct ChatMessage {
    time: String,
    text: String,
    recipient: Option<Recipient>,
    event: SEvent,
}

impl ChatMessage {
    fn new(text: String, event: SEvent, recipient: Option<Recipient>) -> Self {
        ChatMessage {
            time: Utc::now().format(CHAT_TIME_FORMAT).to_string(),
            text,
            event,
            recipient,
        }
    }

    pub fn format(&self) -> String {
        if let Some(recipient) = &self.recipient {
            return match recipient {
                Recipient::Inbound(rcpt) => format!("[{}]: de {}: {}", self.time, rcpt, self.text),
                Recipient::Outbound(rcpt) => format!("[{}]: à {}: {}", self.time, rcpt, self.text),
            };
        }
        format!("[{}]: {}", self.time, self.text)
    }
}

impl ColoredFormat for ChatMessage {
    fn colored_format(&self) -> (types::Color, String) {
        match self.event {
            SEvent::ServerEvent(s) => match ServerEventType::try_from(s) {
                Ok(ServerEventType::SrvInfo) => (color::hex("06cc2a"), self.format()),
                Ok(ServerEventType::SrvWarn) => (color::YELLOW, self.format()),
                Ok(ServerEventType::SrvDang) => (color::RED, self.format()),
                Err(_) => (color::RED, String::from("Unexpected Event !!")),
            },
            SEvent::ChatEvent(c) => match ChatEventType::try_from(c) {
                Ok(ChatEventType::Broadcast) => (color::BLACK, self.format()),
                Ok(ChatEventType::Whisper) => (color::CYAN, self.format()),
                Err(_) => (color::RED, String::from("Unexpected Event !!")),
            },
        }
    }
}

impl TryFrom<ServerChatEvent> for ChatMessage {
    type Error = &'static str;

    fn try_from(value: ServerChatEvent) -> Result<Self, Self::Error> {
        if let Some(event) = value.event {
            let recipient = match value.sender {
                Some(sender) => Some(Recipient::Inbound(sender)),
                None => None,
            };
            return Ok(ChatMessage::new(value.text, event, recipient));
        }
        Err("ChatMessage need an Event to be construct.")
    }
}

impl TryInto<ClientChatEvent> for ChatMessage {
    type Error = &'static str;

    fn try_into(self) -> Result<ClientChatEvent, Self::Error> {
        let chat_event_type = match self.event {
            SEvent::ServerEvent(_) => None,
            SEvent::ChatEvent(c) => match ChatEventType::try_from(c) {
                Ok(evnt) => Some(evnt),
                Err(_) => None,
            },
        };

        if let Some(event) = chat_event_type {
            let recipient = match self.recipient {
                Some(Recipient::Inbound(r)) => Some(r),
                Some(Recipient::Outbound(r)) => Some(r),
                None => None,
            };

            return Ok(ClientChatEvent {
                event: event as i32,
                text: self.text,
                recipient: recipient,
            });
        }
        Err("ClientChatEvent need a valid Event::ChatEvent to be construct.")
    }
}

trait Trim {
    fn trim_v1(&mut self, len: usize);
    fn trim_v2(&mut self, len: usize);
}

impl Trim for Vec<ChatMessage> {
    /// Shortens the vector, keeping the last `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater or equal to the vector's current length, this has
    /// no effect.
    fn trim_v1(&mut self, len: usize) {
        // Compute the n first element to remove
        let first_n_to_remove = self.len().saturating_sub(len);
        // Remove the n first element, keeping the remaining in variable
        let mut remaining: Vec<_> = self.drain(first_n_to_remove..).collect();
        // Clear the whole array and replace it by the remaining (CHAT_MAX_MSG last elements)
        if !remaining.is_empty() {
            self.clear();
            self.append(&mut remaining);
        }
    }

    /// Shortens the vector, keeping the last `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater or equal to the vector's current length, this has
    /// no effect.
    fn trim_v2(&mut self, len: usize) {
        self.reverse();
        self.truncate(len);
        self.reverse();
    }
}

#[derive(Clone)]
struct ChatModel {
    model: Arc<Mutex<Vec<ChatMessage>>>,
    cache: Vec<ChatMessage>,
}

impl ChatModel {
    pub fn new() -> Self {
        ChatModel {
            model: Arc::new(Mutex::new(Vec::<ChatMessage>::new())),
            cache: Vec::default(),
        }
    }

    pub async fn post_from(&mut self, incoming: ServerChatEvent) {
        if let Ok(msg) = ChatMessage::try_from(incoming) {
            self.post_message(msg).await;
        }
    }

    pub async fn post_message(&mut self, msg: ChatMessage) {
        let mut model = self.model.lock().await;

        model.push(msg);
        model.trim_v1(CHAT_MAX_MSG);
    }

    pub async fn log_info(&mut self, text: &str) {
        self.post_message(ChatMessage::new(
            String::from(text),
            SEvent::ServerEvent(ServerEventType::SrvInfo as i32),
            None,
        ))
        .await
    }

    pub async fn log_warning(&mut self, text: &str) {
        self.post_message(ChatMessage::new(
            String::from(text),
            SEvent::ServerEvent(ServerEventType::SrvWarn as i32),
            None,
        ))
        .await
    }

    pub async fn log_danger(&mut self, text: &str) {
        self.post_message(ChatMessage::new(
            String::from(text),
            SEvent::ServerEvent(ServerEventType::SrvDang as i32),
            None,
        ))
        .await
    }

    pub fn get(&mut self) -> Vec<ChatMessage> {
        if let Ok(model) = self.model.try_lock() {
            self.cache = model.clone();
        }
        self.cache.clone()
    }
}

pub struct ChatController {
    model: ChatModel,
    view: ChatGraphicView,
    tx: Sender<ChatMessage>,
    _runtime: Runtime,
}

const CHAT_CONNEXION: &str = "Connexion au serveur de chat ...";
const CHAT_CONNECTED: &str = "Connecté au serveur de chat.";
const CHAT_CONNEXION_FAILED: &str = "La connexion au serveur de chat à échoué.";
const CHAT_CONNEXION_LOST: &str = "La connextion au serveur à été perdue.";
const CHAT_RECONNECT_TIMER: u64 = 5000;

impl ChatController {
    pub fn new(login: String, token: String) -> Self {
        let (controller_tx, mut controller_rx) = mpsc::channel::<ChatMessage>(10);
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        let mut model = ChatModel::new();
        let model_cloned = model.clone();

        runtime.spawn(async move {
            loop {
                model.log_info(CHAT_CONNEXION).await;

                if let Ok(connexion) = ChatClient::connect(login.clone(), token.clone()).await {
                    model.log_info(CHAT_CONNECTED).await;

                    let (stream_tx, response) = connexion.into_parts();
                    let mut stream = response.into_inner();
                    loop {
                        select! {
                            data = stream.message() => {
                                match data {
                                    Ok(Some(server_chat_event)) => model.post_from(server_chat_event).await,
                                    Ok(None) => { 
                                        model.log_warning(CHAT_CONNEXION_LOST).await;
                                        println!("Chat RPC Stream closed by the server.");
                                        break
                                    },
                                    Err(error) => {
                                        model.log_warning(CHAT_CONNEXION_LOST).await;
                                        println!("Chat receive gRPC error from server: {:?}", error);
                                    }
                                }
                            },
                            input = controller_rx.recv() => {
                                if let Some(msg) = input {
                                    model.post_message(msg.clone()).await;
                                    if let Ok(cli_event) = msg.try_into() {
                                        if let Err(error) =  stream_tx.send(cli_event).await {
                                            model.log_warning(CHAT_CONNEXION_LOST).await;
                                            println!("Chat stream tx error, maybe the rx has been dropped (grpc_codegen side): {:?}", error);
                                            break
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    model.log_danger(CHAT_CONNEXION_FAILED).await;
                };
                // The Chat connexion has failed, or has been shutdown
                // Wait some time before trying to reconnect
                sleep(Duration::from_millis(CHAT_RECONNECT_TIMER)).await;
            }
        });

        ChatController {
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
            "/w" if cmd.len() >= 2 => ChatMessage::new(
                cmd[2..].join(" "),
                SEvent::ChatEvent(ChatEventType::Whisper as i32),
                Some(Recipient::Outbound(cmd[1].to_string())),
            ),
            _ => ChatMessage::new(
                cmd.join(" "),
                SEvent::ChatEvent(ChatEventType::Broadcast as i32),
                None,
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

pub struct ChatGraphicView {
    input_field: InputField,
    text_field: TextField<ChatMessage>,
    margin: Size,
}

impl ChatGraphicView {
    pub fn new() -> Self {
        ChatGraphicView {
            input_field: InputField::new([16.0, 928.0], CHAT_FONT_SIZE, 416.0),
            text_field: TextField::new(
                CHAT_FONT_SIZE,
                [
                    GUI_CHAT_X as u32,
                    GUI_CHAT_Y as u32,
                    GUI_CHAT_WIDTH as u32,
                    GUI_CHAT_HEIGHT as u32,
                ],
            ),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        self.input_field.render(evnt, window, font);
        self.text_field.render(evnt, window, font);
    }

    pub fn update(&mut self, delta_ts: u128, model: Vec<ChatMessage>) {
        self.input_field.update(delta_ts);
        self.text_field.update(delta_ts, model);
    }

    pub fn text_input(&mut self, args: &String, font: &mut Font) {
        self.input_field.text_input(args, font);
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.input_field.mouse_cursor_args(args);
    }

    pub fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.text_field.mouse_scroll_args(args);
    }

    pub fn key_press(&mut self, args: &Button, font: &mut Font) -> Option<String> {
        self.input_field.key_press(args, font);

        if let Button::Keyboard(Key::Return) = args {
            if self.input_field.is_focus() {
                let user_input = self.input_field.get_content();
                if user_input.is_empty() == false {
                    self.input_field.clean();
                    self.text_field.set_scroll(0.0);
                    return Some(user_input);
                }
            }
        }
        None
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.input_field.resize(margin);
        self.text_field.resize(margin);
    }
}
