use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use piston_window::*;

use crate::{
    chat::Chat,
    client::GameData,
    constants::*,
    interface::Interface,
    ui::font::Font,
    utils::get_timestamp,
    world::{MapData, World}
};

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GameTexture {
    Character,
    Bouftou,
    Interface
}

fn load_hard_drown_assets(window: &mut PistonWindow) -> HashMap<GameTexture, G2dTexture> {
    // Load whole hard drown PNG interface
    let assets: Vec<&str> = vec![
        "../assets/v2/interface_1024x192_grid16.png",
        "../assets/v3/character.png"
    ];

    let loaded_assets : HashMap<GameTexture, G2dTexture> = assets.iter().map(|path| {
        let text = match Texture::from_path(&mut window.create_texture_context(), path, Flip::None, &TextureSettings::new()) {
            Ok(texture) => texture,
            Err(texture_error) => {
                println!("Fail to load hard drown texture : {} : {}", path, texture_error);
                std::process::exit(2);
            }
        };
        return match path.split("/").last().unwrap() {
            "interface_1024x192_grid16.png" => (GameTexture::Interface, text),
            "character.png" => (GameTexture::Character, text),
            _ => todo!()
        };
    }).collect();
    return loaded_assets;
}

pub struct Game {
    pub hard_textures: HashMap<GameTexture, G2dTexture>,
    pub margin: Size,
    pub world: World,
    pub interface: Interface,
    pub fetched_data: GameData,
    pub ts: u128,
    pub delta_ts: u128,
    pub font: Font,
    pub chat: Chat
}

impl Game {

    pub fn new(window: &mut PistonWindow) -> Self {
        let g_data = match GameData::get_data_from_server() {
            Ok(data) => data,
            Err(error) => {
                // TODO: should not exit
                println!("{error}");
                std::process::exit(1);
            }
        };

        let mut chat = Chat::new(String::from("Sulfurel"));
        chat.log_info("Bienvenue dans RPG!");

        return Game {
            margin: Size { width: MAP_WIDTH_CENTER as f64, height: MAP_HEIGHT_CENTER as f64 },
            hard_textures: load_hard_drown_assets(window),
            world: World::new(window),
            interface: Interface::new(),
            fetched_data: g_data,
            ts: get_timestamp(),
            delta_ts: 0,
            font: Font::new(),
            chat: chat
        }
    }

    pub fn render(&mut self, evnt : &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            clear(color::BLACK, gl);
        });

        self.world.render(evnt, window, &self);
        self.interface.render(evnt, window, &self);

        self.fetched_data.player.render(evnt, window, &self);
        for entity in self.fetched_data.entities.iter() {
            entity.render(evnt, window, &self);
        }
        
        self.chat.render(evnt, window, &mut self.font);
        self.interface.render_text_overlay(evnt, window, &mut self.font, &self.margin, &self.world.world, &self.fetched_data);
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.delta_ts = get_timestamp() - self.ts;
        self.ts = get_timestamp();

        self.chat.update(self.delta_ts);
        self.world.update(self.delta_ts, &self.fetched_data.player.world_coord);
    }

    pub fn key_press(&mut self, args: &Button) {
        self.chat.key_press(&args, &mut self.font);
    
        let map_data: &MapData = &self.world.world[&self.fetched_data.player.world_coord];
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Up => {
                    self.fetched_data.player.move_y(-1, &map_data.sprites, &self.world);
                },
                piston::Key::Down => {
                    self.fetched_data.player.move_y(1, &map_data.sprites, &self.world);
                },
                piston::Key::Left => {
                    self.fetched_data.player.move_x(-1, &map_data.sprites, &self.world);
                },
                piston::Key::Right => {
                    self.fetched_data.player.move_x(1, &map_data.sprites, &self.world);
                },
                _ => {}
           }
        }
    }

    pub fn key_release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                _ => {}
           }
        }
    }

    pub fn text_input(&mut self, args: String) {
        self.chat.text_input(args, &mut self.font);
    }

    pub fn mouse_cursor_args(&mut self, args: [f64; 2]) {
        self.chat.mouse_cursor_args(args);
    }

    pub fn mouse_scroll_args(&mut self, args: [f64; 2]) {
        self.chat.mouse_scroll_args(args);
    }

    pub fn handle_resize(&mut self, new_size: Size) {
        if new_size.width as usize >= MAP_WIDTH {
            self.margin.width = ((new_size.width as usize - MAP_WIDTH) / 2) as f64;
        } else {
            self.margin.width = 0.0;
        }

        if new_size.height as usize >= GAME_HEIGHT {
            self.margin.height = ((new_size.height as usize - GAME_HEIGHT) / 2) as f64;
        } else {
            self.margin.height = 0.0;
        }
    }

    pub fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> Resized: {window_width}x{window_height}");

        self.handle_resize(Size { width: window_width, height: window_height });
        self.interface.resize(&self.margin);
        self.chat.resize(&self.margin);
    }
}