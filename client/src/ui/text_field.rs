use piston_window::*;
use types::Color;

use super::font::Font;

pub struct TextField<T>
where
    T: ContentFormat,
{
    font_size: u32,
    initial_rect: [u32; 4],
    rect: [u32; 4],
    content: Vec<T>,
    margin: Size,
    scroll: f64,
}

pub trait ContentFormat {
    fn content_format(&self) -> (Color, String);
}

impl<T> TextField<T>
where
    T: ContentFormat,
{
    pub fn new(font_size: u32, size: [u32; 4]) -> Self {
        TextField {
            font_size: font_size,
            initial_rect: size,
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
        let _ = self
            .content
            .iter()
            .map(|content| {
                let (text_color, text) = content.content_format();

                line_height += font.text_height_for_max_width(
                    text.as_str(),
                    self.font_size,
                    self.rect[2] as f64 - 8.0,
                ) as f64;

                let msg_position = [
                    self.rect[0] as f64 + 1.0,
                    self.rect[1] as f64 + self.rect[3] as f64 + 10.0 - line_height
                        + (self.scroll * self.font_size as f64),
                ];

                font.render_text_max_width(
                    text.as_str(),
                    self.font_size,
                    evnt,
                    window,
                    text_color,
                    msg_position,
                    self.rect[2] as f64,
                    self.rect,
                );
            })
            .collect::<Vec<_>>();
    }

    pub fn update(&mut self, _delta_ts: u128, mut text: Vec<T>) {
        text.reverse();
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
        self.rect = self.initial_rect.clone();
        self.rect[0] = self.initial_rect[0] + margin.width as u32;
        self.rect[1] = self.initial_rect[1] + margin.height as u32;
    }
}
