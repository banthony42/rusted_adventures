use piston_window::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

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
    fn load_from_json_game_asset(
        json_game_asset: JSONGameAsset,
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

        let texture = match Texture::from_path(
            &mut window.create_texture_context(),
            format!("../assets/tests/{}", json_game_asset.meta.image.clone()),
            Flip::None,
            &TextureSettings::new(),
        ) {
            Ok(texture) => texture,
            Err(error) => {
                println!(
                    "Fail to load texture ({} PNG): {}",
                    json_game_asset.meta.image, error
                );
                std::process::exit(2);
            }
        };

        return GameAsset {
            texture: texture,
            frames: frames,
        };
    }

    pub fn load_asset(path: &str, window: &mut PistonWindow) -> GameAsset {
        let raw_data: String = fs::read_to_string(path).expect("load_asset: Unable to read file.");

        let json_game_asset = serde_json::from_str::<JSONGameAsset>(&raw_data)
            .expect(&format!("Fail to load JSON asset: {}", path));

        return Self::load_from_json_game_asset(json_game_asset, window);
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Animations {
    Idle,
    Run,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum EntityAssets {
    Character(Animations),
    Bouftou(Animations),
}

pub fn load_assets(window: &mut PistonWindow) -> HashMap<EntityAssets, GameAsset> {
    let assets_list = vec![
        (
            EntityAssets::Character(Animations::Idle),
            "../assets/tests/character_animation-Idle.json",
        ),
        (
            EntityAssets::Character(Animations::Run),
            "../assets/tests/character_animation-Run.json",
        ),
        (
            EntityAssets::Bouftou(Animations::Idle),
            "../assets/tests/bouftou_animation-Idle.json",
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
