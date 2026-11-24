use std::collections::HashMap;

use common::entities::traits::GameEntity;
use piston_window::*;

use crate::entities::model::ClientEntity;
use crate::import::assets::{load_assets, EntityAssets, GameAsset};
use crate::ui::font::Font;
use common::{constants::*, Orientation};

pub struct EntityView {
    pub assets: HashMap<EntityAssets, GameAsset>,
    pub margin: Size,
    font: Font,
}

impl EntityView {
    pub fn new(window: &mut PistonWindow) -> Self {
        EntityView {
            font: Font::new(GUI_ENTITY_FONT_SIZE).load(window),
            assets: load_assets(window),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn font(&mut self) -> Vec<&mut Font> {
        vec![&mut self.font]
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, entity: &ClientEntity) {
        if let Some(asset) = self.assets.get(&entity.asset()) {
            window.draw_2d(evnt, |ctx, gl, _device| {
                let asset_src_rect = asset.frames[entity.graphic.get_frame()].src_rect;
                let asset_width = asset_src_rect[2];
                let entity_pos = entity.get_screen_pos();

                let mut trans = ctx.transform.trans(
                    self.margin.width + entity_pos.x as f64,
                    self.margin.height + entity_pos.y as f64 - entity.graphic.render_offset()[1],
                );
                // Offset the point controlled by the user's keyboard, to bottom center of the sprite character.

                // Flip the sprite according to Est/Wes direction
                trans = match entity.graphic.get_orientation() {
                    Orientation::West => trans.flip_h().trans(-asset_width, 0.0),
                    _ => trans, // Orientation::Est default (define by the sprite) will be fixed when all orientation sprites available
                };

                let map_scissor = [
                    self.margin.width as u32,
                    self.margin.height as u32,
                    MAP_WIDTH as u32,
                    MAP_HEIGHT as u32,
                ];

                Image::new().src_rect(asset_src_rect).draw(
                    &asset.texture,
                    &DrawState::default().scissor(map_scissor),
                    trans,
                    gl,
                );
            });
            self.render_entity_name(evnt, window, entity);
        }
    }

    fn render_entity_name(
        &mut self,
        evnt: &Event,
        window: &mut PistonWindow,
        entity: &ClientEntity,
    ) {
        let entity_pos = entity.get_screen_pos();
        let text_color = entity.graphic.font_color();
        let text_position = [
            entity_pos.x as f64 + TILE_WIDTH as f64 / 2.0,
            entity_pos.y as f64 - entity.graphic.render_offset()[1],
        ];

        self.font.set_color(text_color).render_centered(
            entity.model().name(),
            evnt,
            window,
            text_position,
            &self.margin,
        );
    }

    pub fn update(&mut self, delta_ts: u128, entity: &mut ClientEntity) {
        if let Some(asset) = self.assets.get(&entity.asset()) {
            let timer = entity.graphic.get_timer();
            let frame = entity.graphic.get_frame();

            if timer >= asset.frames[frame].duration {
                if frame >= (asset.frames.len() - 1) {
                    entity.graphic.set_frame(0);
                } else {
                    entity.graphic.set_frame(frame as u8 + 1);
                }
                entity.graphic.set_timer(0);
            } else {
                entity.graphic.set_timer(timer + delta_ts);
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = *margin;
    }
}
