use std::fs;
use std::collections::HashMap;
use piston_window::*;
// use opengl_graphics::{Texture, TextureSettings};
use serde::{de::{Visitor}, Deserialize, Deserializer, Serialize};

use crate::{
    constants,
    game::Game
};

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Coord {
    pub x: i32,
    pub y: i32
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Frame {
    pub tilemap_index: u16,     // The sprite number in the tilemap, define WHERE the sprite should be drawn.
    pub tileset_index: u8,      // The sprite number in the tileset, define WHICH sprite to pick in the tileset.
    pub frame_number: u8,
}

#[derive(Deserialize, Debug, Default)]
pub struct Sprite {
    pub layer_name: String,
    pub tileset: u8,
    pub collider: bool,
    pub frames: Vec<Frame>,
    pub timer: u128,
    pub frame_index: usize
}

#[derive(Deserialize, Debug)]
struct Map {
    width: u32,
    height: u32,
    #[serde(rename(deserialize = "layers"))]
    #[serde(deserialize_with = "deserialize_sprites")]
    sprites: Vec<Sprite>,
    #[serde(deserialize_with = "deserialize_frames")]
    frames: Vec<f32>,
    #[serde(deserialize_with = "deserialize_tilesets")]
    tilesets: Vec<String>
}


struct SpriteDeserializer;

impl<'de> Visitor<'de> for SpriteDeserializer {
    type Value = Vec<Sprite>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not deserialize into Sprite.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de>,
    {
        let mut sprites : Vec<Sprite> = Vec::new();

        // Iter on all layers objects
        while let Some(obj) = seq.next_element::<serde_json::Value>()? {
           
            let layer_name = obj.get("name")
                .expect("Layer name not found.")
                .to_string()
                .replace("\"", "");

            let tileset = obj.get("tileset")
                .expect("tileset not found.")
                .as_u64()
                .expect("Fail to get tileset JSON key as u64.")as u8;

            let cels = obj.get("cels")
                .expect("\"cels\" array not found in JSON.")
                .as_array()
                .expect("Fail to get cels JSON array as Vector.");

            // println!("--- Layer:\"{layer_name}\" ---");
            let mut frames : Vec<Frame> = Vec::new();
            for cel in cels {
                // dbg!(&cel);

                let frame_number = cel.get("frame")
                    .expect("\"frame\" key not found in JSON.")
                    .as_u64()
                    .expect("Fail to get frame JSON key as u64.") as u8;

                let tilemap = cel.get("tilemap")
                    .expect("\"tilemap\" not found in JSON.")
                    .as_object()
                    .expect("Fail to get tilemap JSON key as object.");

                let tiles = tilemap.get("tiles")
                    .expect("\"tiles\" array not found in JSON.")
                    .as_array()
                    .expect("Fail to get tiles JSON array as Vector.");

                frames.push(Frame {
                    frame_number:  frame_number,
                    tilemap_index: 0,
                    tileset_index: 0,
                });
                            
                let _ : Vec<_> = tiles.iter().enumerate().map(|(tile_index, tile)| {

                    let tileset_index = tile.as_u64().expect("Fail to get value from tiles JSON array as u64.") as u8;

                    // Value 0 within tilemap is consider empty
                    // It means that the sprite at this position is not define in this layer.
                    if tileset_index == 0 {
                        return
                    }

                    frames.last_mut().unwrap().tilemap_index = tile_index as u16;
                    frames.last_mut().unwrap().tileset_index = tileset_index;

                    sprites.push(Sprite {
                        collider: if layer_name == "Collider" { true } else {false },
                        layer_name: layer_name.clone(),
                        tileset: tileset,
                        frames: frames.clone(),
                        timer: 0,
                        frame_index: 0
                    });
                }).collect();

                if layer_name != "AnimatedSprites" {
                    // Load several frames only for AnimatedSprites layer
                    break ;
                }
            }
        }             

        Ok(sprites)
    }

}

fn deserialize_sprites<'de, D>(deserializer: D) -> Result<Vec<Sprite>, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(SpriteDeserializer)
}

struct FramesDeserializer;

impl<'de> Visitor<'de> for FramesDeserializer {
    type Value = Vec<f32>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not deserialize into Vec<f32>.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de>,
    {
        let mut frames : Vec<f32> = Vec::new();

        // Iter on all frames objects
        while let Some(frame_obj) = seq.next_element::<serde_json::Value>()? {
            frames.push(frame_obj.get("duration")
            .expect("duration not found.")
            .as_f64()
            .expect("Fail to get duration JSON key as f32.") as f32 * 1000.0)
        }
        return Ok(frames)
    }
}

fn deserialize_frames<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(FramesDeserializer)
}


struct TilesetsDeserializer;

impl<'de> Visitor<'de> for TilesetsDeserializer {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not deserialize into Vec<String>.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de>,
    {
        let mut tilesets : Vec<String> = Vec::new();

