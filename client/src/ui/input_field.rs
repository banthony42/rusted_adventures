use piston_window::rectangle::*;
use piston_window::*;

use super::font::Font;

#[derive(Clone)]
pub struct InputField {
    focus: bool,
    mouseover: bool,
    font_size: u32,
    content: String,
    content_width: f64,
    cursor_timer: u128,
    cursor_hidden: bool,
    rect: Rectangle,
    rect_settings: [f64; 4],
    margin: Size,
    masked: bool,
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

        let rect_settings = [pos[0], pos[1], width as f64, font_size as f64 + 4.0];

        InputField {
            focus: false,
            mouseover: false,
            font_size: font_size,
            rect_settings: rect_settings,
            content: String::new(),
            content_width: 0.0,
            cursor_timer: 0,
            cursor_hidden: false,
            rect: rect,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            masked: false,
        }
    }

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn set_height(mut self, height: f64) -> Self {
        self.rect_settings[3] = (self.font_size as f64 + 4.0).max(height) as f64;
        self
    }

    pub fn set_radius(mut self, radius: f64) -> Self {
        self.rect = self.rect.shape(Shape::Round(radius, 32));
        self
    }

    pub fn get_content(&self) -> String {
        self.content.trim().to_string()
    }

    pub fn is_focus(&self) -> bool {
        self.focus
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus
    }

    pub fn clean(&mut self) {
        self.content.clear();
        self.content_width = 0.0;
    }

    fn get_content_or_masked(&self) -> String {
        if self.masked {
            let s: String = self
                .content
                .chars()
                .map(|x| match x {
                    _ => "•",
                })
                .collect();
            return s;
        }
        return self.content.clone();
    }

    fn erase_last_char(&mut self, font: &mut Font) {
        if let Some(char) = self.content.pop() {
            if let Ok(text_width) = font.get().width(self.font_size, &char.to_string()) {
                self.content_width -= text_width;
            }
        }
        // Should never happend
        if self.content_width < 0.0 {
            self.content_width = 0.0;
        }
    }

    fn add_text(&mut self, text: &str, font: &mut Font) {
        match font.get().width(self.font_size, text) {
            Ok(text_width) => {
                let max_content_width = self.rect_settings[2] - 4.0;
                if self.content_width + text_width < max_content_width {
                    self.content.push_str(text);
                    self.content_width += text_width;
                }
            }
            Err(_) => {}
        }
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, font: &mut Font) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            let mut final_position = self.rect_settings.clone();
            final_position[0] += self.margin.width;
            final_position[1] += self.margin.height;
            self.rect
                .draw(final_position, &_ctx.draw_state, _ctx.transform, gl);
        });

        // Assume top offset is the same for all characters, we choose '|' as char model to get the height of char
        // This height is use below to center the text in the text field Rectangle and stick the cursor to the text
        if self.content.len() > 0 {
            if let Ok(char_template) = font.get().character(self.font_size, '|') {
                let text_pos = [
                    self.rect_settings[0],
                    self.rect_settings[1]
                        + char_template.top()
                        + (self.rect_settings[3] - self.font_size as f64) / 2.0,
                ];
                font.render_left_aligned(
                    self.get_content_or_masked().as_str(),
                    self.font_size,
                    evnt,
                    window,
                    color::BLACK,
                    text_pos,
                    Some(&self.margin),
                );
            }
        }

        // Render blink cursor
        match self.cursor_hidden {
            true => { /* Don't render the cursor, should be hidden */ }
            false => {
                if self.focus {
                    if let Ok(text_width) = font
                        .get()
                        .width(self.font_size, self.get_content_or_masked().as_str())
                    {
                        if let Ok(char_template) = font.get().character(self.font_size, '|') {
                            let cursor_pos = [
                                self.rect_settings[0] + text_width - char_template.left(),
                                self.rect_settings[1]
                                    + char_template.top()
                                    + (self.rect_settings[3] - self.font_size as f64) / 2.0,
                            ];
                            font.render_left_aligned(
                                "|",
                                self.font_size,
                                evnt,
                                window,
                                color::BLACK,
                                cursor_pos,
                                Some(&self.margin),
                            )
                        }
                    }
                }
            }
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

    pub fn text_input(&mut self, args: &String, font: &mut Font) {
        if self.focus {
            self.add_text(args.as_str(), font);
        }
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        let mouse_x = args[0];
        let mouse_y = args[1];
        let rect_left = self.rect_settings[0] + self.margin.width;
        let rect_top = self.rect_settings[1] + self.margin.height;
        let rect_right = rect_left + self.rect_settings[2];
        let rect_bottom = rect_top + self.rect_settings[3];

        if mouse_x > rect_left && mouse_x < rect_right {
            if mouse_y > rect_top && mouse_y < rect_bottom {
                self.mouseover = true;
                return;
            }
        }
        self.mouseover = false;
    }

    pub fn key_press(&mut self, args: &Button, font: &mut Font) {
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

        if self.focus {
            if let Button::Keyboard(Key::Backspace) = args {
                self.erase_last_char(font);
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }
}
