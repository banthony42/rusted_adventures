use std::collections::HashMap;

use graphics::{DrawState, Image, Transformed};
use piston_window::*;
use serde::{Deserialize, Serialize};

use crate::{
    assets::{Animations, EntityAssets, GameAsset},
    constants::{
        MAP_HEIGHT, MAP_WIDTH, PLAYER_CENTER_X, PLAYER_HEIGHT, PLAYER_WIDTH, TILEMAP_WIDTH,
        TILE_HEIGHT, TILE_WIDTH,
    },
    game::Game,
    world::{Coord, Sprite, World},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    Monster,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EntityRaces {
    Character,
    Bouftou,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Orientation {
    #[default]
    Est,
    West,
    North,
    South,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub r#type: EntityType,
    pub race: EntityRaces,
    pub state: Animations,
    #[serde(skip)]
    pub frame_number: usize,
    #[serde(skip)]
    pub timer: u128,
    #[serde(skip)]
    orientation: Orientation,
    #[serde(skip)]
    pub offset: Coord,
    pub map_coord: Coord,
    pub world_coord: Coord,
}

pub trait Name {
    fn get_name(&self) -> String;
}

impl Name for Entity {
    fn get_name(&self) -> String {
        match self.r#type {
            EntityType::Player => format!("<{}>", self.name),
            EntityType::Monster => self.name.clone(),
        }
    }
}

const PLAYER_SPEED: i32 = 2; // TODO float

impl Entity {
    fn change_state(&mut self, new_state: Animations) {
        if self.state != new_state {
            self.state = new_state;
            self.frame_number = 0;
            self.timer = 0;
        }
    }

    fn animation_lookup(&self) -> &EntityAssets {
        match self.race {
            EntityRaces::Character => {
                // TODO: find a way to opti using EntityAssets::Character(self.state)
                match self.state {
                    Animations::Idle => &EntityAssets::Character(Animations::Idle),
                    Animations::Run => &EntityAssets::Character(Animations::Run),
                }
            }
            EntityRaces::Bouftou => &EntityAssets::Bouftou,
        }
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, game: &Game) {
        match game.assets.get(self.animation_lookup()) {
            Some(asset) => {
                window.draw_2d(evnt, |ctx, gl, _device| {
                    let mut trans = ctx
                        .transform
                        .trans(
                            game.margin.width as f64 + self.map_coord.x as f64,
                            game.margin.height as f64 + self.map_coord.y as f64,
                        )
                        // Offset the character to bottom center it on the point controlled by the user's keyboard.
                        .trans(PLAYER_CENTER_X as f64 * -1.0, PLAYER_HEIGHT as f64 * -1.0);

                    // Flip the sprite according to Est/Wes direction 
                    trans = match self.offset.x.is_negative() || self.orientation == Orientation::West {
                        true => trans.flip_h().trans(PLAYER_WIDTH as f64 * -1.0, 0.0),
                        false => trans
                    };

                    let map_scissor = [
                        game.margin.width as u32,
                        game.margin.height as u32,
                        MAP_WIDTH as u32,
                        MAP_HEIGHT as u32,
                    ];
                    Image::new()
                        .src_rect(asset.frames[self.frame_number].src_rect)
                        .draw(
                            &asset.texture,
                            &DrawState::default().scissor(map_scissor),
                            trans,
                            gl,
                        );
                });
            }
            None => {} // println!("Assets not found for Entity {{ races: {:?}, state: {:?}}}", self.race, self.state),
        }

        // Draw data for debug purpose
        // self._render_dbg(evnt, window, game);
    }

    fn _render_dbg(&self, evnt: &Event, window: &mut PistonWindow, game: &Game) {
        window.draw_2d(evnt, |ctx, gl, _device| {
            let trans = ctx.transform.trans(
                game.margin.width as f64 + self.map_coord.x as f64,
                game.margin.height as f64 + self.map_coord.y as f64,
            );
            let rzero = rectangle::square(0.0, 0.0, 0.0);
            circle_arc(color::RED, 7.0, 0.0, 10.0, rzero, trans, gl);
        });
    }

    fn detect_map_collisions(&self, world: &World) -> bool {
        let mut pt = self.map_coord + self.offset;
        // Compute the sprite cell x,y coordinate
        pt.x = (pt.x - (pt.x % TILE_WIDTH as i32)) / 64;
        pt.y = (pt.y - (pt.y % TILE_HEIGHT as i32)) / 64;

        // Compute the sprite cell number
        let tile_index = pt.x + (pt.y * TILEMAP_WIDTH as i32);

        return world.world[&self.world_coord]
            .sprites
            .iter()
            .filter(|sprt| sprt.collider == false) // For all sprites collider for this map
            .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16) // Does the entity is on the collider cell
            .collect::<Vec<&Sprite>>()
            .is_empty();
    }

    fn detect_map_change(&mut self, world: &World) {
        if self.map_coord.x >= MAP_WIDTH as i32 {
            match world.get_east_map(&self.world_coord) {
                Some(e) => {
                    self.map_coord.x = 0;
                    self.world_coord = e.0;
                }
                None => {}
            }
        } else if self.map_coord.x <= PLAYER_CENTER_X as i32 - 2 {
            match world.get_west_map(&self.world_coord) {
                Some(e) => {
                    self.map_coord.x = MAP_WIDTH as i32 - 1;
                    self.world_coord = e.0;
                }
                None => {}
            }
        }
        self.map_coord.x = self.map_coord.x.min(MAP_WIDTH as i32).max(PLAYER_CENTER_X as i32);
        self.map_coord.y = self.map_coord.y.min(MAP_HEIGHT as i32).max(PLAYER_CENTER_X as i32);
    }

    pub fn update(&mut self, delta_ts: u128, assets: &HashMap<EntityAssets, GameAsset>, world: &World) {
        if self.detect_map_collisions(world) == false {
            self.map_coord += self.offset;
            self.detect_map_change(world);
        }

        if self.offset.is_null() {
            self.change_state(Animations::Idle);
        } else {
            self.change_state(Animations::Run);
        }

        match assets.get(self.animation_lookup()) {
            Some(asset) => {
                if self.timer >= asset.frames[self.frame_number].duration {
                    if self.frame_number >= (asset.frames.len() - 1) {
                        self.frame_number = 0;
                    } else {
                        self.frame_number += 1;
                    }
                    self.timer = 0;
                } else {
                    self.timer += delta_ts;
                }
            }
            None => {}
        }
    }

    fn move_pos(&mut self, dir: Orientation) {
        self.orientation = dir;
        match dir {
            Orientation::Est => self.offset.x = PLAYER_SPEED,
            Orientation::West => self.offset.x = -PLAYER_SPEED,
            Orientation::North => self.offset.y = -PLAYER_SPEED,
            Orientation::South => self.offset.y = PLAYER_SPEED,
        };
    }

    fn stop_pos(&mut self, dir: Orientation) {
        match dir {
            Orientation::Est => self.offset.x = 0,
            Orientation::West => self.offset.x = 0,
            Orientation::North => self.offset.y = 0,
            Orientation::South => self.offset.y = 0,
        };
    }

    pub fn key_press(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Up => self.move_pos(Orientation::North),
                piston::Key::Down => self.move_pos(Orientation::South),
                piston::Key::Left => self.move_pos(Orientation::West),
                piston::Key::Right => self.move_pos(Orientation::Est),
                _ => {}
            }
        }
    }

    pub fn key_release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Up => self.stop_pos(Orientation::North),
                piston::Key::Down => self.stop_pos(Orientation::South),
                piston::Key::Left => self.stop_pos(Orientation::Est),
                piston::Key::Right => self.stop_pos(Orientation::West),
                _ => {}
            }
        }
    }
}
