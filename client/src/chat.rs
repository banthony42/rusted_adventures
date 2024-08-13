use chrono::Utc;
use piston_window::*;
use crate::{
    constants,
    ui::{
        font::Font,
        text_field::TextField
    }
};

pub enum MessageType {
    General,
    Private
}

pub struct Message {
    time: String,
    content: String,
    channel: MessageType,
    sender: String,
}

pub struct Chat {
    text_field: TextField,
    margin: Size,
    client_name: String,
    content: Vec<Message>
}

const TIME_FORMAT : &str = "%d-%m-%Y %H:%M:%S";

impl Chat {

    pub fn new(client_name: String) -> Self {
        Chat {
            text_field: TextField::new([ 16.0, 928.0 ], 17, 416.0),
            client_name: client_name,
            margin: Size { width: 0.0, height: 0.0 },
            content: vec![ // TODO: Configure maximum size
                Message {  // TODO: remove this ! (tmp message to populate the chat)
                    content: String::from("Yooo"),
                    time: Utc::now().format(TIME_FORMAT).to_string(),
                    channel: MessageType::Private,
                    sender: String::from("fealhach")
                }
            ]
        }
    }

    pub fn render(&mut self, evnt : &Event, window: &mut PistonWindow, font: &mut Font) {
        self.text_field.render(evnt, window, font);
        self.text_field.render(evnt, window, font);

        let _ = self.content.iter().enumerate().map(|(index, msg)| {
            let msg_position = [
                16.0 + 5.0,
                928.0 - 10.0 - (index * 17) as f64
            ];
            let msg_color = match msg.channel {
                MessageType::General => color::BLACK,
                MessageType::Private => color::CYAN
            };

            let final_msg = format!("[{}]: {}: {}", msg.time, msg.sender, msg.content);
            font.render_text(final_msg.as_str(), 17, evnt, window, msg_color, msg_position, Some(&self.margin));
        }).collect::<Vec<_>>();
    }

    pub fn update(&mut self, delta_ts: u128) {
        self.text_field.update(delta_ts);
    }

    pub fn text_input(&mut self, args: String) {
        self.text_field.text_input(args);
    }

    pub fn mouse_cursor_args(&mut self, args: [f64; 2]) {
        self.text_field.mouse_cursor_args(args);
    }
        
    pub fn key_press(&mut self, args: &Button) {
        self.text_field.key_press(args);

        if let Button::Keyboard(Key::Return) = args {
            let user_input = self.text_field.get_content();
            if user_input.len() > 0 {
                let mut a = vec![
                    Message {
                        content: user_input,
                        time: Utc::now().format(TIME_FORMAT).to_string(),
                        channel: MessageType::General,
                        sender: self.client_name.clone()
                }];
                a.append(&mut self.content);
                self.content = a;
                self.text_field.clean();
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.text_field.resize(margin);
    }
}