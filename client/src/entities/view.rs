use std::collections::HashMap;

use piston_window::*;

use crate::constants::*;
use crate::entities::model::Bestiary;
use crate::import::assets::{EntityAssets, GameAsset};
use crate::ui::font::Font;

use super::model::{IEntity, Orientation};

pub struct EntityView {
    pub assets: HashMap<EntityAssets, GameAsset>,
    pub margin: Size,
    color_table: EntityColorTable,
}

struct EntityColorTable(Vec<[f32; 4]>);

impl EntityColorTable {
    fn new() -> Self {
        Self(BESTIARY.iter().map(|species| color::hex(species.font_color)).collect())
    }

    fn get(&self, species: Species) -> [f32; 4] {
        self.0[species as usize]
    }
}

impl EntityView {
    pub fn new(assets: HashMap<EntityAssets, GameAsset>) -> Self {
        EntityView {
            assets,
            color_table: EntityColorTable::new(),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    fn get_render_height_offset(&self, entity: &Box<dyn IEntity>) -> f64 {
        BESTIARY[*entity.get_race() as usize].render_offset_y
    }

    pub fn render(
        &mut self,
        evnt: &Event,
        window: &mut PistonWindow,
        entity: &Box<dyn IEntity>,
        font: &mut Font,
    ) {
        if let Some(asset) = self.assets.get(entity.get_assets()) {
            window.draw_2d(evnt, |ctx, gl, _device| {
                let asset_src_rect = asset.frames[entity.get_frame()].src_rect;
                let asset_width = asset_src_rect[2];
                let entity_pos = entity.get_real_pos();
                let render_offset_y = BESTIARY[*entity.get_race() as usize].render_offset_y;

                let mut trans = ctx.transform.trans(
                    self.margin.width as f64 + entity_pos.x as f64,
                    self.margin.height as f64 + entity_pos.y as f64 - render_offset_y,
                );
                // Offset the point controlled by the user's keyboard, to bottom center of the sprite character.

                // Flip the sprite according to Est/Wes direction
                trans = match entity.get_orientation() {
                    Orientation::West => trans.flip_h().trans(asset_width * -1.0, 0.0),
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
            self.render_entity_name(evnt, window, entity, font);
        }
    }

    fn render_entity_name(
        &self,
        evnt: &Event,
        window: &mut PistonWindow,
        entity: &Box<dyn IEntity>,
        font: &mut Font,
    ) {
        let entity_pos = entity.get_real_pos();

        let text_position = [
            entity_pos.x as f64 + TILE_WIDTH as f64 / 2.0,
            entity_pos.y as f64 - self.get_render_height_offset(entity),
        ];
        let text_color = match entity.get_race() {
            &Bestiary::Human => self.color_table.get(Species::Human),
            &Bestiary::Bouftou => self.color_table.get(Species::Bouftou),
        };
        font.render_text_centered(
            entity.get_name(),
            17,
            evnt,
            window,
            text_color,
            text_position,
            Some(&self.margin),
        );
    }

    pub fn update(&mut self, delta_ts: u128, entity: &mut Box<dyn IEntity>) {
        if let Some(asset) = self.assets.get(entity.get_assets()) {
            let timer = entity.get_timer();
            let frame = entity.get_frame();

            if timer >= asset.frames[frame].duration {
                if frame >= (asset.frames.len() - 1) {
                    entity.set_frame(0);
                } else {
                    entity.set_frame(frame as u8 + 1);
                }
                entity.set_timer(0);
            } else {
                entity.set_timer(timer + delta_ts);
            }
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }
}
