use std::collections::HashMap;

use piston_window::*;

use crate::constants::*;
use crate::import::assets::{EntityAssets, GameAsset};

use super::model::{IEntity, Orientation};

pub struct EntityView {
    pub assets: HashMap<EntityAssets, GameAsset>,
    pub margin: Size,
}

impl EntityView {
    pub fn new(assets: HashMap<EntityAssets, GameAsset>) -> Self {
        EntityView {
            assets,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, entity: &Box<dyn IEntity>) {
        if let Some(asset) = self.assets.get(entity.get_assets()) {
            window.draw_2d(evnt, |ctx, gl, _device| {
                let asset_src_rect = asset.frames[entity.get_frame()].src_rect;
                let asset_width = asset_src_rect[2];
                let asset_height = asset_src_rect[3];

                let entity_pos = entity.get_real_pos();
                let mut trans = ctx
                    .transform
                    .trans(
                        self.margin.width as f64 + entity_pos.x as f64 + 32.0,
                        self.margin.height as f64 + entity_pos.y as f64 + 32.0,
                    )
                    // Offset the point controlled by the user's keyboard, to bottom center of the sprite character.
                    .trans((asset_width / 2.0) * -1.0, asset_height * -1.0);

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
        }
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
