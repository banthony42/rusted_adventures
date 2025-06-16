use piston_window::*;
use std::collections::HashMap;

use crate::{
    chat::controller::ChatController,
    constants::*,
    entities::controller::EntityController,
    import::assets::{load_assets, load_hard_drown_assets, HardTexture},
    interface::Interface,
    tasks::{logout::LogoutTask, task::GameData},
    ui::font::Font,
    world::World,
    GameState,
};

use super::{
    loading::{Loading, LoadingNextState},
    states::StateFactory,
};

pub struct Game {
    pub hard_textures: HashMap<HardTexture, G2dTexture>,
    pub margin: Size,
    pub world: World,
    pub interface: Interface,
    pub font: Font,
    pub chat: ChatController,
    pub entities: EntityController,
    pub logout: bool,
    pub login: String,
    pub token: String,
}

impl Game {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut font = Font::new();
        font.load(window);

        return Game {
            login: String::default(),
            token: String::default(),
            logout: false,
            font: font,
            hard_textures: load_hard_drown_assets(window),
            margin: Size {
                width: MAP_WIDTH_CENTER as f64,
                height: MAP_HEIGHT_CENTER as f64,
            },
            world: World::new(window),
            interface: Interface::new(),
            chat: ChatController::new(String::default(), String::default()),
            entities: EntityController::new(
                String::default(),
                String::default(),
                load_assets(window),
            ),
        };
    }
}

impl GameState for Game {
    fn pass_data(&mut self, data: Vec<GameData>) {
        data.iter().for_each(|gd| match gd {
            GameData::Token(token) => self.token = token.clone(),
            GameData::Message(name) => self.login = name.clone(), //TMP use message to get player name
            GameData::Entities(_vec) => {}
        });
        println!("==> (Game) Player connected: {:?}", data);
        self.chat = ChatController::new(self.login.clone(), self.token.clone());
        self.chat.resize(&self.margin);
    }

    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        if self.logout {
            return StateFactory::<Loading>::new(
                window,
                LoadingNextState::Login,
                Box::new(LogoutTask::new(self.login, self.token)),
            );
        }
        self
    }

    fn render(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            clear(color::BLACK, gl);
        });

        self.world
            .render(evnt, window, &self.entities.player_world());
        self.entities.render(evnt, window);
        self.interface.render(evnt, window, &self);
        self.chat.render(evnt, window, &mut self.font);
    }

    fn update(&mut self, _args: &UpdateArgs, delta_ts: u128) {
        self.world.update(delta_ts, &self.entities.player_world());
        self.entities.update(delta_ts, &self.world);
        self.interface.update(_args, delta_ts);
        self.chat.update(delta_ts);
    }

    fn key_press(&mut self, args: &Button) {
        self.entities.key_press(args, &self.world.world);
        self.chat.key_press(&args, &mut self.font);

        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Escape => self.logout = true,
                _ => {}
            }
        }
    }

    fn key_release(&mut self, _args: &Button) {}

    fn text_input(&mut self, args: &String) {
        self.chat.text_input(args, &mut self.font);
    }

    fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.entities.mouse_cursor_args(args);
        self.chat.mouse_cursor_args(args);
    }

    fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.chat.mouse_scroll_args(args);
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_size = Size {
            width: args.window_size[0],
            height: args.window_size[1],
        };
        println!("==> (Game) Resized: {:?}", window_size);
        self.margin = self.handle_resize(window_size, MAP_WIDTH, GAME_HEIGHT);
        self.world.resize(&self.margin);
        self.entities.resize(&self.margin);
        self.interface.resize(&self.margin);
        self.chat.resize(&self.margin);
    }
}
