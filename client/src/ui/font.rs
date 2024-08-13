use std::{error::Error, path::PathBuf};
use piston_window::*;

pub struct Font {
    font: Option<Glyphs>,
}

#[derive(Debug)]
pub enum FontError {
    CharacterPreload
}

impl Font {

    pub fn new() -> Font {
        Font {
            font: None,
        }
    }

    pub fn load(&mut self, window: &mut PistonWindow) {
        // let font_path = PathBuf::from("../assets/fonts/OpenSans-Regular.ttf");
        // let font_path = PathBuf::from("../assets/fonts/OpenSans_Condensed-Bold.ttf");
        // let font_path = PathBuf::from("../assets/fonts/dejavu-sans.book.ttf");
        // let font_path = PathBuf::from("../assets/fonts/MedievalSharp-Bold.ttf");
        // let font_path = PathBuf::from("../assets/fonts/MedievalSharp-Book.ttf");
        let font_path = PathBuf::from("../assets/fonts/OpenSans_Condensed-SemiBold.ttf");
        self.font = Some(Glyphs::new(font_path, window.create_texture_context(), TextureSettings::new()).unwrap());
    }

    pub fn get(&mut self) -> &mut Glyphs {
        self.font.as_mut().unwrap()
    }

    pub fn get_text_render_size(&mut self, font_size: u32, text: &str) -> Result<[f64; 2], FontError> {
        let mut x = 0.0;
        let mut y = 0.0;
        for ch in text.chars() {
            match self.get().character(font_size, ch) {
                Ok(character) => {
                    x += character.advance_width();
                    if character.top() > y {
                        y = character.top();
                    }
                },
                Err(e) => { return Err(FontError::CharacterPreload); }
            }
        }
        Ok([x, y])
    }

    pub fn render_text(&mut self, text: &str, font_size: u32, evnt : &Event, window: &mut PistonWindow, color: [f32;4], pos: [f64; 2], margin: Option<&Size>) {
        let x = pos[0];
        let y = pos[1];
        let text_split_by_newline : Vec<&str> = text.split("\n").collect();

        let final_margin = match margin {
            Some(m) => m.clone(),
            None => Size { width: 0.0, height: 0.0}
        };

        window.draw_2d(evnt, |ctx, gl, device| {
            let _: Vec<_> = text_split_by_newline.iter().enumerate().map(|(index, text)| {
                let _ = text::Text::new_color(color, font_size).draw(
                    text,
                    self.get(),
                    &ctx.draw_state,
                    ctx.transform.trans(final_margin.width + x as f64, final_margin.height + y as f64 + (index * font_size as usize) as f64 ), gl
                );
                self.font.as_mut().unwrap().factory.encoder.flush(device);
            }).collect();
        });
    }
}