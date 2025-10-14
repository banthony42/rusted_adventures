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
        // For each timer in rolling list that are not == 0
        //      Find a character position for the associated message
        //      Draw a box for this message and display it above the character with the found position

        let _ = self
            .messages
            .iter()
            .filter(|msg| msg.timer > 0)
            .map(|msg| {
                let font_size = 17;
                let window_max_width: f64 = 128.0;
                let text = msg.content.format();
                let model_position = msg.content.position();

                let line_height = font.text_height_for_max_width(
                    text.as_str(),
                    font_size,
                    window_max_width - 8.0,
                ) as f64;

                let bg_rect: [f64; 4] = [
                    model_position[0] as f64 + self.margin.width,
                    model_position[1] as f64 + self.margin.height
                        - msg.content.offset(&self.species_lib)[1]
                        - line_height,
                    window_max_width,
                    line_height,
                ];

                let msg_position = [bg_rect[0], bg_rect[1] + bg_rect[3] + 10.0 - line_height];

                window.draw_2d(evnt, |_ctx, gl, _device| {
                    Rectangle::new([1.0; 4])
                        .color(color::alpha(0.75))
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
                    bg_rect.map(|v| v as u32),
                );
            })
            .collect::<Vec<_>>();
    }

    pub fn add_message(&mut self, content: T) {
        let new_msg = WindowMessage {
            content,
            timer: 5000,
        };
        let found: Vec<_> = self
            .messages
            .iter()
            .filter_map(|msg| msg.content.eq(&new_msg.content).then_some(true))
            .collect();

        if found.len() == 0 {
            self.messages.push(new_msg);
        }
    }

    pub fn retain_message<F>(&mut self, mut retain_rule: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.messages.retain(|msg| (retain_rule)(&msg.content));
    }

    pub fn update<F>(&mut self, delta_ts: u128, mut update_rule: F)
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
                update_rule(&mut msg.content);
            })
            .collect();
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }
}
