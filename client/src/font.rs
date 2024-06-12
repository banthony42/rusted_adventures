use std::path::PathBuf;
use piston_window::{Glyphs, PistonWindow, TextureSettings};

pub struct Font {
    font: Option<Glyphs>
}

impl Font {

    pub fn new() -> Font {
        Font {
            font: None
        }
    }

    pub fn load(&mut self, window: &mut PistonWindow) {
        let font_path = PathBuf::from("../assets/fonts/OpenSans_Condensed-SemiBold.ttf");
        self.font = Some(Glyphs::new(font_path, window.create_texture_context(), TextureSettings::new()).unwrap());
    }

    pub fn get(&mut self) -> &mut Glyphs {
        self.font.as_mut().unwrap()
    }
}