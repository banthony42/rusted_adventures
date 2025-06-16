use piston_window::*;
use std::fs;
use std::ops::MulAssign;
use std::{collections::HashMap, ops::SubAssign};

use crate::{
    constants::*,
    import::tilemap::LoadedMap,
    sprite::{Frame, Sprite},
};

#[derive(Default)]
pub struct Offset {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapCoord {
    pub x: i64,
    pub y: i64,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WorldCoord {
    pub x: i8,
    pub y: i8,
}

impl MapCoord {
    /// Get linear index of this Map coordinate.
    /// Then it can be use to retrieve the same pixel in an 1D array,
    /// where its size is TILEMAP_LINEAR_SIZE (TILEMAP_WIDTH * TILEMAP_HEIGHT)
    pub fn linear_index(&self) -> usize {
        self.limit();
        self.y as usize * TILEMAP_WIDTH + self.x as usize
    }

    /// Limit this Map Coordinate to TILEMAP limits.
    pub fn limit(mut self) -> Self {
        self.x = self.x.max(0).min(TILEMAP_WIDTH as i64 - 1);
        self.y = self.y.max(0).min(TILEMAP_HEIGHT as i64 - 1);
        self
    }

    pub fn min(mut self, rhs: Self) -> Self {
        self.x = self.x.min(rhs.x);
        self.y = self.y.min(rhs.y);
        self
    }

    pub fn is_null(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}

// #[derive(Debug, Clone, Copy, Default)]
// struct Point {
//     pub x: f64,
//     pub y: f64,
// }

// impl Point {
//     pub fn is_null(&self) -> bool {
//         return self.x == 0.0 && self.y == 0.0;
//     }
// }

// impl Into<MapCoord> for Point {
//     fn into(self) -> MapCoord {
//         MapCoord {
//             x: self.x as i16,
//             y: self.y as i16,
//         }
//     }
// }

// impl std::ops::Mul<f64> for Point {
//     type Output = Self;

//     fn mul(self, rhs: f64) -> Self::Output {
//         Self {
//             x: self.x * rhs,
//             y: self.y * rhs,
//         }
//     }
// }

impl std::ops::Mul<f64> for MapCoord {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: (self.x as f64 * rhs) as i64,
            y: (self.y as f64 * rhs) as i64,
        }
    }
}

impl MulAssign<f64> for MapCoord {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl std::ops::Sub for MapCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for MapCoord {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Add for MapCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for MapCoord {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Add for WorldCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for WorldCoord {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
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
