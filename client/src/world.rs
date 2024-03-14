use std::{clone, collections::HashMap};
use opengl_graphics::{Texture, TextureSettings};

use crate::aseprite_export_tilemap::{self, AsepriteExportTileMap};

#[derive(Eq, Hash, PartialEq)]
pub struct Coord {
    pub x: u32,
    pub y: u32
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
    pub tileset_index: usize,
}

pub struct MapData {
    pub tilesets: Vec<Tileset>,
    pub map: TileMapData,
    pub collider: TileMapData,
    pub sprites: TileMapData
}

pub struct World {
    pub world: HashMap<Coord, MapData>
}

impl World {

    pub fn new() -> Self {

        let WORLD = HashMap::from([
            (Coord { x:0, y:0}, "../assets/map_collision_sprites_v2/sprite.json")
        ]);

        let mut world = World {
            world: HashMap::new()
        };

        for (coord, map_file) in WORLD {
            let loaded_map = AsepriteExportTileMap::new(map_file);

            let map_tileset = loaded_map.tilesets.iter().map(|tileset| {
                let tilset_path = format!("../assets/{}", tileset.image.replace("\\", "/"));
                return Tileset {
                    tileset: Texture::from_path(&tilset_path, &TextureSettings::new()).unwrap(),
                    file_path: tilset_path.to_string(),
                    tile_width: tileset.grid.tileSize.width,
                    tile_height: tileset.grid.tileSize.height
                }
            }).collect::<Vec<Tileset>>();

            world.world.insert(coord, MapData {
                tilesets: map_tileset,
                map: Self::aseprite_tilemap_to_game_tilemap(loaded_map.map),
                collider: Self::aseprite_tilemap_to_game_tilemap(loaded_map.collider),
                sprites: Self::aseprite_tilemap_to_game_tilemap(loaded_map.sprites)
            });
        }

        return world;
    }

    fn aseprite_tilemap_to_game_tilemap(aseprite_tm: aseprite_export_tilemap::TileMapData) -> TileMapData {
        TileMapData {
            tiles: aseprite_tm.tilemap.tiles,
            width: aseprite_tm.tilemap.width,
            height: aseprite_tm.tilemap.height,
            tileset_index: aseprite_tm.tileset_index
        }
    }
}

pub fn test() {
    let world = World::new();

    let map_from_server = Coord { x:0, y:0};

    let map = &world.world[&map_from_server];
}