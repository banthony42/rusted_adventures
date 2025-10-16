use common::constants::{
    CHAT_WINDOW_MARGIN_BOTTOM, CHAT_WINDOW_PADDING_HEIGHT, CHAT_WINDOW_PADDING_WIDTH,
    GUI_ENTITY_FONT_SIZE, TILE_WIDTH,
};
use graphics::rectangle::Shape;
use piston_window::*;

use crate::{import::assets::SpeciesLibrary, ui::font::Font};

#[derive(PartialEq)]
struct WindowMessage<T> {
    content: T,
    timer: u128,
}

pub trait TextWindowFormat {
    fn format(&self) -> String;
    fn position(&self) -> [u32; 2];
    fn offset(&self, species_lib: &SpeciesLibrary) -> [f64; 2];
}

pub struct TextWindow<T>
where
    T: TextWindowFormat + PartialEq,
{
    font_size: u32,
    max_width: f64,
    messages: Vec<WindowMessage<T>>,
    margin: Size,
    species_lib: SpeciesLibrary,
    timer: u128,
}

impl<T> TextWindow<T>
where
    T: TextWindowFormat + PartialEq,
{
    pub fn new(font_size: u32, max_width: f64, timer: u128) -> Self {
        Self {
            font_size,
            max_width,
            timer,
            species_lib: SpeciesLibrary::new(),
            messages: Vec::default(),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    fn get_font_height(&self, font_size: u32, font: &mut Font) -> f64 {
        font.get()
            .character(font_size, '|')
            .map_or(0.0, |c| c.top())
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        let _ = self
            .messages
            .iter()
            .rev()
            .filter(|msg| msg.timer > 0)
            .map(|msg| {
                let text = msg.content.format();
                let model_position = msg.content.position();
                let text_width = self.max_width - CHAT_WINDOW_PADDING_WIDTH;

                let window_height =
                    font.height_with_auto_newline(text.as_str(), self.font_size, text_width)
                        + CHAT_WINDOW_PADDING_HEIGHT;

                // Compute the chat window position
                let window_position = [
                    model_position[0] as f64 + self.margin.width - (TILE_WIDTH / 2) as f64,
                    model_position[1] as f64 + self.margin.height
                        - msg.content.offset(&self.species_lib)[1]
                        - window_height // Need to offset with the rectangle height, therefore we control the bottom right anchor
                        - self.get_font_height(GUI_ENTITY_FONT_SIZE, font)
                        - CHAT_WINDOW_MARGIN_BOTTOM,
                ];

                // Compute the message position inside the chat window
                let msg_position = [
                    window_position[0] + (CHAT_WINDOW_PADDING_WIDTH / 2.0),
                    window_position[1]
                        + self.get_font_height(self.font_size, font)
                        + (CHAT_WINDOW_PADDING_HEIGHT / 2.0),
                ];

                // Prepare the chat window rectangle (position + size)
                let window_box = [
                    window_position[0],
                    window_position[1],
                    self.max_width,
                    window_height,
                ];
                window.draw_2d(evnt, |_ctx, gl, _device| {
                    Rectangle::new([1.0; 4])
                        .color(color::alpha(0.5))
                        .shape(Shape::Round(5.0, 32))
                        .draw(window_box, &_ctx.draw_state, _ctx.transform, gl);
                });
                font.render_with_auto_newline(
                    text.as_str(),
                    self.font_size,
                    evnt,
                    window,
                    color::BLACK,
                    msg_position,
                    self.max_width - CHAT_WINDOW_PADDING_WIDTH,
                    window_box,
                );
            })
            .collect::<Vec<_>>();
    }

    pub fn add_message<F>(&mut self, content: T, aggregate: F)
    where
        F: Fn(&T) -> bool,
    {
        let new_msg = WindowMessage {
            content,
            timer: self.timer,
        };

        let message_missing = self
            .messages
            .iter()
            .filter_map(|msg| msg.content.eq(&new_msg.content).then(|| true))
            .collect::<Vec<_>>()
            .is_empty();

        if message_missing {
            let agg_messages = self
                .messages
                .iter()
                .enumerate()
                .filter_map(|(index, msg)| (aggregate)(&msg.content).then(|| (index, msg.timer)))
                .collect::<Vec<_>>();

            // Overwrite the existing WindowMessage for this aggregation
            if agg_messages.len().ge(&1) {
                for agg_msg in agg_messages.iter() {
                    self.messages.remove(agg_msg.0);
                }
            }
            self.messages.push(new_msg);
        }
    }

    pub fn retain_message<F>(&mut self, retain_content: F)
    where
        F: Fn(&T) -> bool,
    {
        self.messages.retain(|msg| (retain_content)(&msg.content));
    }

    pub fn update<F>(&mut self, delta_ts: u128, mut update_content: F)
    where
        F: FnMut(&mut T),
    {
        //  drop all WindowMessage when their timer == 0
        self.messages.retain(|msg| msg.timer > 0);

        //  decrement all WindowMessage.timer with delta_ts
        let _: Vec<_> = self
            .messages
            .iter_mut()
            .map(|msg| {
                msg.timer = msg.timer.saturating_sub(delta_ts);
                update_content(&mut msg.content);
            })
            .collect();
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }
}
