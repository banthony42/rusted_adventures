use piston_window::*;
use std::collections::HashMap;

use common::{
    constants::*,
    world::{tilemap::LoadedMap, ColliderMap, WorldImport},
    MapCoord,
};

use crate::sprite::{Frame, Offset, Sprite};

pub struct Frames(Vec<Frame>);

impl Frames {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_frame(&self, index: usize) -> &Frame {
        &self.0[index]
    }

    pub fn from(loaded_map: &LoadedMap, tilesets: Vec<G2dTexture>) -> Self {
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
        Frames(frames)
    }
}

pub struct MapData {
    info: String,
    frames: Frames,
    timer: u128,
    frame_index: usize,
    pub collider_map: ColliderMap,
}

impl MapData {
    /// Increment frame index, reseting it when frames len is reached.
    pub fn increment_frame_index(&mut self) {
        if self.frame_index >= (self.frames.len() - 1) {
            self.frame_index = 0;
        } else {
            self.frame_index += 1;
        }
    }
}
pub struct World {
    pub world: HashMap<MapCoord, MapData>,
    margin: Size,
}

impl World {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut world = World {
            world: HashMap::new(),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        };
        let world_importer = WorldImport::new();

        for (map_coord, raw_map) in world_importer.atlas {
            let tilesets: Vec<G2dTexture> = raw_map
                .loaded_map
                .tilesets
                .iter()
                .map(|path| {
                    Texture::from_path(
                        &mut window.create_texture_context(),
                        &path,
                        Flip::None,
                        &TextureSettings::new(),
                    )
                    .expect("Fail to load texture (tileset PNG): {}")
                })
                .collect();

            world.world.insert(
                map_coord,
                MapData {
                    info: raw_map.info,
                    frames: Frames::from(&raw_map.loaded_map, tilesets),
                    collider_map: raw_map.collider_map,
                    timer: 0,
                    frame_index: 0,
                },
            );
        }
        return world;
    }

    pub fn render(&self, evnt: &Event, window: &mut PistonWindow, coord: &MapCoord) {
        if let Some(map_data) = self.world.get(coord) {
            window.draw_2d(evnt, |ctx, gl, _device| {
                map_data
                    .frames
                    .get_frame(map_data.frame_index)
                    .sprites
                    .iter()
                    .for_each(|sprt| {
                        let pos = sprt.get_tile_position();
                        Image::new().src_rect(sprt.get_src_rect()).draw(
                            sprt.get_texture(),
                            &DrawState::default(),
                            ctx.transform.trans(
                                self.margin.width
                                    + pos[0] * TILE_WIDTH as f64
                                    + sprt.offset.x as f64,
                                self.margin.height
                                    + pos[1] * TILE_HEIGHT as f64
                                    + sprt.offset.y as f64,
                            ),
                            gl,
                        );
                    });
            });
        }
    }

    pub fn update(&mut self, delta_ts: u128, coord: &MapCoord) {
        let map_data = self.world.get_mut(coord).expect(
            format!(
                "Client: world: update: trying to get map_data from : {:?}",
                coord
            )
            .as_str(),
        );

        let frame = map_data.frames.get_frame(map_data.frame_index);
        if map_data.timer >= frame.duration as u128 {
            map_data.timer = 0;
            map_data.increment_frame_index();
        } else {
            map_data.timer += delta_ts;
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
    }

    fn get_map(&self, coord: MapCoord) -> Option<(MapCoord, &MapData)> {
        let map = match self.world.get(&coord) {
            Some(map_data) => map_data,
            None => return None,
        };
        return Some((coord, map));
    }

    pub fn get_east_map(&self, coord: MapCoord) -> Option<(MapCoord, &MapData)> {
        self.get_map(coord + MapCoord { x: 1, y: 0 })
    }

    pub fn get_west_map(&self, coord: MapCoord) -> Option<(MapCoord, &MapData)> {
        self.get_map(coord + MapCoord { x: -1, y: 0 })
    }
}
