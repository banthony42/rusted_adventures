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
        let raw_data: String = fs::read_to_string(path).expect("AsepriteExportTileMap: new(): Unable to read file.");
        let data: serde_json::Value = serde_json::from_str(&raw_data).expect("AsepriteExportTileMap: new(): Unable to parse.");
        let mut tilesets: Vec<Tileset> = Vec::new();

        let tilesets_values = data.get("tilesets")
            .expect("AsepriteExportTileMap: new(): key 'tilesets' not found in JSON.")
            .as_array()
            .expect("AsepriteExportTileMap: new(): Fail to get 'tilesets' as array.");

        for tileset in tilesets_values {
            let tileset_json = serde_json::json!(tileset).to_string();
            let loaded_tileset = serde_json::from_str(&tileset_json)
                .expect("AsepriteExportTileMap: new(): Fail to load 'tileset' value.");
            tilesets.push(loaded_tileset);
        }

        let map_layer: (Bounds, Tilemap, usize) = Self::extract_layer_by_name(&data, "Map")
            .expect("AsepriteExportTileMap: new(): Fail to extract Map layer.");
        let collider_layer: (Bounds, Tilemap, usize) = Self::extract_layer_by_name(&data, "Collider")
            .expect("AsepriteExportTileMap: new(): Fail to extract Collider layer.");
        let sprites_layer: (Bounds, Tilemap, usize) = Self::extract_layer_by_name(&data, "AnimatedSprites")
            .expect("AsepriteExportTileMap: new(): Fail to extract AnimatedSprites layer.");

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
        let layers = data["layers"].as_array()
            .expect("AsepriteExportTileMap: extract_layer_by_name: Fail to get 'layers' as array.");

        for layer in layers {
            match layer.get("name") {
                Some(layer_name) if layer_name == name => {
                    let map_layer = &layer["cels"][0];

                    let bounds_json = serde_json::json!(map_layer["bounds"]).to_string();
                    let bounds: Bounds   = serde_json::from_str(&bounds_json)
                       .expect("AsepriteExportTileMap: extract_layer_by_name: Fail to load 'bounds' value.");

                    let tilemap_json = serde_json::json!(map_layer["tilemap"]).to_string();
                    let tilemap: Tilemap   = serde_json::from_str(&tilemap_json)
                        .expect("AsepriteExportTileMap: extract_layer_by_name: Fail to load 'bounds' value.");

                    let tileset_index: usize = layer["tileset"].as_u64()
                        .expect("AsepriteExportTileMap: extract_layer_by_name: Fail to load 'tileset' (index) value.") as usize;
                    return Some((bounds, tilemap, tileset_index));
                },
                _ => {}
            }
        }
        return None;
    }
}