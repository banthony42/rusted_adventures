use piston_window::*;
use std::path::PathBuf;

pub struct Font {
    font: Option<Glyphs>,
}

#[derive(Debug)]
pub enum FontError {
    CharacterPreload,
}

enum TextAlign {
    Centered,
    Left,
}

impl Font {
    pub fn new() -> Font {
        Font { font: None }
    }

    pub fn load(&mut self, window: &mut PistonWindow) {
        // let font_path = PathBuf::from("../assets/fonts/OpenSans-Regular.ttf");
        // let font_path = PathBuf::from("../assets/fonts/OpenSans_Condensed-Bold.ttf");
        // let font_path = PathBuf::from("../assets/fonts/dejavu-sans.book.ttf");
        // let font_path = PathBuf::from("../assets/fonts/MedievalSharp-Bold.ttf");
        // let font_path = PathBuf::from("../assets/fonts/MedievalSharp-Book.ttf");
        let font_path = PathBuf::from("../assets/fonts/OpenSans_Condensed-SemiBold.ttf");
        self.font = Some(
            Glyphs::new(
                font_path,
                window.create_texture_context(),
                TextureSettings::new(),
            )
            .unwrap(),
        );
    }

    pub fn get(&mut self) -> &mut Glyphs {
        self.font.as_mut().unwrap()
    }

    pub fn render_text_centered(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        margin: Option<&Size>,
    ) {
        self.__render_text(
            text,
            font_size,
            evnt,
            window,
            color,
            pos,
            margin,
            TextAlign::Centered,
        )
    }

    pub fn render_text(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        margin: Option<&Size>,
    ) {
        self.__render_text(
            text,
            font_size,
            evnt,
            window,
            color,
            pos,
            margin,
            TextAlign::Left,
        )
    }

    fn __render_text(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        margin: Option<&Size>,
        alignment: TextAlign,
    ) {
        let mut x = pos[0];
        let y = pos[1];
        let text_width = self.get().width(font_size, text).unwrap();

        match alignment {
            TextAlign::Centered => x -= text_width / 2.0,
            TextAlign::Left => { /* By design */ }
        };

        let mut width_cursor = 0.0;
        let mut newline = 0;
        let text_split_by_newline: Vec<&str> = text.split("\n").collect();

        let final_margin = match margin {
            Some(m) => m.clone(),
            None => Size {
                width: 0.0,
                height: 0.0,
            },
        };

        window.draw_2d(evnt, |ctx, gl, device| {
            let _: Vec<_> = text_split_by_newline
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    width_cursor += self.get().width(font_size, text).unwrap();
                    if width_cursor > text_width {
                        newline += 1;
                    }

                    let _ = text::Text::new_color(color, font_size).draw(
                        text,
                        self.get(),
                        &ctx.draw_state,
                        ctx.transform.trans(
                            final_margin.width + x as f64,
                            final_margin.height
                                + y as f64
                                + ((index + newline) * font_size as usize) as f64,
                        ),
                        gl,
                    );
                    self.font.as_mut().unwrap().factory.encoder.flush(device);
                })
                .collect();
        });
    }

    pub fn get_text_render_size(
        &mut self,
        font_size: u32,
        text: &str,
    ) -> Result<[f64; 2], FontError> {
        let mut x = 0.0;
        let mut y = 0.0;
        for ch in text.chars() {
            match self.get().character(font_size, ch) {
                Ok(character) => {
                    x += character.advance_width();
                    if character.top() > y {
                        y = character.top();
                    }
                }
                Err(_) => {
                    return Err(FontError::CharacterPreload);
                }
            }
        }
        Ok([x, y])
    }

    pub fn text_height_for_max_width(&mut self, text: &str, font_size: u32, max_width: f64) -> u32 {
        let mut width_cursor = 0.0;
        let mut newlines = 1;

        for char in text.chars() {
            if let Ok(ch) = self.get().character(font_size, char) {
                width_cursor += ch.advance_width();
                if width_cursor > max_width {
                    width_cursor = 0.0;
                    newlines += 1;
                }
            }
        }
        return newlines * font_size;
    }

    pub fn render_text_max_width(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        max_text_width: f64,
        scissor: [u32; 4],
    ) {
        let x = pos[0];
        let y = pos[1];

        let mut width_cursor = 0.0;
        let mut final_text = text.to_string();

        for (ind, char) in text.chars().enumerate() {
            if let Ok(ch) = self.get().character(font_size, char) {
                width_cursor += ch.advance_width();
                if width_cursor > max_text_width {
                    width_cursor = 0.0;
                    let end = ind + 1;
                    final_text.replace_range(ind..end, "\n");
                }
            }
        }

        let text_split_by_newline: Vec<&str> = final_text.split("\n").collect();

        window.draw_2d(evnt, |ctx, gl, device| {
            let _: Vec<_> = text_split_by_newline
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    let _ = text::Text::new_color(color, font_size).draw(
                        text,
                        self.get(),
                        &DrawState::default().scissor(scissor),
                        ctx.transform
                            .trans(x as f64, y as f64 + (index * font_size as usize) as f64),
                        gl,
                    );
                    self.font.as_mut().unwrap().factory.encoder.flush(device);
                })
                .collect();
        });
    }
}
