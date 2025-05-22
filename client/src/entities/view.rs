use std::collections::HashMap;

use piston_window::*;

use crate::constants::*;
use crate::import::assets::{EntityAssets, GameAsset};

use super::model::IEntity;

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
                let trans = ctx
                    .transform
                    .trans(
                        self.margin.width as f64 + (entity.get_map().x * 64) as f64 + 32.0,
                        self.margin.height as f64 + (entity.get_map().y * 64) as f64 + 32.0,
                    )
                    // Offset the point controlled by the user's keyboard, to bottom center of the sprite character.
                    .trans(PLAYER_CENTER_X as f64 * -1.0, PLAYER_HEIGHT as f64 * -1.0);

                let map_scissor = [
                    self.margin.width as u32,
                    self.margin.height as u32,
                    MAP_WIDTH as u32,
                    MAP_HEIGHT as u32,
                ];

                Image::new()
                    .src_rect(asset.frames[entity.get_frame()].src_rect)
                    .draw(
                        &asset.texture,
                        &DrawState::default().scissor(map_scissor),
                        trans,
                        gl,
                    );

                if let Some(path) = entity.get_path() {
                    let _: Vec<_> = path
                        .iter()
                        .enumerate()
                        .map(|(index, cell)| {
                            let cell_color = [0.0, (1.0 - (1.0 / index as f32)), 1.0, 0.4];
                            Rectangle::new(cell_color).draw(
                                [
                                    self.margin.width + (cell.x * 64 as i32) as f64,
                                    self.margin.height + (cell.y * 64 as i32) as f64,
                                    64.0,
                                    64.0,
                                ],
                                &ctx.draw_state,
                                ctx.transform,
                                gl,
                            );
                        })
                        .collect();
                }
            });
        }
    }

    pub fn update(&mut self, delta_ts: u128, entity: &mut Box<dyn IEntity>) {
        if let Some(asset) = self.assets.get(entity.get_assets()) {
            let timer = entity.get_timer();
            let frame = entity.get_frame();

            if timer >= asset.frames[entity.get_frame()].duration {
                if entity.get_frame() >= (asset.frames.len() - 1) {
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
