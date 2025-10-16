use piston_window::*;
use std::path::PathBuf;

pub struct Font {
    font: Option<Glyphs>,
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

    /// Render text center aligned
    pub fn render_centered(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        margin: Option<&Size>,
    ) {
        self.render_text(
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

    /// Render text left aligned
    pub fn render_left_aligned(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        margin: Option<&Size>,
    ) {
        self.render_text(
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

    fn render_text(
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

    fn split_into_lines(&mut self, text: &str, font_size: u32, max_text_width: f64) -> Vec<String> {
        // Browse the text String char by char, computing final text width in pixel.
        // Each time the width exceed the `max_text_width` we extract all browsed chars into a string, (with String::drain)
        // And we store the extraction in a Vector
        // At the end we land with Vec<String> with each string will not exceed `max_text_width` pixel

        let mut row_width = 0.0;
        let mut row = Vec::new();
        let mut text_split: Vec<String> = Vec::new();
        let text_by_words: Vec<&str> = text.split_whitespace().collect();
        let space_width = self
            .get()
            .character(font_size, ' ')
            .map_or(0.0, |ch| ch.advance_width());

        for word in text_by_words.iter() {
            let Ok(mut word_width) = self.get().width(font_size, word) else {
                // Skip the word on any font error
                continue;
            };
            word_width += space_width;
            if row_width + word_width < max_text_width {
                row_width += word_width;
                row.push(word.to_string());
            } else {
                // The row is large enough, push it and clear the row
                text_split.push(row.join(" "));
                row.clear();
                row_width = 0.0;
                // Word too large for one row, we have to split the word
                if word_width > max_text_width {
                    let nb_split: u8 = (word_width / max_text_width).ceil() as u8;
                    let part_len = word.len() / nb_split as usize;
                    let mut splitter: &str = word;
                    for n in 0..nb_split {
                        let (first, second) = splitter.split_at(part_len);
                        if first.len() > 0 {
                            text_split.push(first.to_string());
                        }
                        if n == nb_split - 1 && second.len() > 0 {
                            text_split.push(second.to_string());
                        } else {
                            splitter = second;
                        }
                    }
                } else {
                    // The word can be added to the row, we are still under max width
                    row.push(word.to_string());
                    row_width = word_width;
                }
            }
        }
        // Don't forget to push the remaining text
        if !row.is_empty() {
            text_split.push(row.join(" "));
        }
        text_split.retain(|txt| txt.len() > 0);
        text_split
    }

    /// Compute the total height of lines, the text will use
    /// to get render with ```render_with_auto_newline```
    pub fn height_with_auto_newline(&mut self, text: &str, font_size: u32, line_width: f64) -> f64 {
        self.split_into_lines(text, font_size, line_width).len() as f64 * font_size as f64
    }

    /// Render text with automatic jump to newline, a maximum width for the line is needed.
    ///
    /// ```height_with_auto_newline``` can be used to know in advance the total height of lines.
    pub fn render_with_auto_newline(
        &mut self,
        text: &str,
        font_size: u32,
        evnt: &Event,
        window: &mut PistonWindow,
        color: [f32; 4],
        pos: [f64; 2],
        line_width: f64,
        scissor: [f64; 4],
    ) {
        let x = pos[0];
        let y = pos[1];
        let lines = self.split_into_lines(text, font_size, line_width);

        window.draw_2d(evnt, |ctx, gl, device| {
            let _: Vec<_> = lines
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    let _ = text::Text::new_color(color, font_size).draw(
                        text,
                        self.get(),
                        &DrawState::default().scissor(scissor.map(|value| value as u32)),
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
