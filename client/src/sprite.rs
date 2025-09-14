use crate::world::Offset;
use common::constants::*;
use piston_window::*;

pub struct Sprite {
    tileset: G2dTexture, // Textures
    pub tile: u8,        // Value of the tile to draw from the tileset texture
    pub offset: Offset,  // Where the flat position start in the tilemap (coord in pixel)
    pub position: u32, // Flat position in the tilemap. (x = position % tilemap_width and y = position / tilemap_height)
    pub collider: bool, // Does this sprite is a collider
}

pub struct Frame {
    pub sprites: Vec<Sprite>,
    pub duration: f32,
}

impl Frame {
    pub fn new(sprites: Vec<Sprite>, duration: f32) -> Frame {
        Frame {
            sprites: sprites,
            duration: duration,
        }
    }
}

impl Sprite {
    pub fn new(
        tileset: G2dTexture,
        tile: u8,
        offset: Offset,
        position: u32,
        collider: bool,
    ) -> Sprite {
        Sprite {
            tileset,
            tile,
            offset,
            position,
            collider,
        }
    }

    pub fn get_texture(&self) -> &G2dTexture {
        &self.tileset
    }

    pub fn get_src_rect(&self) -> [f64; 4] {
        let f64width = TILE_WIDTH as f64;
        let f64height = TILE_HEIGHT as f64;
        [
            self.tile as f64 % (self.tileset.get_width() as f64 / f64width) * f64width,
            self.tile as f64 / (self.tileset.get_width() as f64 / f64width) * f64height,
            f64width,
            f64height,
        ]
    }

    pub fn get_tile_position(&self) -> [f64; 2] {
        let u32width = TILEMAP_WIDTH as u32;
        [
            (self.position % u32width) as f64,
            (self.position / u32width) as f64,
        ]
    }
}
