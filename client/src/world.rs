use piston_window::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::{
    constants,
    import::tilemap::LoadedMap,
    sprite::{Frame, Sprite},
    states::game::Game,
};

pub struct Offset {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug, Clone, Copy, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Coord_tmp {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug)]
pub struct WorldCoord {
    pub map: Coord_tmp,
    pub world: Coord_tmp,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn is_null(&self) -> bool {
        return self.x == 0.0 && self.y == 0.0;
    }
}

impl Into<Coord_tmp> for Point {
    fn into(self) -> Coord_tmp {
        Coord_tmp {
            x: self.x as i32,
            y: self.y as i32,
        }
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Add for Coord_tmp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for Coord_tmp {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

pub struct MapData {
    pub info: String,
    pub frames: Vec<Frame>,
    pub timer: u128,
    pub f_ptr: usize,
}

struct MapImport {
    path: String,
    info: String,
}

pub struct World {
    pub world: HashMap<Coord_tmp, MapData>,
}

impl World {
    pub fn new(window: &mut PistonWindow) -> Self {
        let __world = HashMap::from([
            (
                Coord_tmp { x: 0, y: 0 },
                MapImport {
                    path: String::from("../assets/maps/map.0.0/sprite.json"),
                    info: String::from("Plaines"),
                },
            ),
            (
                Coord_tmp { x: 1, y: 0 },
                MapImport {
                    path: String::from("../assets/maps/map.1.0/sprite.json"),
                    info: String::from("Plage cliquetante"),
                },
            ),
        ]);

        let mut world = World {
            world: HashMap::new(),
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
                },
            );
        }
        return world;
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, game: &Game) {
        let map_data = self.world.get(&game.fake_gdata.player.world_coord).unwrap();
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
                            game.margin.width
                                + pos[0] as f64 * constants::TILE_WIDTH as f64
                                + sprt.offset.x as f64,
                            game.margin.height
                                + pos[1] as f64 * constants::TILE_HEIGHT as f64
                                + sprt.offset.y as f64,
                        ),
                        gl,
                    );
                })
                .collect::<Vec<_>>();
        });
    }

    pub fn update(&mut self, delta_ts: u128, world_coord: &Coord_tmp) {
        let map_data = self
            .world
            .get_mut(world_coord)
            .expect(format!("==> Trying to get map_data from : {:?}", world_coord).as_str());

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

    fn get_map(&self, coord: &Coord_tmp) -> Option<(Coord_tmp, &MapData)> {
        let map = match self.world.get(coord) {
            Some(map_data) => map_data,
            None => return None,
        };
        return Some((coord.clone(), map));
    }

    pub fn get_east_map(&self, coord: &Coord_tmp) -> Option<(Coord_tmp, &MapData)> {
        let coord_tentative = coord.clone() + Coord_tmp { x: 1, y: 0 };
        self.get_map(&coord_tentative)
    }

    pub fn get_west_map(&self, coord: &Coord_tmp) -> Option<(Coord_tmp, &MapData)> {
        let coord_tentative = coord.clone() + Coord_tmp { x: -1, y: 0 };
        self.get_map(&coord_tentative)
    }
}
