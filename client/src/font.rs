use std::path::PathBuf;
use piston_window::*;

use crate::world::Coord;

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

    pub fn render_text(&mut self, text: &str, evnt : &Event, window: &mut PistonWindow, color: [f32;4], pos: Coord, margin: &Size) {
        let texts : Vec<&str> = text.split("\n").collect();

        window.draw_2d(evnt, |ctx, gl, device| {
            let _: Vec<_> = texts.iter().enumerate().map(|(index, text)| {
                let _ = text::Text::new_color(color, 17).draw(
                    text,
                    self.get(),
                    &ctx.draw_state,
                    ctx.transform.trans(margin.width + pos.x as f64, margin.height + pos.y as f64 + (index * 17) as f64 ), gl
                );
                self.font.as_mut().unwrap().factory.encoder.flush(device);
            }).collect();
        });
    }
}