use piston_window::rectangle::*;
use piston_window::*;

use super::font::Font;

// #[derive(Clone)]
pub struct InputField {
    focus: bool,
    mouseover: bool,
    font: Font,
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
    pub fn new(pos: [f64; 2], font_size: u32, width: f64, window: &mut PistonWindow) -> Self {
        let rect = Rectangle::new([1.0; 4])
            .color(color::WHITE)
            .shape(Shape::Round(1.0, 32))
            .border(Border {
                color: color::TRANSPARENT,
                radius: 1.0,
            });

        let rect_settings = [pos[0], pos[1], width, font_size as f64 + 4.0];

        InputField {
            focus: false,
            mouseover: false,
            font: Font::new(font_size).load(window),
            rect_settings,
            content: String::new(),
            content_width: 0.0,
            cursor_timer: 0,
            cursor_hidden: false,
            rect,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            masked: false,
        }
    }

    pub fn font(&mut self) -> &mut Font {
        &mut self.font
    }

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn set_height(mut self, height: f64) -> Self {
        self.rect_settings[3] = (self.font.size() as f64 + 4.0).max(height);
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
            let s: String = self.content.chars().map(|_| "•").collect();
            return s;
        }
        self.content.clone()
    }

    fn erase_last_char(&mut self) {
        if let Some(char) = self.content.pop() {
            if let Some(text_width) = self.font.width(&char.to_string()) {
                self.content_width -= text_width;
            }
        }
        // Should never happend
        if self.content_width < 0.0 {
            self.content_width = 0.0;
        }
    }

    fn add_text(&mut self, text: &str) {
        if let Some(text_width) = self.font.width(text) {
            let max_content_width = self.rect_settings[2] - 4.0;
            if self.content_width + text_width < max_content_width {
                self.content.push_str(text);
                self.content_width += text_width;
            }
        }
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            let mut final_position = self.rect_settings;
            final_position[0] += self.margin.width;
            final_position[1] += self.margin.height;
            self.rect
                .draw(final_position, &_ctx.draw_state, _ctx.transform, gl);
        });

        // Assume top offset is the same for all characters, we choose '|' as char model to get the height of char
        // This height is use below to center the text in the text field Rectangle and stick the cursor to the text
        if !self.content.is_empty() {
            if let Some(char_offset) = self.font.char_offset('|') {
                let text_pos = [
                    self.rect_settings[0],
                    self.rect_settings[1]
                        + char_offset.top
                        + (self.rect_settings[3] - self.font.size() as f64) / 2.0,
                ];

                self.font.render_left_aligned(
                    self.get_content_or_masked().as_str(),
                    evnt,
                    window,
                    text_pos,
                    &self.margin,
                );
            }
        }

        // Render blink cursor
        match self.cursor_hidden {
            true => { /* Don't render the cursor, should be hidden */ }
            false => {
                if self.focus {
                    if let Some(text_width) = self.font.width(self.get_content_or_masked().as_str())
                    {
                        if let Some(char_offset) = self.font.char_offset('|') {
                            let cursor_pos = [
                                self.rect_settings[0] + text_width - char_offset.left,
                                self.rect_settings[1]
                                    + char_offset.top
                                    + (self.rect_settings[3] - self.font.size() as f64) / 2.0,
                            ];
                            self.font.render_left_aligned(
                                "|",
                                evnt,
                                window,
                                cursor_pos,
                                &self.margin,
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

    pub fn text_input(&mut self, args: &str) {
        if self.focus {
            self.add_text(args);
        }
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        let mouse_x = args[0];
        let mouse_y = args[1];
        let rect_left = self.rect_settings[0] + self.margin.width;
        let rect_top = self.rect_settings[1] + self.margin.height;
        let rect_right = rect_left + self.rect_settings[2];
        let rect_bottom = rect_top + self.rect_settings[3];

        if mouse_x > rect_left
            && mouse_x < rect_right
            && mouse_y > rect_top
            && mouse_y < rect_bottom
        {
            self.mouseover = true;
            return;
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

        if self.focus {
            if let Button::Keyboard(Key::Backspace) = args {
                self.erase_last_char();
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = *margin;
    }
}
