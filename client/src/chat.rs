use std::sync::Arc;

use crate::{
    constants::*,
    ui::{font::Font, input_field::InputField, text_field::ContentFormat, text_field::TextField},
};
use chrono::Utc;
use piston_window::*;
use tokio::{
    runtime::{Builder, Runtime},
    select,
    sync::{
        mpsc::{self, Sender},
        Mutex, MutexGuard,
    },
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status, Streaming};
use types::Color;

use common::grpc_codegen::rpg_chat_client::RpgChatClient;
use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::ClientChatEvent;
use common::grpc_codegen::ServerChatEvent;
use std::error::Error;

const CHAT_MAX_MSG: usize = 20;
const CHAT_FONT_SIZE: u32 = 17;
const CHAT_TIME_FORMAT: &str = "%H:%M:%S";

#[derive(Clone, Debug)]
enum MessageType {
    General,
    Private,
    Info,
}

#[derive(Clone, Debug)]
pub struct Message {
    time: String,
    content: String,
    channel: MessageType,
    recipient: Option<String>,
}

impl Message {
    fn new(content: String, channel: MessageType, sender: Option<String>) -> Self {
        Message {
            time: Utc::now().format(CHAT_TIME_FORMAT).to_string(),
            content,
            channel,
            recipient: sender,
        }
    }

    pub fn format(&self) -> String {
        match &self.recipient {
            Some(sender) => format!("[{}]: {}: {}", self.time, sender, self.content),
            None => format!("[{}]: {}", self.time, self.content),
        }
    }
}

impl ContentFormat for Message {
    fn content_format(&self) -> (Color, std::string::String) {
        match self.channel {
            MessageType::General => (color::BLACK, self.format()),
            MessageType::Private => (color::CYAN, self.format()),
            MessageType::Info => (color::hex("06cc2a"), self.format()),
        }
    }
}

type SenderResponse = (
    Sender<ClientChatEvent>,
    Response<Streaming<ServerChatEvent>>,
);

fn create_interceptor(
    login: String,
    token: String,
) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    return move |mut req: Request<()>| -> Result<Request<()>, Status> {
        let login_md: MetadataValue<_> = login.parse().unwrap();
        let token_md: MetadataValue<_> = token.parse().unwrap();

        req.metadata_mut().insert("login", login_md);
        req.metadata_mut().insert("authorization", token_md);
        Ok(req)
    };
}

async fn connect_to_chat(
    login: String,
    token: String,
) -> Result<SenderResponse, Box<dyn Error + Send + Sync>> {
    let endpoint = Endpoint::from_static("http://127.0.0.1:2121")
        .connect()
        .await?;

    let mut client = RpgChatClient::with_interceptor(endpoint, create_interceptor(login, token));

    // Pass the channel rx therefore we can easily write to the stream using tx
    let (tx, rx) = mpsc::channel::<ClientChatEvent>(10);
    let response = client.chat(ReceiverStream::new(rx)).await?;

    Ok((tx, response))
}

fn handle_receive_message(
    chat_event: Option<ServerChatEvent>,
    mut content: MutexGuard<'_, Vec<Message>>,
) {
    if let Some(event) = chat_event {
        let sender = event.sender.unwrap_or_default();

        // TODO: The .proto describe that a ServerChatEvent can be a ServerEventType or a ChatEventType
        // Tonic generated code give us enum Event::ServerEvent(i32) / Event::ChatEvent(i32)
        // Unfortunately i didn't find yet the way to use ChatEventType enum within ChatEvent
        // Therefore to use an Event of type ChatEventType::Whisper
        // I have to use Event::ChatEvent(1) instead of Event::ChatEvent(ChatEventType:Whisper)
        let msg_type = match event.event {
            Some(SEvent::ChatEvent(1)) => MessageType::Private,
            Some(SEvent::ChatEvent(0)) | Some(SEvent::ChatEvent(_)) => MessageType::General,
            Some(SEvent::ServerEvent(_)) => MessageType::Info,
            None => return,
        };
        content.push(Message::new(event.text, msg_type, Some(sender)));
        // Compute the n first element to remove
        let first_n_to_remove = content.len().saturating_sub(CHAT_MAX_MSG);
        // Remove the n first element, keeping the remaining in variable
        let mut remaining_content: Vec<_> = content.drain(first_n_to_remove..).collect();
        // Clear the whole array and replace it by the remaining (CHAT_MAX_MSG last elements)
        if !remaining_content.is_empty() {
            content.clear();
            content.append(&mut remaining_content);
        }
    }
}

struct ChatController {
    content: Arc<Mutex<Vec<Message>>>,
    cache_content: Vec<Message>,
    tx: Sender<Message>,
    _rt: Runtime,
    timer: u128,
}

