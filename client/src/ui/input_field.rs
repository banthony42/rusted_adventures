use piston_window::*;
use piston_window::rectangle::*;

use super::font::Font;


// Move utils.rs in common lib easily accessible for ui folder modules

use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
}

pub struct InputField {
    focus: bool,
    mouseover: bool,
    font_size: u32,
    content: String,
    cursor_timer: u128,
    cursor_hidden: bool,
    rect: Rectangle,
    rect_settings: [f64; 4],
    margin: Size,
}

impl InputField {

    pub fn new(pos: [f64; 2], font_size: u32, width: f64) -> Self {
        let rect = Rectangle::new([1.0; 4])
        .color(color::WHITE)
        .shape(Shape::Round(1.0, 32))
        .border(Border {
            color: color::TRANSPARENT,
            radius: 1.0,
        });

        let rect_settings = [
            pos[0],
            pos[1],
            width as f64,
            font_size as f64 + 4.0
        ];

        InputField {
            focus: false,
            mouseover: false,
            font_size: font_size,
            rect_settings: rect_settings,
            content: String::new(),
            cursor_timer: 0,
            cursor_hidden: false,
            rect: rect,
            margin: Size { width: 0.0, height: 0.0 },
        }
    }

    pub fn clean(&mut self) {
        self.content.clear()
    }

    pub fn get_content(&self) -> String {
        self.content.trim().to_string()
    }

    pub fn render(&self, evnt : &Event, window: &mut PistonWindow, font: &mut Font) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            let mut final_position = self.rect_settings.clone();
            final_position[0] += self.margin.width;
            final_position[1] += self.margin.height;
            self.rect.draw(final_position, &_ctx.draw_state, _ctx.transform, gl);
        });

        // Assume top offset is the same for all characters, we choose '|' as char model to get the height of char
        // This height is use below to center the text in the text field Rectangle and stick the cursor to the text
        if self.content.len() > 0 {
            if let Ok(char_template) = font.get().character(self.font_size, '|') {
                let text_pos = [
                    self.rect_settings[0],
                    self.rect_settings[1] + char_template.top() + (self.rect_settings[3] - self.font_size as f64) / 2.0
                ];
                font.render_text(self.content.as_str(), self.font_size, evnt, window, color::BLACK, text_pos, Some(&self.margin));
            }
        }

        // Render blink cursor
        match self.cursor_hidden {
            true => { /* Don't render the cursor, should be hidden */ },
            false => {
                if self.focus {
                    if let Ok(content_size) = font.get_text_render_size(self.font_size, self.content.as_str()) {
                        if let Ok(char_template) = font.get().character(self.font_size, '|') {
                            let cursor_pos = [
                                self.rect_settings[0] + content_size[0] - char_template.left(),
                                self.rect_settings[1] + char_template.top() + (self.rect_settings[3] - self.font_size as f64) / 2.0
                            ];
                            font.render_text("|", self.font_size, evnt, window, color::BLACK, cursor_pos, Some(&self.margin))
                        }
                    }
                }
            },
        }
    }

    pub fn update(&mut self, delta_ts: u128) {
        if self.focus {
            self.cursor_timer += delta_ts;
            if self.cursor_timer >= 600 {
                self.cursor_timer = 0;
                self.cursor_hidden = !self.cursor_hidden;
            }
        }
    }

    pub fn text_input(&mut self, args: String) {
        if self.focus {
            self.content.push_str(args.as_str());
        }
    }

    pub fn mouse_cursor_args(&mut self, args: [f64; 2]) {
        let mouse_x = args[0];
        let mouse_y = args[1];
        let rect_left = self.rect_settings[0] + self.margin.width;
        let rect_top = self.rect_settings[1] + self.margin.height;
        let rect_right= rect_left + self.rect_settings[2];
        let rect_bottom = rect_top + self.rect_settings[3];

        if mouse_x > rect_left && mouse_x < rect_right {
            if mouse_y > rect_top && mouse_y < rect_bottom {
                self.mouseover = true;
                return
            }
        }
        self.mouseover = false;
    }
        
    pub fn key_press(&mut self, args: &Button) {

        if let Button::Mouse(MouseButton::Left) = args {
            match self.mouseover {
                true => self.focus = true,
                false => {
                    self.focus = false;
                    self.cursor_hidden = false;
                    self.cursor_timer = 0;
                }
            }
        }

        if let Button::Keyboard(Key::Backspace) = args {
            if self.content.len() > 0 {
                let _ = self.content.drain(self.content.len() - 1 .. self.content.len());
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }
}