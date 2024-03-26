
use serde::{Deserialize, Serialize};

use crate::{
    world::TileMapData, constants, world::Coord
};

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GameTexture {
    Character,
    Interface
}

// TODO: use getter / setter make attributes private
#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub texture: GameTexture,
    pub map_coord: Coord,
    pub world_coord: Coord,
}


impl Player {

    pub fn new (world_coord :Coord, map_coord : Coord) -> Self {
        return Player {
            texture: GameTexture::Character,
            map_coord: map_coord,
            world_coord: world_coord
        }
    }
    pub fn move_player_x(&mut self, step: i8, collider: &TileMapData) -> bool {
        if step > 0 {
            // Right
            if self.map_coord.x < (constants::TILEMAP_WIDTH - 1) as i32 {
                let tile_index = (self.map_coord.x + step as i32) + (self.map_coord.y * collider.width as i32);
                if collider.tiles[tile_index as usize] == 0 {
                    self.map_coord.x += step as i32;
                    return true;
                }
            }
        } else {
            // Left
            if self.map_coord.x > 0 {
                let tile_index = (self.map_coord.x + step as i32) + (self.map_coord.y * collider.width as i32);
                if collider.tiles[tile_index as usize] == 0 {
                    self.map_coord.x += step as i32;
                    return true;
                }
            }
        }
        // Collision
        println!("Boum ...");
        return false;
    }


    pub fn move_player_y(&mut self, step: i8, collider: &TileMapData) -> bool {
        if step > 0 {
            // Right
            if self.map_coord.y < (constants::TILEMAP_HEIGHT - 1) as i32 {
                let tile_index = self.map_coord.x + ((self.map_coord.y + step as i32) * collider.width as i32);
                if collider.tiles[tile_index as usize] == 0 {
                    self.map_coord.y += step as i32;
                    return true;
                }
            }
        } else {
            // Left
            if self.map_coord.y > 0 {
                let tile_index = self.map_coord.x + ((self.map_coord.y + step as i32) * collider.width as i32);
                if collider.tiles[tile_index as usize] == 0 {
                    self.map_coord.y += step as i32;
                    return true;
                }
            }
        }
        // Collision
        println!("Boum ...");
        return false;
    }
}