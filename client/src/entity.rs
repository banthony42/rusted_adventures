
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{TILEMAP_HEIGHT, TILEMAP_WIDTH}, world::{Coord, Sprite, World}
};

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GameTexture {
    Character,
    Interface
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub texture: GameTexture,
    pub map_coord: Coord,
    pub world_coord: Coord,
}

impl Entity {

    pub fn move_x(&mut self, step: i8,  sprites: &Vec<Sprite>, world: &World) -> bool {
        if step > 0 {
            // Right
            if self.map_coord.x < (TILEMAP_WIDTH - 1) as i32 {
                let tile_index = (self.map_coord.x + step as i32) + (self.map_coord.y * TILEMAP_WIDTH as i32);
 
                
                let a : Vec<&Sprite> = sprites.iter()
                .filter(|sprt| sprt.collider == false)
                .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16).collect();
            
                if a.len() > 0 {
                    self.map_coord.x += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the EAST
            else if self.map_coord.x >= (TILEMAP_WIDTH - 1) as i32 {
                let coord_tentative = Coord { x: self.world_coord.x + 1, y: self.world_coord.y};
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.x += 1;
                        self.map_coord.x = 0;
                        return true;
                    }
                    Err(_) => { println!("Unknown world map: {:?}", coord_tentative) }
                }
            }
        } else {
            // Left
            if self.map_coord.x > 0 {
                let tile_index = (self.map_coord.x + step as i32) + (self.map_coord.y * TILEMAP_WIDTH as i32);
                let a : Vec<&Sprite> = sprites.iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16).collect();

                if a.len() > 0 {
                    self.map_coord.x += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the WEST
            else if self.map_coord.x <= 0 {
                let coord_tentative = Coord { x: self.world_coord.x - 1, y: self.world_coord.y};
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.x -= 1;
                        self.map_coord.x = TILEMAP_WIDTH as i32 - 1;
                        return true;
                    }
                    Err(_) => { println!("Unknown world map: {:?}", coord_tentative) }
                }
            }
        }
        // Collision
        println!("[{}]: Boum ...", self.name);
        return false;
    }

    pub fn move_y(&mut self, step: i8, sprites: &Vec<Sprite>, world: &World) -> bool {
        if step > 0 {
            // Down
            if self.map_coord.y < (TILEMAP_HEIGHT - 1) as i32 {
                let tile_index = self.map_coord.x + ((self.map_coord.y + step as i32) * TILEMAP_WIDTH as i32);
                let a : Vec<&Sprite> = sprites.iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16).collect();

                if a.len() > 0 {
                    self.map_coord.y += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the SOUTH
            else if self.map_coord.y >= (TILEMAP_HEIGHT - 1) as i32 {
                let coord_tentative = Coord { x: self.world_coord.x , y: self.world_coord.y + 1};
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.y += 1;
                        self.map_coord.y = 0;
                        return true;
                    }
                    Err(_) => { println!("Unknown world map: {:?}", coord_tentative) }
                }
            }
        } else {
            // Up
            if self.map_coord.y > 0 {
                let tile_index = self.map_coord.x + ((self.map_coord.y + step as i32) * TILEMAP_WIDTH as i32);
                let a : Vec<&Sprite> = sprites.iter()
                    .filter(|sprt| sprt.collider == false)
                    .filter(|sprt| sprt.frames[sprt.frame_index].tilemap_index == tile_index as u16).collect();

                if a.len() > 0 {
                    self.map_coord.y += step as i32;
                    return true;
                }
            }
            // Player try to go to the next map to the NORTH
            else if self.map_coord.y <= 0 {
                let coord_tentative = Coord { x: self.world_coord.x , y: self.world_coord.y - 1};
                match world.get_world_map(&coord_tentative) {
                    Ok(_) => {
                        self.world_coord.y -= 1;
                        self.map_coord.y = TILEMAP_HEIGHT as i32 - 1;
                        return true;
                    }
                    Err(_) => { println!("Unknown world map: {:?}", coord_tentative) }
                }
            }
        }
        // Collision
        println!("[{}]: Boum ...", self.name);
        return false;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub base: Entity,
}

impl Deref for Player {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        return &self.base;
    }
}

impl DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Entity {
        return &mut self.base;
    }
}