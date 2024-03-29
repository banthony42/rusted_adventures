
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{self, TILEMAP_HEIGHT, TILEMAP_WIDTH}, world::{Coord, Sprite}
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
}

impl Entity {
    pub fn move_x(&mut self, step: i8,  sprites: &Vec<Sprite>) -> bool {
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
        }
        // Collision
        println!("[{}]: Boum ...", self.name);
        return false;
    }

    pub fn move_y(&mut self, step: i8, sprites: &Vec<Sprite>) -> bool {
        if step > 0 {
            // Right
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
        } else {
            // Left
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
        }
        // Collision
        println!("[{}]: Boum ...", self.name);
        return false;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub base: Entity,
    pub world_coord: Coord,
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