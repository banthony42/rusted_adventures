use std::collections::HashMap;
use piston_window::*;
use types::Color;

use crate::{
    assets::HardTexture, client::GameData, constants, entity::{
        Entity,
        EntityType,
        Name
    }, game::Game, ui::font::Font, world::{
        Coord,
        MapData
    }
};

pub struct Interface {
    img: Image,
    player_color: Color,
    mob_color: Color
}

const UI_OVERLAY_FONT_SIZE: u32 = 17;

impl Interface {

    pub fn new() -> Self {
        return Interface {
            img: Image::new().rect([constants::GUI_WIDTH_CENTER as f64, constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]),
            // player_color: color::hex("7483e8"),
            // player_color: color::hex("6274e5"),
            player_color: color::hex("0017ad"),
            mob_color: color::hex("c5c9e8")
        }
    }

    pub fn render(&self, evnt : &Event, window: &mut PistonWindow, game: &Game) {
        window.draw_2d(evnt, |ctx, gl, _device| {
            self.img.draw(&game.hard_textures[&HardTexture::Interface] , &DrawState::default(), ctx.transform, gl);
        });
    }

    pub fn resize(&mut self, margin: &Size) {
        self.img = Image::new().rect([margin.width, margin.height + constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]);        
    }

    pub fn render_text_overlay(&mut self, evnt : &Event, window: &mut PistonWindow, font: &mut Font, margin: &Size, world: &HashMap<Coord, MapData>, game_data: &GameData) {
        let map_data = world.get(&game_data.player.world_coord).unwrap();
        let map_coord_txt = format!("{}\nCoordonnées: {}, {}", map_data.info, game_data.player.world_coord.x, game_data.player.world_coord.y);
        font.render_text(&map_coord_txt.as_str(), UI_OVERLAY_FONT_SIZE, evnt, window, color::WHITE, [5.0, 17.0], Some(margin));

        let mut render_entity_name = |entity: &Entity| {
            let name_width_with_font = font.get().width(UI_OVERLAY_FONT_SIZE, entity.get_name().as_str()).unwrap();

            let e_name_coord = [
                entity.map_coord.x as f64 * 64.0 + 32.0 - (name_width_with_font / 2.0),
                (entity.map_coord.y * 64) as f64 - 64.0
            ];
            let final_color = match entity.r#type {
                EntityType::Player => self.player_color,
                EntityType::Monster => self.mob_color,
            };
            font.render_text(entity.get_name().as_str(), 17, evnt, window, final_color, e_name_coord, Some(margin)); //TODO: implem resize method to get margin
        };

        render_entity_name(&game_data.player);
        for entity in game_data.entities.iter() {
            render_entity_name(entity);
        }
    }
}