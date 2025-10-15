use std::collections::HashMap;

use piston_window::*;

use crate::{
    entities::model::UIEntityModel,
    import::assets::SpeciesLibrary,
    ui::{
        font::Font,
        input_field::InputField,
        text_area::{TextArea, TextAreaFormat},
        text_window::{TextWindow, TextWindowFormat},
    },
};

use super::model::ChatMessage;

use common::grpc_codegen::server_chat_event::Event as SEvent;
use common::grpc_codegen::ChatEventType;
use common::grpc_codegen::ServerEventType;
use common::{constants::*, utils::get_timestamp};

impl TextAreaFormat for ChatMessage {
    fn colored_format(&self) -> (types::Color, String) {
        match self.event() {
            SEvent::ServerEvent(s) => match ServerEventType::try_from(*s) {
                Ok(ServerEventType::SrvInfo) => (color::hex("06cc2a"), self.chat_area_format()),
                Ok(ServerEventType::SrvWarn) => (color::YELLOW, self.chat_area_format()),
                Ok(ServerEventType::SrvDang) => (color::RED, self.chat_area_format()),
                Ok(ServerEventType::SrvAck) | Ok(ServerEventType::SrvUnack) => {
                    (color::hex("f5b01a"), self.chat_area_format())
                }
                Err(_) => (color::RED, String::from("Unexpected Event !!")),
            },
            SEvent::ChatEvent(c) => match ChatEventType::try_from(*c) {
                Ok(ChatEventType::Broadcast) => (color::BLACK, self.chat_area_format()),
                Ok(ChatEventType::Whisper) => (color::CYAN, self.chat_area_format()),
                Err(_) => (color::RED, String::from("Unexpected Event !!")),
            },
        }
    }
}

#[derive(PartialEq)]
struct WindowChatMessage {
    msg: ChatMessage,
    ui_model: UIEntityModel,
}

impl TextWindowFormat for WindowChatMessage {
    fn format(&self) -> String {
        self.msg.chat_window_format()
    }

    fn position(&self) -> [u32; 2] {
        [
            self.ui_model.real_position.x as u32,
            self.ui_model.real_position.y as u32,
        ]
    }

    fn offset(&self, species_lib: &SpeciesLibrary) -> [f64; 2] {
        [0.0, species_lib.get_height_offset(&self.ui_model.species)]
    }
}

const CHAT_FONT_SIZE: u32 = 17;

pub struct ChatGraphicView {
    chat_input: InputField,
    chat_area: TextArea<ChatMessage>,
    chat_window: TextWindow<WindowChatMessage>,
    margin: Size,
}

impl ChatGraphicView {
    pub fn new() -> Self {
        ChatGraphicView {
            //  drop all WindowMessage with map != ui_model[WindowMessage.sender].map
            chat_window: TextWindow::new(),
            chat_input: InputField::new([16.0, 928.0], CHAT_FONT_SIZE, 416.0),
            chat_area: TextArea::new(
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
        self.chat_input.render(evnt, window, font);
        self.chat_area.render(evnt, window, font);
        self.chat_window.render(evnt, window, font);
    }

    /// If the given ChatMessage is a broadcast message and less than 200ms age, return true.
    fn is_recent_broadcast(msg: &&ChatMessage) -> bool {
        let now = get_timestamp();
        match msg.event() {
            SEvent::ServerEvent(_) => false,
            SEvent::ChatEvent(c) => match ChatEventType::try_from(*c) {
                Ok(ChatEventType::Broadcast) => (now - msg.time()) < 200,
                _ => false,
            },
        }
    }

    pub fn update(
        &mut self,
        delta_ts: u128,
        model: Vec<ChatMessage>,
        ui_models: HashMap<String, UIEntityModel>,
    ) {
        self.chat_input.update(delta_ts);
        self.chat_area.update(delta_ts, model.clone());

        self.chat_window.retain_message(|window_msg| {
            let Some(sender) = window_msg.msg.target() else {
                // Not expected, since we add window_msg only if target exist.
                // But its handled, System message don't have target
                return false;
            };
            let Some(ui_model) = ui_models.get(sender.inner()) else {
                // If no model found for this sender, just drop the window message
                // We can't display it, we need this data
                // It could mean that entity that send this message
                // Is not on the map anymore
                return false;
            };
            // Keep all messages comming from entities that are still on the map
            ui_model.map == window_msg.ui_model.map
        });

        self.chat_window.update(delta_ts, |window_msg| {
            let Some(sender) = window_msg.msg.target() else {
                // Not expected, since we add window_msg only if target exist.
                // But its handled, System message don't have target
                return;
            };
            let Some(ui_model) = ui_models.get(sender.inner()) else {
                // If no model found for this sender, just drop the window message
                // We can't display it, we need this data
                // It could mean that entity that send this message
                // Is not on the map anymore
                return;
            };
            window_msg.ui_model.real_position = ui_model.real_position;
        });

        for msg in model.iter().filter(Self::is_recent_broadcast) {
            let Some(sender) = msg.target() else {
                // No sender for this message skip it
                continue;
            };
            let Some(ui_model) = ui_models.get(sender.inner()) else {
                // Fail to get model ui data for sender skip it
                continue;
            };
            let new_msg = WindowChatMessage {
                msg: msg.clone(),
                ui_model: ui_model.clone(),
            };
            self.chat_window.add_message(new_msg, |window_msg| {
                window_msg.msg.target().as_ref().eq(&Some(sender))
            });
        }
    }

    pub fn text_input(&mut self, args: &String, font: &mut Font) {
        self.chat_input.text_input(args, font);
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.chat_input.mouse_cursor_args(args);
    }

    pub fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.chat_area.mouse_scroll_args(args);
    }

    pub fn key_press(&mut self, args: &Button, font: &mut Font) -> Option<String> {
        self.chat_input.key_press(args, font);

        if let Button::Keyboard(Key::Return) = args {
            if self.chat_input.is_focus() {
                let user_input = self.chat_input.get_content();
                if user_input.is_empty() == false {
                    self.chat_input.clean();
                    self.chat_area.set_scroll(0.0);
                    return Some(user_input);
                }
            }
        }

        if let Button::Keyboard(Key::Tab) = args {
            self.chat_input.set_focus(!self.chat_input.is_focus());
        }
        None
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.chat_input.resize(margin);
        self.chat_area.resize(margin);
        self.chat_window.resize(margin);
    }
}
