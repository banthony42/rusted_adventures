use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum MapLayer {
    AnimatedSprites,
    Map,
    Collider,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct LoadedSprite {
    pub layer: MapLayer,
    pub bound_x: u64,
    pub bound_y: u64,
    pub tileset_id: usize,
    pub collider: bool,
    pub tile_index: u32,
    pub tileset_index: u8,
    pub frame: usize,
}

#[derive(Deserialize, Debug)]
pub struct LoadedMap {
    #[serde(rename(deserialize = "layers"))]
    #[serde(deserialize_with = "deserialize_sprites")]
    pub sprites: Vec<LoadedSprite>,
    #[serde(deserialize_with = "deserialize_frames")]
    pub frames: Vec<f32>,
    #[serde(deserialize_with = "deserialize_tilesets")]
    pub tilesets: Vec<String>,
}

struct SpriteDeserializer;

impl<'de> Visitor<'de> for SpriteDeserializer {
    type Value = Vec<LoadedSprite>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not deserialize into Sprite.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut sprites: Vec<LoadedSprite> = Vec::new();

        // Iter on all layers objects
        while let Some(obj) = seq.next_element::<serde_json::Value>()? {
            let layer_name = obj
                .get("name")
                .expect("Layer name not found.")
                .to_string()
                .replace("\"", "");

            let tileset = obj
                .get("tileset")
                .expect("tileset not found.")
                .as_u64()
                .expect("Fail to get tileset JSON key as u64.") as u8;

            let cels = obj
                .get("cels")
                .expect("\"cels\" array not found in JSON.")
                .as_array()
                .expect("Fail to get cels JSON array as Vector.");

            for cel in cels {
                let bounds = cel
                    .get("bounds")
                    .expect("\"bounds\" not found in JSON.")
                    .as_object()
                    .expect("Fail to get bounds JSON key as object.");

                let bounds_x = bounds
                    .get("x")
                    .expect("\"bounds.x\" not found in JSON.")
                    .as_u64()
                    .expect("Fail to get bounds.x JSON key as u64.");
                let bounds_y = bounds
                    .get("y")
                    .expect("\"bounds.y\" not found in JSON.")
                    .as_u64()
                    .expect("Fail to get bounds.y JSON key as u64.");

                let frame = cel
                    .get("frame")
                    .expect("\"frame\" not found in JSON.")
                    .as_u64()
                    .expect("Fail to get frame JSON key as object.");

                let tilemap = cel
                    .get("tilemap")
                    .expect("\"tilemap\" not found in JSON.")
                    .as_object()
                    .expect("Fail to get tilemap JSON key as object.");

                let tiles = tilemap
                    .get("tiles")
                    .expect("\"tiles\" array not found in JSON.")
                    .as_array()
                    .expect("Fail to get tiles JSON array as Vector.");

                let _: Vec<_> = tiles
                    .iter()
                    .enumerate()
                    .map(|(tile_index, tile)| {
                        let tileset_index = tile
                            .as_u64()
                            .expect("Fail to get value from tiles JSON array as u64.")
                            as u8;

                        // Value 0 within tilemap is consider empty
                        // It means that the sprite at this position is not define in this layer.
                        // We can't use .filter after .iter because we need that .enumerate count all the tiles
                        if tileset_index == 0 {
                            return;
                        }

                        let layer = match layer_name.as_str() {
                            "Collider" => MapLayer::Collider,
                            "AnimatedSprites" => MapLayer::AnimatedSprites,
                            "Map" => MapLayer::Map,
                            _ => panic!("Error: layer {:?} not supported.", layer_name),
                        };

                        sprites.push(LoadedSprite {
                            collider: layer == MapLayer::Collider,
                            layer: layer,
                            bound_x: bounds_x,
                            bound_y: bounds_y,
                            tileset_id: tileset as usize,
                            tile_index: tile_index as u32,
                            tileset_index: tileset_index,
                            frame: frame as usize,
                        });
                    })
                    .collect();
            }
        }

        Ok(sprites)
    }
}

fn deserialize_sprites<'de, D>(deserializer: D) -> Result<Vec<LoadedSprite>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(SpriteDeserializer)
}

struct FramesDeserializer;

impl<'de> Visitor<'de> for FramesDeserializer {
    type Value = Vec<f32>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not deserialize into Vec<f32>.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut frames: Self::Value = Vec::new();

        // Iter on all frames objects
        while let Some(frame_obj) = seq.next_element::<serde_json::Value>()? {
            frames.push(
                frame_obj
                    .get("duration")
                    .expect("duration not found.")
                    .as_f64()
                    .expect("Fail to get duration JSON key as f32.") as f32
                    * 1000.0,
            )
        }
        return Ok(frames);
    }
}

fn deserialize_frames<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(FramesDeserializer)
}

struct TilesetsDeserializer;

impl<'de> Visitor<'de> for TilesetsDeserializer {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not deserialize into Vec<String>.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut tilesets: Vec<String> = Vec::new();

        // Iter on all frames objects
        while let Some(frame_obj) = seq.next_element::<serde_json::Value>()? {
            tilesets.push(format!(
                "../assets/maps/{}",
                frame_obj
                    .get("image")
                    .expect("tilesets not found.")
                    .as_str()
                    .expect("Fail to get tilesets image JSON key as &str.")
                    .replace("\\", "/")
            ))
        }
        return Ok(tilesets);
    }
}

fn deserialize_tilesets<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(TilesetsDeserializer)
}
