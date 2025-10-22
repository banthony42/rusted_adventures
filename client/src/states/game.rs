use piston_window::*;
use std::collections::HashMap;

use crate::{
    chat::controller::ChatController,
    entities::controller::EntityController,
    import::assets::{load_hard_drawn_assets, UITexture},
    interface::Interface,
    tasks::logout::LogoutTask,
    ui::font::Font,
    world::World,
    GameState,
};
use common::constants::*;

use super::{
    loading::{Loading, LoadingNextState},
    states::StateFactory,
};

#[derive(Clone)]
pub struct GameCredentials {
    pub login: String,
    pub token: String,
}

impl GameCredentials {
    /// Consume self returning (login: String, token: String)
    pub fn into_parts(self) -> (String, String) {
        (self.login, self.token)
    }
}

pub struct Game {
    pub hard_textures: HashMap<UITexture, G2dTexture>,
    pub margin: Size,
    pub world: World,
    pub interface: Interface,
    pub font: Font,
    pub chat_controller: ChatController,
    pub ent_controller: EntityController,
    pub logout: bool,
    pub credentials: GameCredentials,
}

impl Game {
    pub fn new(
        window: &mut PistonWindow,
        credentials: GameCredentials,
        chat: ChatController,
        entities: EntityController,
    ) -> Self {
        let mut font = Font::new();
        font.load(window);

        return Game {
            credentials,
            logout: false,
            font: font,
            hard_textures: load_hard_drawn_assets(window),
            margin: Size {
                width: MAP_WIDTH_CENTER as f64,
                height: MAP_HEIGHT_CENTER as f64,
            },
            world: World::new(window),
            interface: Interface::new(),
            chat_controller: chat,
            ent_controller: entities,
        };
    }
}

impl GameState for Game {
    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        if self.logout {
            let (login, token) = self.credentials.into_parts();
            return StateFactory::<Loading>::new(
                window,
                LoadingNextState::Login,
                Box::new(LogoutTask::new(login, token)),
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

    fn get_font(&mut self) -> &mut Font {
        &mut self.font
    }
}
