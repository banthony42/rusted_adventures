use std::{collections::HashMap};
use opengl_graphics::{Texture, TextureSettings, ImageSize};

use crate::aseprite_export_tilemap::{self, AsepriteExportTileMap};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Coord {
    pub x: i32,
    pub y: i32
}

pub struct Tileset {
    pub tileset: Texture,
    pub tile_width: u32,
    pub tile_height: u32,
    pub file_path: String
}

pub struct TileMapData {
    pub tiles: Vec<u32>,
    pub width: u32,
    pub height: u32,
    pub tileset: Tileset,
}

impl TileMapData {
    pub fn draw<F>(&self, draw_func: &mut F)
    where F: FnMut(&[f64;4], &Tileset, f64, f64)
    {
        let _ = self.tiles.iter().enumerate().map(|(index, tile_number)| {

            if *tile_number == 0 {
                return
            }

            let src_rect = [
                (*tile_number % (self.tileset.tileset.get_width() / self.tileset.tile_width ) * self.tileset.tile_width) as f64,
                (*tile_number / (self.tileset.tileset.get_width() / self.tileset.tile_width) * self.tileset.tile_height) as f64,
                self.tileset.tile_width as f64,
                self.tileset.tile_height as f64,
            ];

            let x = (index as u32 % self.width) as f64;
            let y = (index as u32 / self.width) as f64;

            draw_func(&src_rect, &self.tileset, x, y);

        }).collect::<Vec<_>>();
    }
}

pub struct MapData {
    pub map: TileMapData,
    pub collider: TileMapData,
    pub sprites: TileMapData
}

pub struct World {
    pub world: HashMap<Coord, MapData>
}

impl World {

    pub fn new() -> Self {

        let __world  = HashMap::from([
            (Coord { x:0, y:0}, "../assets/map_v3/sprite.json")
        ]);

        let mut world = World {
            world: HashMap::new()
        };

        for (coord, map_file) in __world {
            let loaded_map = AsepriteExportTileMap::new(map_file);

            let map_new_tileset = World::aseprite_tileset_to_game_tileset(&loaded_map.tilesets, loaded_map.map.tileset_index);
            let collider_new_tileset = World::aseprite_tileset_to_game_tileset(&loaded_map.tilesets, loaded_map.collider.tileset_index);
            let sprite_new_tileset = World::aseprite_tileset_to_game_tileset(&loaded_map.tilesets, loaded_map.sprites.tileset_index);

            world.world.insert(coord, MapData {
                map: World::aseprite_tilemap_to_game_tilemap(loaded_map.map, map_new_tileset),
                collider: World::aseprite_tilemap_to_game_tilemap(loaded_map.collider, collider_new_tileset),
                sprites: World::aseprite_tilemap_to_game_tilemap(loaded_map.sprites, sprite_new_tileset),
            });
        }
        return world;
    }

    fn aseprite_tileset_to_game_tileset(tilesets: &Vec<aseprite_export_tilemap::Tileset>, index: usize) -> Tileset {
        let map_export_tileset : &aseprite_export_tilemap::Tileset = &tilesets[index];
        let tilset_path = format!("../assets/{}", map_export_tileset.image.replace("\\", "/"));
        let tileset_texture = match Texture::from_path(&tilset_path, &TextureSettings::new()) {
            Ok(texture) => texture,
            Err(texture_error) => {
                println!("Fail to load texture (tileset PNG): {}", texture_error);
                std::process::exit(2);
            }
        };
        return Tileset {
                tileset: tileset_texture,
                file_path: tilset_path.to_string(),
                tile_width: map_export_tileset.grid.tileSize.width,
                tile_height: map_export_tileset.grid.tileSize.height
            };
    }

    fn aseprite_tilemap_to_game_tilemap(aseprite_tm: aseprite_export_tilemap::TileMapData, tileset: Tileset) -> TileMapData {
        TileMapData {
            tiles: aseprite_tm.tilemap.tiles,
            width: aseprite_tm.tilemap.width,
            height: aseprite_tm.tilemap.height,
            tileset: tileset
        }
    }
}