impl ChatController {
    fn new(login: String, token: String) -> Self {
        let content = Arc::new(Mutex::new(Vec::<Message>::new()));
        let (controller_tx, mut controller_rx) = mpsc::channel::<Message>(10);
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        let controller = ChatController {
            content: content.clone(),
            cache_content: vec![],
            tx: controller_tx.clone(),
            _rt: runtime,
            timer: 0,
        };

        // Spawn a task to receive data from server
        controller._rt.spawn(async move {
            // Connect to chat returning stream to wait on and tx to write on
            let (stream_tx, response) = connect_to_chat(login, token)
            .await
            .expect("Chat connection failed.");

            // let (stream_tx, mut stream_rx) = mpsc::channel::<ClientChatEvent>(10);
            let mut client_stream = response.into_inner();
            loop {
                println!("\n");
                select! {
                    data = client_stream.message() => {
                        if let Ok(msg) = data {
                            let c = content.lock().await;
                            handle_receive_message(msg, c);
                        } else {
                            println!("rpg-chat-cli: receive stream error: {:?}", data);
                        }
                    },
                    receive = controller_rx.recv() => {
                        if let Some(recv) = receive                     {
                            // Update the chat content for graphic render later
                            {
                                println!("Controller: receive: await lock ...");
                                let mut c = content.lock().await;
                                println!("Controller: receive: lock obtained: push ...");
                                c.push(recv.clone());
                                // Compute the n first element to remove
                                let first_n_to_remove = c.len().saturating_sub(CHAT_MAX_MSG);
                                // Remove the n first element, keeping the remaining in variable
                                let mut remaining_content: Vec<_> = c.drain(first_n_to_remove..).collect();
                                // Clear the whole array and replace it by the remaining (CHAT_MAX_MSG last elements)
                                if !remaining_content.is_empty() {
                                    c.clear();
                                    c.append(&mut remaining_content);
                                }
                                dbg!("Controller: receive: content: len:{:?} last:{:?}", c.len(), c.last());
                            }
                            println!("Controller: receive: stream_tx send ...");
                            // Send the received msg to the task which will send it to the server
                            stream_tx
                                .send(ClientChatEvent {
                                    event: recv.channel as i32,
                                    text: recv.content,
                                    recipient: recv.recipient,
                                })
                                .await.unwrap(); // Handle error here (server down / disconnection) At least logError in chat to warn player
                        } else {
                            println!("Controller: receive: error");
                        }
                    },
                }
            }
            println!("Loop break ! end async task !");
        });
        return controller;
    }

    fn get_content(&mut self) -> Vec<Message> {
        if let Ok(content) = self.content.try_lock() {
            self.cache_content = content.clone();
        } else {
            println!("Controller: get_content: fail to obtained lock ...");
        }
        self.cache_content.clone()
    }

    fn push_message(&self, msg: Message) {
        match self.tx.blocking_send(msg) {
            Ok(_) => {}
            Err(e) => println!("Chat controller: push_message: {:?}", e),
        };
    }
}

pub struct Chat {
    input_field: InputField,
    text_field: TextField<Message>,
    margin: Size,
    controller: ChatController,
}

impl Chat {
    pub fn new(client_name: String, token: String) -> Self {
        let mut new_instance = Chat {
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
            controller: ChatController::new(client_name, token),
        };
        new_instance.log_info("Bienvenue dans RPG!");
        return new_instance;
    }

    fn log(&mut self, text: &str, channel: MessageType) {
        self.push_message(Message::new(text.to_string(), channel, None));
    }

    pub fn log_info(&mut self, text: &str) {
        self.log(text, MessageType::Info);
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        self.input_field.render(evnt, window, font);
        self.text_field.render(evnt, window, font);
    }

    pub fn update(&mut self, delta_ts: u128) {
        self.input_field.update(delta_ts);
        self.text_field
            .update(delta_ts, self.controller.get_content());
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

    fn push_message(&mut self, msg: Message) {
        self.controller.push_message(msg);
    }

    fn parse_input(&self, line: String) -> Message {
        let cmd: Vec<&str> = line.split(' ').collect();

        match cmd[0] {
            "/w" if cmd.len() >= 2 => Message::new(
                cmd[2..].join(" "),
                MessageType::Private,
                Some(cmd[1].to_string()),
            ),
            _ => Message::new(cmd.join(" "), MessageType::General, None),
        }
    }

    pub fn key_press(&mut self, args: &Button, font: &mut Font) {
        self.input_field.key_press(args, font);

        if let Button::Keyboard(Key::Return) = args {
            if self.input_field.is_focus() {
                let user_input = self.input_field.get_content();
                if user_input.is_empty() == false {
                    self.push_message(self.parse_input(user_input));
                    self.input_field.clean();
                    self.text_field.set_scroll(0.0);
                }
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.input_field.resize(margin);
        self.text_field.resize(margin);
    }
}
