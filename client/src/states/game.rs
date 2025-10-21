use piston_window::*;
use std::collections::HashMap;

use crate::{
    chat::controller::ChatController,
    entities::controller::EntityController,
    import::assets::{load_assets, load_hard_drawn_assets, UITexture},
    interface::Interface,
    tasks::{logout::LogoutTask, task::GameData},
    ui::font::Font,
    world::World,
    GameState,
};
use common::constants::*;

use super::{
    loading::{Loading, LoadingNextState},
    states::StateFactory,
};

pub struct Game {
    pub hard_textures: HashMap<UITexture, G2dTexture>,
    pub margin: Size,
    pub world: World,
    pub interface: Interface,
    pub font: Font,
    pub chat_controller: ChatController,
    pub ent_controller: EntityController,
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
            hard_textures: load_hard_drawn_assets(window),
            margin: Size {
                width: MAP_WIDTH_CENTER as f64,
                height: MAP_HEIGHT_CENTER as f64,
            },
            world: World::new(window),
            interface: Interface::new(),
            // TODO: this cause tentative gRPC connection with empty login/token
            // "Authenticator: Fail to get token for: "
            chat_controller: ChatController::new(String::default(), String::default()),
            ent_controller: EntityController::new(load_assets(window)),
        };
    }
}

impl GameState for Game {
    fn pass_data(&mut self, data: Vec<GameData>) {
        data.iter().for_each(|gd| match gd {
            GameData::Login(login) => self.login = login.clone(),
            GameData::Token(token) => self.token = token.clone(),
            GameData::Message(_) => {}
            GameData::Player(player) => self.ent_controller.set_player(player),
            GameData::Entities(entities) => self.ent_controller.set_entities(entities),
        });
        println!("Client: Game: Player connected: {:?}", data);
        self.ent_controller
            .init(self.login.clone(), self.token.clone());
        self.chat_controller = ChatController::new(self.login.clone(), self.token.clone());
        self.chat_controller.resize(&self.margin);
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
            .render(evnt, window, &self.ent_controller.player_map());
        self.ent_controller.render(evnt, window, &mut self.font);
        self.interface.render(evnt, window, &self.hard_textures);
        self.chat_controller.render(evnt, window, &mut self.font);
    }

    fn update(&mut self, _args: &UpdateArgs, delta_ts: u128) {
        self.world
            .update(delta_ts, &self.ent_controller.player_map());
        self.ent_controller.update(delta_ts, &self.world);
        self.interface.update(_args, delta_ts);
        // Get players position from ent_controller and pass it to chat_controller.update
        self.chat_controller
            .update(delta_ts, self.ent_controller.ui_entities_models());
    }

    fn key_press(&mut self, args: &Button) {
        self.ent_controller.key_press(args, &self.world.world);
        self.chat_controller.key_press(&args, &mut self.font);

        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Escape => self.logout = true,
                _ => {}
            }
        }
    }

    fn key_release(&mut self, _args: &Button) {}

    fn text_input(&mut self, args: &String) {
        self.chat_controller.text_input(args, &mut self.font);
    }

    fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.ent_controller.mouse_cursor_args(args);
        self.chat_controller.mouse_cursor_args(args);
    }

    fn mouse_scroll_args(&mut self, args: &[f64; 2]) {
        self.chat_controller.mouse_scroll_args(args);
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_size = Size {
            width: args.window_size[0],
            height: args.window_size[1],
        };
        println!("Client: Game: Resized: {:?}", window_size);
        self.margin = self.handle_resize(window_size, MAP_WIDTH, GAME_HEIGHT);
        self.world.resize(&self.margin);
        self.ent_controller.resize(&self.margin);
        self.interface.resize(&self.margin);
        self.chat_controller.resize(&self.margin);
    }
}
