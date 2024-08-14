use chrono::Utc;
use piston_window::*;
use types::Color;
use crate::{
    constants::*,
    ui::{
        font::Font,
        input_field::InputField,
        text_field::TextField,
        text_field::ContentFormat
    }
};

#[derive(Clone)]
pub enum MessageType {
    General,
    Private
}

#[derive(Clone)]
pub struct Message {
    time: String,
    content: String,
    channel: MessageType,
    sender: String,
}

impl Message {
    pub fn format(&self) -> String {
        format!("[{}]: {}: {}", self.time, self.sender, self.content)
    }
}

impl ContentFormat for Message {
    fn content_format(&self) -> (Color, std::string::String) {
        match self.channel {
            MessageType::General => (color::BLACK, self.format()),
            MessageType::Private => (color::CYAN, self.format()),
        }
    }
}

pub struct Chat {
    input_field: InputField,
    text_field: TextField<Message>,
    margin: Size,
    client_name: String,
    content: Vec<Message>,
}

const CHAT_FONT_SIZE: u32 = 17;
const CHAT_TIME_FORMAT : &str = "%H:%M:%S";

impl Chat {

    pub fn new(client_name: String) -> Self {
        Chat {
            input_field: InputField::new([ 16.0, 928.0 ],
                CHAT_FONT_SIZE,
                416.0),
            text_field: TextField::new(
                CHAT_FONT_SIZE,
                [
                    GUI_CHAT_X as u32,
                    GUI_CHAT_Y as u32,
                    GUI_CHAT_WIDTH as u32,
                    GUI_CHAT_HEIGHT as u32
                    ]),
            client_name: client_name,
            margin: Size { width: 0.0, height: 0.0 },
            content: vec![ // TODO: Configure maximum size
                Message {  // TODO: remove this ! (tmp message to populate the chat)
                    content: String::from("Yooo !"),
                    time: Utc::now().format(CHAT_TIME_FORMAT).to_string(),
                    channel: MessageType::Private,
                    sender: String::from("fealhach")
                }
            ]
        }
    }

    pub fn render(&mut self, evnt : &Event, window: &mut PistonWindow, font: &mut Font) {
        self.input_field.render(evnt, window, font);
        self.text_field.render(evnt, window, font);
    }

    pub fn update(&mut self, delta_ts: u128) {
        self.input_field.update(delta_ts);
        self.text_field.update(delta_ts, self.content.clone());
    }

    pub fn text_input(&mut self, args: String) {
        self.input_field.text_input(args);
    }

    pub fn mouse_cursor_args(&mut self, args: [f64; 2]) {
        self.input_field.mouse_cursor_args(args);
    }

    pub fn mouse_scroll_args(&mut self, args: [f64; 2]) {
        self.text_field.mouse_scroll_args(args);
    }

    pub fn key_press(&mut self, args: &Button) {
        self.input_field.key_press(args);

        if let Button::Keyboard(Key::Return) = args {
            let user_input = self.input_field.get_content();
            if user_input.is_empty() == false {
                let mut a = vec![
                    Message {
                        content: user_input,
                        time: Utc::now().format(CHAT_TIME_FORMAT).to_string(),
                        channel: MessageType::General,
                        sender: self.client_name.clone()
                }];
                a.append(&mut self.content);
                self.content = a;
                self.input_field.clean();
                self.text_field.set_scroll(0.0);
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.input_field.resize(margin);
        self.text_field.resize(margin);
    }
}