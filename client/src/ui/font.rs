use graphics::types::Color;
use piston_window::*;
use std::path::PathBuf;

pub struct Font {
    font: Option<Glyphs>,
    size: u32,
    color: Color,
}

enum TextAlign {
    Centered,
    Left,
}

pub struct FontCharOffset {
    pub top: f64,
    pub left: f64,
}

impl FontCharOffset {
    fn new(top: f64, left: f64) -> Self {
        Self { top, left }
    }
}

impl Font {
    pub fn new(size: u32) -> Font {
        Font {
            font: None,
            size,
            color: color::BLACK,
        }
    }

    pub fn load(mut self, window: &mut PistonWindow) -> Self {
        let font_path = PathBuf::from("assets/fonts/OpenSans_Condensed-SemiBold.ttf");
        self.font = Some(
            window
                .load_font(font_path)
                .expect("Engine can't run without font."),
        );
        self
    }

    pub fn get(&mut self) -> &mut Glyphs {
        self.font
            .as_mut()
            .expect("Font should be loaded (with `load` function) before accessing it.")
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn set_size(&mut self, size: u32) -> &mut Self {
        self.size = size;
        self
    }

    pub fn width(&mut self, text: &str) -> Option<f64> {
        let size = self.size;
        match self.get().width(size, text) {
            Ok(w) => Some(w),
            Err(e) => {
                println!("Error trying to get font width for: '{text}' : {e}");
                None
            }
        }
    }

    pub fn char_offset(&mut self, ch: char) -> Option<FontCharOffset> {
        let size = self.size;
        match self.get().character(size, ch) {
            Ok(ch) => Some(FontCharOffset::new(ch.top(), ch.left())),
            Err(e) => {
                println!("Error trying to get font top offset for: '{ch}' : {e}");
                None
            }
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) -> &mut Self {
        self.color = color;
        self
    }

    /// Render text center aligned
    pub fn render_centered(
        &mut self,
        text: &str,
        evnt: &Event,
        window: &mut PistonWindow,
        pos: [f64; 2],
        margin: &Size,
    ) {
        self.render_text(text, evnt, window, pos, margin, TextAlign::Centered)
    }

    /// Render text left aligned
    pub fn render_left_aligned(
        &mut self,
        text: &str,
        evnt: &Event,
        window: &mut PistonWindow,
        pos: [f64; 2],
        margin: &Size,
    ) {
        self.render_text(text, evnt, window, pos, margin, TextAlign::Left)
    }

    fn render_text(
        &mut self,
        text: &str,
        evnt: &Event,
        window: &mut PistonWindow,
        pos: [f64; 2],
        margin: &Size,
        alignment: TextAlign,
    ) {
        let mut x = pos[0];
        let y = pos[1];
        let size = self.size;
        let Ok(text_width) = self.get().width(size, text) else {
            println!("Font: render text: fail to get width.");
            return;
        };

        match alignment {
            TextAlign::Centered => x -= text_width / 2.0,
            TextAlign::Left => { /* By design */ }
        };

        let text_split_by_newline: Vec<&str> = text.split("\n").collect();

        window.draw_2d(evnt, |ctx, gl, _| {
            text_split_by_newline
                .iter()
                .enumerate()
                .for_each(|(index, text)| {
                    let _ = text::Text::new_color(self.color, size).draw(
                        text,
                        self.get(),
                        &ctx.draw_state,
                        ctx.transform.trans(
                            margin.width + x,
                            margin.height + y + (index * size as usize) as f64,
                        ),
                        gl,
                    );
                });
        });
    }

    fn split_into_lines(&mut self, text: &str, max_text_width: f64) -> Vec<String> {
        // Browse the text String char by char, computing final text width in pixel.
        // Each time the width exceed the `max_text_width` we extract all browsed chars into a string, (with String::drain)
        // And we store the extraction in a Vector
        // At the end we land with Vec<String> with each string will not exceed `max_text_width` pixel

        let mut row_width = 0.0;
        let mut row = Vec::new();
        let size = self.size;
        let mut text_split: Vec<String> = Vec::new();
        let text_by_words: Vec<&str> = text.split_whitespace().collect();
        let space_width = self
            .get()
            .character(size, ' ')
            .map_or(0.0, |ch| ch.advance_width());

        for word in text_by_words.iter() {
            let Ok(mut word_width) = self.get().width(size, word) else {
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
                        if !first.is_empty() {
                            text_split.push(first.to_string());
                        }
                        if n == nb_split - 1 && !second.is_empty() {
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
        text_split.retain(|txt| !txt.is_empty());
        text_split
    }

    /// Compute the total height of lines, the text will use
    /// to get render with ```render_with_auto_newline```
    pub fn height_with_auto_newline(&mut self, text: &str, line_width: f64) -> f64 {
        self.split_into_lines(text, line_width).len() as f64 * self.size as f64
    }

    /// Render text with automatic jump to newline, a maximum width for the line is needed.
    ///
    /// ```height_with_auto_newline``` can be used to know in advance the total height of lines.
    pub fn render_with_auto_newline(
        &mut self,
        text: &str,
        evnt: &Event,
        window: &mut PistonWindow,
        pos: [f64; 2],
        line_width: f64,
        scissor: [f64; 4],
    ) {
        let x = pos[0];
        let y = pos[1];
        let size = self.size;
        let lines = self.split_into_lines(text, line_width);

        window.draw_2d(evnt, |ctx, gl, _| {
            let _: Vec<_> = lines
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    let _ = text::Text::new_color(self.color, size).draw(
                        text,
                        self.get(),
                        &DrawState::default().scissor(scissor.map(|value| value as u32)),
                        ctx.transform.trans(x, y + (index * size as usize) as f64),
                        gl,
                    );
                })
                .collect();
        });
    }
}
