use std::sync::Arc;

use chrono::Utc;

use tokio::sync::Mutex;

use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::ChatEventType;
use common::grpc_codegen::ClientChatEvent;
use common::grpc_codegen::ServerChatEvent;
use common::grpc_codegen::ServerEventType;

const CHAT_MAX_MSG: usize = 20;
const CHAT_TIME_FORMAT: &str = "%H:%M:%S";

#[derive(Clone)]
pub enum Target {
    /// Entrant, en provenance
    Inbound(String),
    /// Sortant, en partance
    Outbound(String),
}

impl Target {
    fn format_as_prefix(&self, event: ChatEventType) -> String {
        match &self {
            Target::Inbound(tgt) => match event {
                ChatEventType::Whisper => format!("de {}:", tgt),
                _ => format!("{}: ", tgt),
            },
            Target::Outbound(tgt) => match event {
                ChatEventType::Whisper => format!("à {}:", tgt),
                _ => format!("{}: ", tgt),
            },
        }
    }
}

#[derive(Clone)]
pub struct ChatMessage {
    time: String,
    text: String,
    target: Option<Target>,
    event: SEvent,
}

impl ChatMessage {
    pub fn new(text: String, event: SEvent, target: Option<Target>) -> Self {
        ChatMessage {
            time: Utc::now().format(CHAT_TIME_FORMAT).to_string(),
            text,
            event,
            target,
        }
    }

    pub fn event(&self) -> &SEvent {
        &self.event
    }

    pub fn format(&self) -> String {
        let mut prefix = match self.event {
            SEvent::ServerEvent(s) => match ServerEventType::try_from(s) {
                Ok(_) => "Système :".to_owned(),
                Err(_) => "<se::unknown>:".to_owned(),
            },
            SEvent::ChatEvent(c) => match ChatEventType::try_from(c) {
                Ok(chat_event) => {
                    if let Some(target) = &self.target {
                        target.format_as_prefix(chat_event)
                    } else {
                        String::default()
                    }
                }
                Err(_) => "<ce::unknown>:".to_owned(),
            },
        };
        if !prefix.is_empty() {
            prefix.push(' ');
        }
        format!("[{}]: {}{}", self.time, prefix, self.text)
    }
}

impl TryFrom<ServerChatEvent> for ChatMessage {
    type Error = &'static str;

    fn try_from(value: ServerChatEvent) -> Result<Self, Self::Error> {
        if let Some(event) = value.event {
            let target = match value.sender {
                Some(sender) => Some(Target::Inbound(sender)),
                None => None,
            };
            return Ok(ChatMessage::new(value.text, event, target));
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
            let target = match self.target {
                Some(Target::Inbound(target)) if event.eq(&ChatEventType::Whisper) => Some(target),
                Some(Target::Outbound(target)) if event.eq(&ChatEventType::Whisper) => Some(target),
                // Only the Whisper event need a recipient.
                Some(_) => None,
                None => None,
            };

            return Ok(ClientChatEvent {
                event: event as i32,
                text: self.text,
                recipient: target,
            });
        }
        Err("ClientChatEvent need a valid Event::ChatEvent to be construct.")
    }
}

trait Trim {
    fn _trim_v1(&mut self, len: usize);
    fn _trim_v2(&mut self, len: usize);
}

impl Trim for Vec<ChatMessage> {
    /// Shortens the vector, keeping the last `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater or equal to the vector's current length, this has
    /// no effect.
    fn _trim_v1(&mut self, len: usize) {
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
    fn _trim_v2(&mut self, len: usize) {
        self.reverse();
        self.truncate(len);
        self.reverse();
    }
}

#[derive(Clone)]
pub struct ChatModel {
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
        model._trim_v2(CHAT_MAX_MSG);
    }

    pub async fn local_info(&mut self, text: &str) {
        self.post_message(ChatMessage::new(
            String::from(text),
            SEvent::ServerEvent(ServerEventType::SrvInfo as i32),
            None,
        ))
        .await
    }

    pub async fn local_warning(&mut self, text: &str) {
        self.post_message(ChatMessage::new(
            String::from(text),
            SEvent::ServerEvent(ServerEventType::SrvWarn as i32),
            None,
        ))
        .await
    }

    pub async fn local_danger(&mut self, text: &str) {
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
