use std::collections::HashMap;
use piston_window::*;

use crate::{
    assets::{load_assets, load_hard_drown_assets, EntityAssets, GameAsset, HardTexture}, chat::Chat, client::GameData, constants::*, interface::Interface, ui::font::Font, utils::get_timestamp, world::{MapData, World}
};

pub struct Game {
    pub hard_textures: HashMap<HardTexture, G2dTexture>,
    pub assets : HashMap<EntityAssets, GameAsset>,
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
            assets: load_assets(window),
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

        self.world.update(self.delta_ts, &self.fetched_data.player.world_coord);

        self.fetched_data.player.update(self.delta_ts, &self.assets);
        for entity in self.fetched_data.entities.iter_mut() {
            entity.update(self.delta_ts, &self.assets);
        }

        self.chat.update(self.delta_ts);
    }

    pub fn key_press(&mut self, args: &Button) {
        self.chat.key_press(&args, &mut self.font);
        self.fetched_data.player.key_press(args, &self.world);
    }

    pub fn key_release(&mut self, args: &Button) {
        self.fetched_data.player.key_release(args);
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