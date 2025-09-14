use piston_window::*;
use std::collections::HashMap;
use std::fs;

use crate::{
    import::tilemap::LoadedMap,
    sprite::{Frame, Sprite},
};
use common::{constants::*, WorldCoord};

#[derive(Default)]
pub struct Offset {
    pub x: u64,
    pub y: u64,
}

pub struct MapData {
    pub info: String,
    pub frames: Vec<Frame>,
    pub timer: u128,
    pub f_ptr: usize,
    pub colliders: Vec<Vec<bool>>,
}

struct MapImport {
    path: String,
    info: String,
}

pub struct World {
    pub world: HashMap<WorldCoord, MapData>,
    margin: Size,
}

impl World {
    pub fn new(window: &mut PistonWindow) -> Self {
        let __world = HashMap::from([
            (
                WorldCoord { x: 0, y: 0 },
                MapImport {
                    path: String::from("../assets/maps/map.0.0/sprite.json"),
                    info: String::from("Plaines"),
                },
            ),
            (
                WorldCoord { x: 1, y: 0 },
                MapImport {
                    path: String::from("../assets/maps/map.1.0/sprite.json"),
                    info: String::from("Plage cliquetante"),
                },
            ),
        ]);

        let mut world = World {
            world: HashMap::new(),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        };

        for (coord, map_import) in __world {
            let raw_data: String = fs::read_to_string(&map_import.path)
                .expect("test_map_import: Unable to read file.");
            let loaded_map = serde_json::from_str::<LoadedMap>(&raw_data)
                .expect(&format!("Fail to load JSON map: {}", &map_import.path));

            let tilesets: Vec<G2dTexture> = loaded_map
                .tilesets
                .iter()
                .map(|path| {
                    match Texture::from_path(
                        &mut window.create_texture_context(),
                        &path,
                        Flip::None,
                        &TextureSettings::new(),
                    ) {
                        Ok(texture) => texture,
                        Err(texture_error) => {
                            println!("Fail to load texture (tileset PNG): {}", texture_error);
                            std::process::exit(2);
                        }
                    }
                })
                .collect();

            let mut collider_map =
                vec![vec![false; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize];

            let frames: Vec<Frame> = loaded_map
                .frames
                .iter()
                .enumerate()
                .map(|(index, duration)| {
                    let sprites: Vec<Sprite> = loaded_map
                        .sprites
                        .iter()
                        .filter(|lsprt| lsprt.frame == index)
                        .map(|sprt| {
                            if sprt.collider {
                                let x = sprt.tile_index as usize % TILEMAP_WIDTH;
                                let y = sprt.tile_index as usize / TILEMAP_WIDTH;
                                collider_map[y][x] = sprt.collider;
                            }
                            Sprite::new(
                                tilesets[sprt.tileset_id].clone(),
                                sprt.tileset_index,
                                Offset {
                                    x: sprt.bound_x,
                                    y: sprt.bound_y,
                                },
                                sprt.tile_index,
                                sprt.collider,
                            )
                        })
                        .collect();

                    Frame::new(sprites, *duration)
                })
                .collect();

            world.world.insert(
                coord,
                MapData {
                    info: map_import.info,
                    frames: frames,
                    timer: 0,
                    f_ptr: 0,
                    colliders: collider_map,
                },
            );
        }
        return world;
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, player_world_map: &WorldCoord) {
        let map_data = self.world.get(player_world_map).unwrap();
        window.draw_2d(evnt, |ctx, gl, _device| {
            let _ = map_data.frames[map_data.f_ptr]
                .sprites
                .iter()
                .map(|sprt| {
                    let pos = sprt.get_tile_position();
                    Image::new().src_rect(sprt.get_src_rect()).draw(
                        sprt.get_texture(),
                        &DrawState::default(),
                        ctx.transform.trans(
                            self.margin.width + pos[0] * TILE_WIDTH as f64 + sprt.offset.x as f64,
                            self.margin.height + pos[1] * TILE_HEIGHT as f64 + sprt.offset.y as f64,
                        ),
                        gl,
                    );
                })
                .collect::<Vec<_>>();
        });
    }

    pub fn update(&mut self, delta_ts: u128, world_coord: &WorldCoord) {
        let map_data = self.world.get_mut(world_coord).expect(
            format!(
                "Client: world: update: trying to get map_data from : {:?}",
                world_coord
            )
            .as_str(),
        );

        let frame = &map_data.frames[map_data.f_ptr];
        if map_data.timer >= frame.duration as u128 {
            map_data.timer = 0;
            if map_data.f_ptr >= (map_data.frames.len() - 1) {
                map_data.f_ptr = 0;
            } else {
                map_data.f_ptr += 1;
            }
        } else {
            map_data.timer += delta_ts;
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }

    fn get_map(&self, coord: &WorldCoord) -> Option<(WorldCoord, &MapData)> {
        let map = match self.world.get(coord) {
            Some(map_data) => map_data,
            None => return None,
        };
        return Some((coord.clone(), map));
    }

    pub fn get_east_map(&self, coord: &WorldCoord) -> Option<(WorldCoord, &MapData)> {
        let coord_tentative = coord.clone() + WorldCoord { x: 1, y: 0 };
        self.get_map(&coord_tentative)
    }

    pub fn get_west_map(&self, coord: &WorldCoord) -> Option<(WorldCoord, &MapData)> {
        let coord_tentative = coord.clone() + WorldCoord { x: -1, y: 0 };
        self.get_map(&coord_tentative)
    }
}