        // Iter on all frames objects
        while let Some(frame_obj) = seq.next_element::<serde_json::Value>()? {
            tilesets.push(format!("../assets/{}", frame_obj.get("image")
                .expect("tilesets not found.")
                .as_str()
                .expect("Fail to get tilesets image JSON key as &str.")
                .replace("\\", "/")))
        }
        return Ok(tilesets)
    }
}

fn deserialize_tilesets<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(TilesetsDeserializer)
}

pub struct MapData {
    pub info: String,
    pub sprites: Vec<Sprite>,
    pub frames: Vec<f32>,
    pub tilesets: Vec<G2dTexture>  // TODO: merge with Sprite ; Split World from Deserializer
}

struct MapImport {
    path: String,
    info: String
}

pub struct World {
    pub world: HashMap<Coord, MapData>
}

pub enum WorldError {
    UnknownMap
}

impl World {

    pub fn new(window : &mut PistonWindow) -> Self {

        let __world  = HashMap::from([
            (Coord { x:0, y:0 }, MapImport { path: String::from("../assets/map.0.0/sprite.json"), info: String::from("Plaines")}),
            (Coord { x:1, y:0 }, MapImport { path: String::from("../assets/map.1.0/sprite.json"), info: String::from("Plaines")})
        ]);

        let mut world = World {
            world: HashMap::new()
        };

        for (coord, map_import) in __world {
            let raw_data: String = fs::read_to_string(&map_import.path).expect("test_map_import: Unable to read file.");
            let loaded_map = serde_json::from_str::<Map>(&raw_data)
                .expect(&format!("Fail to load JSON map: {}", &map_import.path));

            let tilesets = loaded_map.tilesets.iter().map(|path| {
                match Texture::from_path(&mut window.create_texture_context(), &path, Flip::None, &TextureSettings::new()) {
                    Ok(texture) => texture,
                    Err(texture_error) => {
                        println!("Fail to load texture (tileset PNG): {}", texture_error);
                        std::process::exit(2);
                    }
                }
            }).collect();

            world.world.insert(coord, MapData {
                info: map_import.info,
                sprites: loaded_map.sprites,
                frames: loaded_map.frames,
                tilesets: tilesets
            });
        }
        return world;
    }

    pub fn render(&self, evnt : &Event, window: &mut PistonWindow, game: &Game) {
        let map_data = self.world.get(&game.fetched_data.player.world_coord).unwrap();
        window.draw_2d(evnt, |ctx, gl, _device| {
        let _ = map_data.sprites.iter().map(|sprite| {

            let sprite_texture = &map_data.tilesets[sprite.tileset as usize];
            let tile_number = sprite.frames[sprite.frame_index].tileset_index;

            let src_rect = [
                (tile_number as u32 % (sprite_texture.get_width() / constants::TILE_WIDTH) * constants::TILE_WIDTH) as f64,
                (tile_number as u32 / (sprite_texture.get_width() / constants::TILE_WIDTH) * constants::TILE_HEIGHT) as f64,
                constants::TILE_WIDTH as f64,
                constants::TILE_HEIGHT as f64,
            ];

            let x = (sprite.frames[sprite.frame_index].tilemap_index as u32 % constants::TILEMAP_WIDTH) as f64;
            let y = (sprite.frames[sprite.frame_index].tilemap_index as u32 / constants::TILEMAP_WIDTH) as f64;

            // TODO: delete map_img from struct Game and create Image here to draw the map
            let map_img = Image::new()
            .src_rect(src_rect)
            .draw(sprite_texture,
                &DrawState::default(),
                ctx.transform.trans(game.margin.width + x as f64 * constants::TILE_WIDTH as f64, game.margin.height + y as f64 * constants::TILE_HEIGHT as f64),
                gl);

        }).collect::<Vec<_>>();
    });
    }

    pub fn update(&mut self, delta_ts: u128, world_coord: &Coord) {
        let map_data = self.world.get_mut(world_coord).expect(format!("====> Trying to get map_data from : {:?}", world_coord).as_str());
        let _ = map_data.sprites.iter_mut().map(|sprite| {
            // When the timer for the frame reach the total duration for this frame
            // Pass to the next frame.
            if sprite.timer >= (map_data.frames[sprite.frame_index]) as u128 {
                if sprite.frame_index >= (sprite.frames.len() -1) {
                    sprite.frame_index = 0;
                } else {
                    sprite.frame_index += 1;
                }
                sprite.timer = 0;
            } else {
                sprite.timer += delta_ts;
            }
        }).collect::<Vec<_>>();
    }

    pub fn get_world_map(&self, coord: &Coord) -> Result<&MapData, WorldError> {
        match self.world.get(&coord) {
            Some(map_data) => return Ok(map_data),
            None => Err(WorldError::UnknownMap)
        }
    }
}