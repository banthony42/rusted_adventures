use piston_window::*;
use std::collections::HashMap;

use crate::{
    assets::{load_assets, load_hard_drown_assets, EntityAssets, GameAsset, HardTexture},
    chat::Chat,
    client::FakeGameData,
    constants::*,
    interface::Interface,
    states::StateFactory,
    tasks::task::GameData,
    ui::font::Font,
    world::World,
    GameState, Login,
};

pub struct Game {
    pub hard_textures: HashMap<HardTexture, G2dTexture>,
    pub assets: HashMap<EntityAssets, GameAsset>,
    pub margin: Size,
    pub world: World,
    pub interface: Interface,
    pub fake_gdata: FakeGameData,
    pub server_data: Vec<GameData>,
    pub font: Font,
    pub chat: Chat,
    pub logout: bool,
}

impl Game {
    pub fn new(window: &mut PistonWindow) -> Self {
        let fg_data = match FakeGameData::get_data_from_server() {
            Ok(data) => data,
            Err(error) => {
                // TODO: should not exit
                println!("{error}");
                std::process::exit(1);
            }
        };

        let mut font = Font::new();
        font.load(window);

        return Game {
            logout: false,
            margin: Size {
                width: MAP_WIDTH_CENTER as f64,
                height: MAP_HEIGHT_CENTER as f64,
            },
            hard_textures: load_hard_drown_assets(window),
            assets: load_assets(window),
            world: World::new(window),
            interface: Interface::new(),
            fake_gdata: fg_data,
            server_data: Vec::new(), // TODO: will replace fg_data
            font: font,
            chat: Chat::new(String::from("unknown")),
        };
    }
}

impl GameState for Game {
    fn pass_data(&mut self, data: Vec<GameData>) {
        data.iter().for_each(|gd| match gd {
            GameData::Token(_) => {}
            GameData::Message(name) => self.fake_gdata.player.name = name.clone(), //TMP use message to get player name
            GameData::Entities(_vec) => {}
        });
        self.server_data = data;
        println!("==> (Game) Player connected: {:?}", self.server_data);
        // TODO: get rid of this, should be handle at Game instanciation
        self.chat = Chat::new(self.fake_gdata.player.name.clone());
        self.chat.resize(&self.margin);
    }

    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        if self.logout {
            return StateFactory::<Login>::new(window);
        }
        self
    }

    fn render(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            clear(color::BLACK, gl);
        });

        self.world.render(evnt, window, &self);
        self.interface.render(evnt, window, &self);

        self.fake_gdata.player.render(evnt, window, &self);
        for entity in self.fake_gdata.entities.iter() {
            entity.render(evnt, window, &self);
        }

        self.chat.render(evnt, window, &mut self.font);
        self.interface.render_text_overlay(
            evnt,
            window,
            &mut self.font,
            &self.margin,
            &self.world.world,
            &self.fake_gdata,
        );
    }

    fn update(&mut self, _args: &UpdateArgs, delta_ts: u128) {
        self.world
            .update(delta_ts, &self.fake_gdata.player.world_coord);

        self.fake_gdata
            .player
            .update(delta_ts, &self.assets, &self.world);
        for entity in self.fake_gdata.entities.iter_mut() {
            entity.update(delta_ts, &self.assets, &self.world);
        }

        self.chat.update(delta_ts);
    }

    fn key_press(&mut self, args: &Button) {
        self.chat.key_press(&args, &mut self.font);
        self.fake_gdata.player.key_press(args);

        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Escape => self.logout = true,
                _ => {}
            }
        }
    }

    fn key_release(&mut self, args: &Button) {
        self.fake_gdata.player.key_release(args);
    }

    fn text_input(&mut self, args: &String) {
        self.chat.text_input(args, &mut self.font);
    }

    fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.chat.mouse_cursor_args(args);
    }

    fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.chat.mouse_scroll_args(args);
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> (Game) Resized: {window_width}x{window_height}");

        self.margin = self.handle_resize(
            Size {
                width: window_width,
                height: window_height,
            },
            MAP_WIDTH,
            GAME_HEIGHT,
        );
        self.interface.resize(&self.margin);
        self.chat.resize(&self.margin);
    }
}
