use common::constants::TILE_WIDTH;
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
    messages: Vec<WindowMessage<T>>,
    margin: Size,
    species_lib: SpeciesLibrary,
}

impl<T> TextWindow<T>
where
    T: TextWindowFormat + PartialEq,
{
    pub fn new() -> Self {
        Self {
            species_lib: SpeciesLibrary::new(),
            messages: Vec::default(),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        let mut msg_raw = 0.0;
        let _ = self
            .messages
            .iter()
            .rev()
            .filter(|msg| msg.timer > 0)
            .map(|msg| {
                let font_size = 17;
                let window_max_width: f64 = 128.0;
                let padding_height = 10.0;
                let padding_width = 10.0;
                let bg_rect_padding = 2.0;
                let entity_name_offset = 20.0;
                let text = msg.content.format();
                let model_position = msg.content.position();

                let line_height =
                    font.text_height_for_max_width(text.as_str(), font_size, window_max_width)
                        as f64;

                let Ok(char_template) = font.get().character(font_size, '|') else {
                    return;
                };

                let bg_height = line_height + padding_height;
                let bg_rect: [f64; 4] = [
                    model_position[0] as f64 + self.margin.width - TILE_WIDTH as f64 / 2.0,
                    model_position[1] as f64 + self.margin.height
                        - msg.content.offset(&self.species_lib)[1]
                        - bg_height // Need to offset with the rectangle height, therefore we control the bottom right anchor
                        - entity_name_offset
                        - msg_raw,
                    window_max_width + padding_width,
                    bg_height,
                ];
                let mut scissor = bg_rect.map(|v| v as u32);
                scissor[3] += msg_raw as u32;
                let msg_position = [
                    bg_rect[0] + (padding_width / 2.0),
                    bg_rect[1] + char_template.top() + (padding_height / 2.0),
                ];

                window.draw_2d(evnt, |_ctx, gl, _device| {
                    Rectangle::new([1.0; 4])
                        .color(color::alpha(0.5))
                        .shape(Shape::Round(5.0, 32))
                        .draw(bg_rect, &_ctx.draw_state, _ctx.transform, gl);
                });
                font.render_text_max_width(
                    text.as_str(),
                    font_size,
                    evnt,
                    window,
                    color::BLACK,
                    msg_position,
                    window_max_width,
                    scissor,
                );
                msg_raw += bg_height + bg_rect_padding;
            })
            .collect::<Vec<_>>();
    }

    pub fn add_message<F>(&mut self, content: T, aggregate: F)
    where
        F: Fn(&T) -> bool,
    {
        let new_msg = WindowMessage {
            content,
            timer: 8000,
        };

        let message_missing = self
            .messages
            .iter()
            .filter_map(|msg| msg.content.eq(&new_msg.content).then(|| true))
            .collect::<Vec<_>>()
            .is_empty();

        if message_missing {
            let mut agg_messages = self
                .messages
                .iter()
                .enumerate()
                .filter_map(|(index, msg)| (aggregate)(&msg.content).then(|| (index, msg.timer)))
                .collect::<Vec<_>>();
            // Sort by timer
            agg_messages.sort_by(|a, b| a.1.cmp(&b.1));

            // If the limit is reached for this aggregation
            // Just remove the oldest timer to make some space
            if agg_messages.len().ge(&3) {
                self.messages.remove(agg_messages[0].0);
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
