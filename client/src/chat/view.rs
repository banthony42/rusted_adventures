use piston_window::*;

use crate::ui::{
    font::Font,
    input_field::InputField,
    text_area::{TextArea, TextAreaFormat},
};

use super::model::ChatMessage;

use common::constants::*;
use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::ChatEventType;
use common::grpc_codegen::ServerEventType;

impl TextAreaFormat for ChatMessage {
    fn colored_format(&self) -> (types::Color, String) {
        match self.event() {
            SEvent::ServerEvent(s) => match ServerEventType::try_from(*s) {
                Ok(ServerEventType::SrvInfo) => (color::hex("06cc2a"), self.format()),
                Ok(ServerEventType::SrvWarn) => (color::YELLOW, self.format()),
                Ok(ServerEventType::SrvDang) => (color::RED, self.format()),
                Ok(ServerEventType::SrvAck) | Ok(ServerEventType::SrvUnack) => {
                    (color::hex("f5b01a"), self.format())
                }
                Err(_) => (color::RED, String::from("Unexpected Event !!")),
            },
            SEvent::ChatEvent(c) => match ChatEventType::try_from(*c) {
                Ok(ChatEventType::Broadcast) => (color::BLACK, self.format()),
                Ok(ChatEventType::Whisper) => (color::CYAN, self.format()),
                Err(_) => (color::RED, String::from("Unexpected Event !!")),
            },
        }
    }
}

const CHAT_FONT_SIZE: u32 = 17;

pub struct ChatGraphicView {
    input_field: InputField,
    text_area: TextArea<ChatMessage>,
    margin: Size,
}

impl ChatGraphicView {
    pub fn new() -> Self {
        ChatGraphicView {
            input_field: InputField::new([16.0, 928.0], CHAT_FONT_SIZE, 416.0),
            text_area: TextArea::new(
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
        self.text_area.render(evnt, window, font);
    }

    pub fn update(&mut self, delta_ts: u128, model: Vec<ChatMessage>) {
        self.input_field.update(delta_ts);
        self.text_area.update(delta_ts, model);
    }

    pub fn text_input(&mut self, args: &String, font: &mut Font) {
        self.input_field.text_input(args, font);
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.input_field.mouse_cursor_args(args);
    }

    pub fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.text_area.mouse_scroll_args(args);
    }

    pub fn key_press(&mut self, args: &Button, font: &mut Font) -> Option<String> {
        self.input_field.key_press(args, font);

        if let Button::Keyboard(Key::Return) = args {
            if self.input_field.is_focus() {
                let user_input = self.input_field.get_content();
                if user_input.is_empty() == false {
                    self.input_field.clean();
                    self.text_area.set_scroll(0.0);
                    return Some(user_input);
                }
            }
        }
        None
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.input_field.resize(margin);
        self.text_area.resize(margin);
    }
}
