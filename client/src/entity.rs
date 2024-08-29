use std::{collections::HashMap, default};

use graphics::{DrawState, Image, Transformed};
use piston_window::*;
use serde::{Deserialize, Serialize};

use crate::{
    assets::{Animations, EntityAssets, GameAsset},
    constants::{TILEMAP_HEIGHT, TILEMAP_WIDTH},
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

#[derive(Debug, Default)]
enum Orientation {
    #[default]
    Est,
    West,
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
    pub move_timer: u128,
    #[serde(skip)]
    pub allow_movement: bool,
    #[serde(skip)]
    pub orientation: Orientation,
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

const MOVE_LIMIT: u128 = 150;

impl Entity {
    fn change_state(&mut self, new_state: Animations) {
        if self.state != new_state {
            self.state = new_state;
            self.frame_number = 0;
            self.timer = 0;
        }
    }

    pub fn move_x(&mut self, step: i8, sprites: &Vec<Sprite>, world: &World) -> bool {
        self.change_state(Animations::Run);
        if self.allow_movement {
            self.allow_movement = false
        } else {
            return true;
        }
        if step > 0 {
            // Right
            self.orientation = Orientation::Est;
            if self.map_coord.x < (TILEMAP_WIDTH - 1) as i32 {
                let tile_index =
                    (self.map_coord.x + step as i32) + (self.map_coord.y * TILEMAP_WIDTH as i32);

                let a: Vec<&Sprite> = sprites
                    .iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16)
                    .collect();

                if a.len() > 0 {
                    self.map_coord.x += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the EAST
            else if self.map_coord.x >= (TILEMAP_WIDTH - 1) as i32 {
                let coord_tentative = Coord {
                    x: self.world_coord.x + 1,
                    y: self.world_coord.y,
                };
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.x += 1;
                        self.map_coord.x = 0;
                        return true;
                    }
                    Err(_) => {
                        println!("Unknown world map: {:?}", coord_tentative)
                    }
                }
            }
        } else {
            // Left
            self.orientation = Orientation::West;
            if self.map_coord.x > 0 {
                let tile_index =
                    (self.map_coord.x + step as i32) + (self.map_coord.y * TILEMAP_WIDTH as i32);
                let a: Vec<&Sprite> = sprites
                    .iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16)
                    .collect();

                if a.len() > 0 {
                    self.map_coord.x += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the WEST
            else if self.map_coord.x <= 0 {
                let coord_tentative = Coord {
                    x: self.world_coord.x - 1,
                    y: self.world_coord.y,
                };
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.x -= 1;
                        self.map_coord.x = TILEMAP_WIDTH as i32 - 1;
                        return true;
                    }
                    Err(_) => {
                        println!("Unknown world map: {:?}", coord_tentative)
                    }
                }
            }
        }
        // Collision
        println!("[{}]: Boum ...", self.name);
        return false;
    }

    pub fn move_y(&mut self, step: i8, sprites: &Vec<Sprite>, world: &World) -> bool {
        self.change_state(Animations::Run);
        if self.allow_movement {
            self.allow_movement = false
        } else {
            return true;
        }
        if step > 0 {
            // Down
            if self.map_coord.y < (TILEMAP_HEIGHT - 1) as i32 {
                let tile_index =
                    self.map_coord.x + ((self.map_coord.y + step as i32) * TILEMAP_WIDTH as i32);
                let a: Vec<&Sprite> = sprites
                    .iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16)
                    .collect();

                if a.len() > 0 {
                    self.map_coord.y += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the SOUTH
            else if self.map_coord.y >= (TILEMAP_HEIGHT - 1) as i32 {
                let coord_tentative = Coord {
                    x: self.world_coord.x,
                    y: self.world_coord.y + 1,
                };
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.y += 1;
                        self.map_coord.y = 0;
                        return true;
                    }
                    Err(_) => {
                        println!("Unknown world map: {:?}", coord_tentative)
                    }
                }
            }
        } else {
            // Up
            if self.map_coord.y > 0 {
                let tile_index =
                    self.map_coord.x + ((self.map_coord.y + step as i32) * TILEMAP_WIDTH as i32);
                let a: Vec<&Sprite> = sprites
                    .iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16)
                    .collect();

                if a.len() > 0 {
                    self.map_coord.y += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the NORTH
            else if self.map_coord.y <= 0 {
                let coord_tentative = Coord {
                    x: self.world_coord.x,
                    y: self.world_coord.y - 1,
                };
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.y -= 1;
                        self.map_coord.y = TILEMAP_HEIGHT as i32 - 1;
                        return true;
                    }
                    Err(_) => {
                        println!("Unknown world map: {:?}", coord_tentative)
                    }
                }
            }
        }
        // Collision
        println!("[{}]: Boum ...", self.name);
        return false;
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
                    let trans: [[f64; 3]; 2] = ctx.transform.trans(
                        game.margin.width as f64 + self.map_coord.x as f64 * 64.0,
                        game.margin.height as f64 + (self.map_coord.y as f64 * 64.0) - 64.0,
                    );

                    let trans = match self.orientation {
                        Orientation::Est => trans,
                        Orientation::West => trans.flip_h().trans(-64.0, 0.0),
                    };

                    Image::new()
                        .src_rect(asset.frames[self.frame_number].src_rect)
                        .draw(&asset.texture, &DrawState::default(), trans, gl);
                });
            }
            None => println!(
                "Assets not found for Entity {{ races: {:?}, state: {:?}}}",
                self.race, self.state
            ),
        }
    }

    pub fn update(&mut self, delta_ts: u128, assets: &HashMap<EntityAssets, GameAsset>) {
        if self.move_timer >= MOVE_LIMIT {
            self.move_timer = 0;
            self.allow_movement = true;
        } else if self.allow_movement == false {
            self.move_timer += delta_ts;
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

    pub fn key_press(&mut self, args: &Button, world: &World) {
        let map_data = &world.world[&self.world_coord];
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Up => {
                    self.move_y(-1, &map_data.sprites, &world);
                }
                piston::Key::Down => {
                    self.move_y(1, &map_data.sprites, &world);
                }
                piston::Key::Left => {
                    self.move_x(-1, &map_data.sprites, &world);
                }
                piston::Key::Right => {
                    self.move_x(1, &map_data.sprites, &world);
                }
                _ => {}
            }
        }
    }

    pub fn key_release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Up | piston::Key::Down | piston::Key::Left | piston::Key::Right => {
                    self.change_state(Animations::Idle);
                    self.move_timer = 0;
                    self.allow_movement = true;
                }
                _ => {}
            }
        }
    }
}
