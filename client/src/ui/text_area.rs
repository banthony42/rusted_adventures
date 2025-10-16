use common::constants::GUI_CHAT_PADDING_WIDTH;
use piston_window::*;
use types::Color;

use super::font::Font;

pub struct TextArea<T>
where
    T: TextAreaFormat,
{
    font_size: u32,
    rect: [f64; 4],
    position: [f64; 2],
    scissor: [u32; 4],
    content: Vec<T>,
    margin: Size,
    scroll: f64,
}

pub trait TextAreaFormat {
    fn colored_format(&self) -> (Color, String);
}

impl<T> TextArea<T>
where
    T: TextAreaFormat,
{
    pub fn new(font_size: u32, size: [f64; 4]) -> Self {
        TextArea {
            font_size: font_size,
            rect: size,
            content: vec![],
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            scroll: 0.0,
            position: [0.0; 2],
            scissor: [0; 4],
        }
    }

    fn height(&self) -> f64 {
        self.rect[3]
    }

    fn width(&self) -> f64 {
        self.rect[2]
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        let mut line_height = 0.0;
        let _ = self
            .content
            .iter()
            .rev()
            .map(|content| {
                let (text_color, text) = content.colored_format();

                line_height += font.text_height_for_max_width(
                    text.as_str(),
                    self.font_size,
                    self.width() - GUI_CHAT_PADDING_WIDTH,
                ) as f64;

                let msg_position = [
                    self.position[0],
                    self.position[1] - line_height + (self.scroll * self.font_size as f64),
                ];

                font.render_text_max_width(
                    text.as_str(),
                    self.font_size,
                    evnt,
                    window,
                    text_color,
                    msg_position,
                    self.width() - GUI_CHAT_PADDING_WIDTH,
                    self.scissor,
                );
            })
            .collect::<Vec<_>>();
    }

    pub fn update(&mut self, _delta_ts: u128, text: Vec<T>) {
        self.content = text;
    }

    pub fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.set_scroll(self.scroll + (args[1]));
    }

    pub fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll;
        if !self.content.is_empty() && self.scroll >= self.content.len() as f64 {
            self.scroll = (self.content.len() - 1) as f64;
        }
        if self.scroll < 0.0 {
            self.scroll = 0.0;
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        // Since we render text from the bottom to the top
        // position should start at bottom left of the chat rectangle area
        // we shift with (padding / 2), and text will be render with
        // line max width = width - padding
        // therefore it will remain (padding / 2) pixel minimum after the line
        self.position = [
            self.rect[0] + self.margin.width + (GUI_CHAT_PADDING_WIDTH / 2.0),
            self.rect[1] + self.margin.height + self.height() + 10.0,
        ];
        // Scissor start at top left of the chat rectangle area
        self.scissor = [
            (self.rect[0] + self.margin.width) as u32,
            (self.rect[1] + self.margin.height) as u32,
            self.width() as u32,
            self.height() as u32,
        ]
    }
}
