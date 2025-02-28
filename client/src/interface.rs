use piston_window::*;
use types::Color;

use crate::{
    client::FakeGameData,
    constants::{self, PLAYER_HEIGHT},
    entity::{Entity, EntityType, Name},
    import::assets::HardTexture,
    states::game::Game,
    ui::font::Font,
};

pub struct Interface {
    img: Image,
    player_color: Color,
    mob_color: Color,
    delta_ts: u128,
    fps: i16,
    timer: u128,
}

const UI_OVERLAY_FONT_SIZE: u32 = 17;

impl Interface {
    pub fn new() -> Self {
        return Interface {
            timer: 0,
            fps: 0,
            delta_ts: 0,
            img: Image::new().rect([
                constants::GUI_WIDTH_CENTER as f64,
                constants::MAP_HEIGHT as f64,
                constants::GUI_WIDTH as f64,
                constants::GUI_HEIGHT as f64,
            ]),
            // player_color: color::hex("7483e8"),
            // player_color: color::hex("6274e5"),
            player_color: color::hex("0017ad"),
            mob_color: color::hex("c5c9e8"),
        };
    }

    pub fn update(&mut self, _args: &UpdateArgs, delta_ts: u128) {
        self.delta_ts = delta_ts;
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, game: &Game) {
        window.draw_2d(evnt, |ctx, gl, _device| {
            self.img.draw(
                &game.hard_textures[&HardTexture::Interface],
                &DrawState::default(),
                ctx.transform,
                gl,
            );
        });
    }

    pub fn resize(&mut self, margin: &Size) {
        self.img = Image::new().rect([
            margin.width,
            margin.height + constants::MAP_HEIGHT as f64,
            constants::GUI_WIDTH as f64,
            constants::GUI_HEIGHT as f64,
        ]);
    }

    pub fn render_text_overlay(
        &mut self,
        evnt: &Event,
        window: &mut PistonWindow,
        font: &mut Font,
        margin: &Size,
        map_info: &String,
        game_data: &FakeGameData,
    ) {
        let map_coord_txt = format!(
            "{}\nCoordonnées: {}, {}",
            map_info, game_data.player.world_coord.x, game_data.player.world_coord.y
        );
        font.render_text(
            &map_coord_txt.as_str(),
            UI_OVERLAY_FONT_SIZE,
            evnt,
            window,
            color::WHITE,
            [5.0, 17.0],
            Some(margin),
        );

        if self.timer > 500 {
            self.timer = 0;
            self.fps = -1;
            if self.delta_ts > 0 {
                self.fps = (1000 / self.delta_ts) as i16;
            }
        } else {
            self.timer += self.delta_ts;
        }

        font.render_text(
            format!("delta_time: {:?} ms\nFPS: {:?}", self.delta_ts, self.fps).as_str(),
            UI_OVERLAY_FONT_SIZE,
            evnt,
            window,
            color::WHITE,
            [5.0, 64.0],
            Some(margin),
        );

        let mut render_entity_name = |entity: &Entity| {
            let e_name_coord = [
                entity.map_coord.x as f64,
                entity.map_coord.y as f64 - PLAYER_HEIGHT as f64,
            ];
            let final_color = match entity.r#type {
                EntityType::Player => self.player_color,
                EntityType::Monster => self.mob_color,
            };
            font.render_text_centered(
                entity.get_name().as_str(),
                17,
                evnt,
                window,
                final_color,
                e_name_coord,
                Some(margin),
            );
        };

        render_entity_name(&game_data.player);
        for entity in game_data.entities.iter() {
            render_entity_name(entity);
        }
    }
}
