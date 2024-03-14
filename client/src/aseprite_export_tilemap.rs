use std::fs;

use serde::{Deserialize, Serialize};

/*
** Aseprite tilemap JSON export using assets/aseprite_convert_map.bat
*/

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounds {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tilesize {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)] // struct fields name should be the same as JSON fields.
pub struct Grid {
    pub tileSize: Tilesize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tileset {
    pub grid: Grid,
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tilemap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u32>,
}

////////////////////////////////////////////////

/*
**  Final data structure for the Map
*/

#[derive(Debug)]
pub struct TileMapData {
    pub tilemap: Tilemap,
    pub tileset_index: usize,
    pub bounds: Bounds
}

#[derive(Debug)]
pub struct AsepriteExportTileMap {
    pub tilesets: Vec<Tileset>,
    pub map: TileMapData,
    pub collider: TileMapData,
    pub sprites: TileMapData
}

impl AsepriteExportTileMap {

    pub fn new(path :&str) -> Self {
        let raw_data: String = fs::read_to_string(path).expect("Unable to read file");
        let data: serde_json::Value = serde_json::from_str(&raw_data).expect("Unable to parse");
        let mut tilesets: Vec<Tileset> = Vec::new();

        for tileset in data["tilesets"].as_array().unwrap() {
            let tileset_json = serde_json::json!(tileset).to_string();
            tilesets.push(serde_json::from_str(&tileset_json).unwrap())
        }

        let map_layer: (Bounds, Tilemap, usize) = Self::extract_layer_by_name(&data, "Map").unwrap();
        let collider_layer: (Bounds, Tilemap, usize) = Self::extract_layer_by_name(&data, "Collider").unwrap();
        let sprites_layer: (Bounds, Tilemap, usize) = Self::extract_layer_by_name(&data, "AnimatedSprites").unwrap();

        return AsepriteExportTileMap {
            tilesets: tilesets,
            map: TileMapData {
                bounds: map_layer.0,
                tilemap: map_layer.1,
                tileset_index: map_layer.2
            },
            collider: TileMapData {
                bounds: collider_layer.0,
                tilemap: collider_layer.1,
                tileset_index: collider_layer.2
            },
            sprites: TileMapData {
                bounds: sprites_layer.0,
                tilemap: sprites_layer.1,
                tileset_index: sprites_layer.2
            }
        };
    }

    fn extract_layer_by_name(data: &serde_json::Value, name: &str) -> Option<(Bounds, Tilemap, usize)> {
        for layer in data["layers"].as_array().unwrap() {
            match layer.get("name") {
                Some(layer_name) if layer_name == name => {
                    let map_layer = &layer["cels"][0];

                    let bounds_json = serde_json::json!(map_layer["bounds"]).to_string();
                    let bounds: Bounds   = serde_json::from_str(&bounds_json).unwrap();

                    let tilemap_json = serde_json::json!(map_layer["tilemap"]).to_string();
                    let tilemap: Tilemap   = serde_json::from_str(&tilemap_json).unwrap();

                    let tileset_index: usize = layer["tileset"].as_u64().unwrap() as usize;
                    return Some((bounds, tilemap, tileset_index));
                },
                _ => {}
            }
        }
        return None;
    }
}