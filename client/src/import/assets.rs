use common::constants::Species;
use graphics::types::Color;
use piston_window::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default, Debug)]
struct JSONAssetRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Deserialize, Debug)]
struct JSONAssetFrame {
    frame: JSONAssetRect,
    duration: u128,
}

#[derive(Deserialize, Debug)]
struct JSONAssetMeta {
    image: String,
}

#[derive(Deserialize, Debug)]
struct JSONGameAsset {
    frames: Vec<JSONAssetFrame>,
    meta: JSONAssetMeta,
}

#[derive(Debug, Default)]
pub struct AssetFrame {
    pub src_rect: [f64; 4],
    pub duration: u128,
}

#[derive(Debug)]
pub struct GameAsset {
    pub texture: G2dTexture,
    pub frames: Vec<AssetFrame>,
}

impl GameAsset {
    fn load_game_asset_from_json(
        json_game_asset: JSONGameAsset,
        base_folder: &Path,
        window: &mut PistonWindow,
    ) -> GameAsset {
        let frames: Vec<AssetFrame> = json_game_asset
            .frames
            .iter()
            .map(|json_frame| {
                return AssetFrame {
                    src_rect: [
                        json_frame.frame.x,
                        json_frame.frame.y,
                        json_frame.frame.w,
                        json_frame.frame.h,
                    ],
                    duration: json_frame.duration,
                };
            })
            .collect();

        let full_path = base_folder.join(json_game_asset.meta.image);
        let texture = match Texture::from_path(
            &mut window.create_texture_context(),
            &full_path,
            Flip::None,
            &TextureSettings::new(),
        ) {
            Ok(texture) => texture,
            Err(error) => {
                println!(
                    "Fail to load texture ({:?} PNG): {}",
                    full_path.to_str(),
                    error
                );
                std::process::exit(2);
            }
        };

        return GameAsset {
            texture: texture,
            frames: frames,
        };
    }

    pub fn load_asset(path: &Path, window: &mut PistonWindow) -> GameAsset {
        let parent = path
            .parent()
            .expect(&format!("Fail to get parent dir of: {:?}", path.to_str()));

        let raw_data: String = fs::read_to_string(path).expect("load_asset: Unable to read file.");

        let json_game_asset = serde_json::from_str::<JSONGameAsset>(&raw_data)
            .expect(&format!("Fail to load JSON asset: {:?}", path.to_str()));

        return Self::load_game_asset_from_json(json_game_asset, parent, window);
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Animations {
    #[default]
    Idle,
    Run,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum EntityAssets {
    Character(Animations),
    Warrior(Animations),
    Mage(Animations),
    Bouftou(Animations),
    Crabedoeuf(Animations),
}

impl Default for EntityAssets {
    fn default() -> Self {
        Self::Character(Animations::default())
    }
}

pub struct SpeciesConstants {
    pub font_color: Color,
    // See notes about render_offest below
    pub _render_offset_x: f64,
    pub render_offset_y: f64,
}

pub struct SpeciesLibrary(HashMap<Species, SpeciesConstants>);

impl SpeciesLibrary {
    pub fn new() -> Self {
        SpeciesLibrary(HashMap::from([
            (
                Species::Warrior,
                SpeciesConstants {
                    font_color: color::hex("0017ad"),
                    _render_offset_x: 0.0,
                    render_offset_y: 64.0,
                },
            ),
            (
                Species::Mage,
                SpeciesConstants {
                    font_color: color::hex("0017ad"),
                    _render_offset_x: 0.0,
                    render_offset_y: 64.0,
                },
            ),
            (
                Species::Bouftou,
                SpeciesConstants {
                    font_color: color::hex("c5c9e8"),
                    _render_offset_x: 0.0,
                    render_offset_y: 0.0,
                },
            ),
            (
                Species::Crabedoeuf,
                SpeciesConstants {
                    font_color: color::hex("c5c9e8"),
                    _render_offset_x: 0.0,
                    render_offset_y: 0.0,
                },
            ),
        ]))
    }

    fn _get_species_constants(&self, species: &Species) -> &SpeciesConstants {
        self.0
            .get(&species)
            .expect(format!("SpeciesLibrary: Unknow species: {:?}", species).as_str())
    }

    pub fn get_font_color(&self, species: &Species) -> Color {
        self._get_species_constants(species).font_color.clone()
    }

    pub fn get_height_offset(&self, species: &Species) -> f64 {
        self._get_species_constants(species).render_offset_y
    }
}

// render_offset:
// Sprites are render at their position, and drawn from top to bottom
// Knowing that we need offsets for sprites higher and or larger than one tile.

pub fn load_assets(window: &mut PistonWindow) -> HashMap<EntityAssets, GameAsset> {
    let assets_list = vec![
        (
            EntityAssets::Warrior(Animations::Idle),
            Path::new("../assets/characters/warrior-Idle.json"),
        ),
        (
            EntityAssets::Warrior(Animations::Run),
            Path::new("../assets/characters/warrior-Run.json"),
        ),
        (
            EntityAssets::Mage(Animations::Idle),
            Path::new("../assets/characters/mage-Idle.json"),
        ),
        (
            EntityAssets::Mage(Animations::Run),
            Path::new("../assets/characters/mage-Run.json"),
        ),
        (
            EntityAssets::Character(Animations::Idle),
            Path::new("../assets/tests/character_animation-Idle.json"),
        ),
        (
            EntityAssets::Character(Animations::Run),
            Path::new("../assets/tests/character_animation-Run.json"),
        ),
        (
            EntityAssets::Bouftou(Animations::Idle),
            Path::new("../assets/tests/bouftou_animation-Idle.json"),
        ),
        (
            EntityAssets::Bouftou(Animations::Run),
            Path::new("../assets/tests/bouftou_animation-Run.json"),
        ),
        (
            EntityAssets::Crabedoeuf(Animations::Idle),
            Path::new("../assets/tests/crabedoeuf-Idle.json"),
        ),
        (
            EntityAssets::Crabedoeuf(Animations::Run),
            Path::new("../assets/tests/crabedoeuf-Run.json"),
        ),
    ];

    return assets_list
        .iter()
        .map(|asset_data| {
            return (asset_data.0, GameAsset::load_asset(asset_data.1, window));
        })
        .collect();
}

// Temporary Hard loaded textures

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum HardTexture {
    Interface,
}

pub fn load_hard_drown_assets(window: &mut PistonWindow) -> HashMap<HardTexture, G2dTexture> {
    // Load whole hard drown PNG interface
    let assets: Vec<&str> = vec!["../assets/interface/interface_1024x192_grid16.png"];

    let loaded_assets: HashMap<HardTexture, G2dTexture> = assets
        .iter()
        .map(|path| {
            let text = match Texture::from_path(
                &mut window.create_texture_context(),
                path,
                Flip::None,
                &TextureSettings::new(),
            ) {
                Ok(texture) => texture,
                Err(texture_error) => {
                    println!(
                        "Fail to load hard drown texture : {} : {}",
                        path, texture_error
                    );
                    std::process::exit(2);
                }
            };
            return match path.split("/").last().unwrap() {
                "interface_1024x192_grid16.png" => (HardTexture::Interface, text),
                _ => todo!(),
            };
        })
        .collect();
    return loaded_assets;
}
