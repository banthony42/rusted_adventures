use crate::{
    constants::{TILEMAP_HEIGHT, TILEMAP_WIDTH},
    world::tilemap::{LoadedMap, MapLayer},
    MapCoord,
};
use std::{collections::HashMap, fs};

pub mod tilemap;

struct MapImport {
    path: String,
    info: String,
}

#[derive(Eq, Hash, PartialEq)]
pub struct MapInfo {
    pub coord: MapCoord,
    pub info: String,
}

pub struct RawMap {
    pub loaded_map: LoadedMap,
    pub collider_map: ColliderMap,
}
pub struct WorldImport {
    pub atlas: HashMap<MapInfo, RawMap>,
}

impl WorldImport {
    fn atlas_import() -> HashMap<MapCoord, MapImport> {
        HashMap::from([
            (
                MapCoord { x: 0, y: 0 },
                MapImport {
                    // TODO: fix path here it is relative could be an issue when run from different location client/server folders
                    path: String::from("../assets/maps/map.0.0/sprite.json"),
                    info: String::from("Plaines"),
                },
            ),
            (
                MapCoord { x: 1, y: 0 },
                MapImport {
                    path: String::from("../assets/maps/map.1.0/sprite.json"),
                    info: String::from("Plage cliquetante"),
                },
            ),
        ])
    }

    pub fn new() -> Self {
        let atlas_import = Self::atlas_import();
        let mut atlas = HashMap::<MapInfo, RawMap>::new();
        for (coord, map_import) in atlas_import {
            let raw_data: String = fs::read_to_string(&map_import.path)
                .expect("test_map_import: Unable to read file.");
            let loaded_map = serde_json::from_str::<LoadedMap>(&raw_data)
                .expect(&format!("Fail to load JSON map: {}", &map_import.path));
            atlas.insert(
                MapInfo {
                    coord,
                    info: map_import.info,
                },
                RawMap {
                    collider_map: ColliderMap::from(&loaded_map),
                    loaded_map,
                },
            );
        }
        Self { atlas }
    }
}

pub struct ColliderMap(Vec<Vec<bool>>);

impl ColliderMap {
    pub fn is_collider(&self, y: usize, x: usize) -> bool {
        self.0[y][x]
    }

    pub fn is_not_collider(&self, y: usize, x: usize) -> bool {
        self.is_collider(y, x) == false
    }
}

impl From<&LoadedMap> for ColliderMap {
    fn from(loaded_map: &LoadedMap) -> Self {
        let mut colliders = vec![vec![false; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize];
        let _: Vec<_> = loaded_map
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == MapLayer::Collider)
            .map(|sprite| {
                let x = sprite.tile_index as usize % TILEMAP_WIDTH;
                let y = sprite.tile_index as usize / TILEMAP_WIDTH;
                colliders[y][x] = sprite.collider;
            })
            .collect();
        ColliderMap(colliders)
    }
}
