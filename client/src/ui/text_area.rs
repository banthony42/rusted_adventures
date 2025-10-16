use piston_window::*;
use types::Color;

use super::font::Font;

pub struct TextArea<T>
where
    T: TextAreaFormat,
{
    font_size: u32,
    rect: [u32; 4],
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
    pub fn new(font_size: u32, size: [u32; 4]) -> Self {
        TextArea {
            font_size: font_size,
            rect: size,
            content: vec![],
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            scroll: 0.0,
        }
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        let mut line_height = 0.0;
        let padding_width = 5.0;
        let _ = self
            .content
            .iter()
            .rev()
            .map(|content| {
                let (text_color, text) = content.colored_format();

                line_height += font.text_height_for_max_width(
                    text.as_str(),
                    self.font_size,
                    self.rect[2] as f64 - padding_width,
                ) as f64;

                let bg_rect = [
                    self.rect[0] as f64 + self.margin.width,
                    self.rect[1] as f64 + self.margin.height,
                    self.rect[2] as f64,
                    self.rect[3] as f64,
                ];

                let msg_position = [
                    bg_rect[0] + (padding_width / 2.0),
                    bg_rect[1] + bg_rect[3] + 10.0 - line_height
                        + (self.scroll * self.font_size as f64),
                ];

                font.render_text_max_width(
                    text.as_str(),
                    self.font_size,
                    evnt,
                    window,
                    text_color,
                    msg_position,
                    bg_rect[2] as f64 - padding_width,
                    bg_rect.map(|v| v as u32),
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
    }
}
