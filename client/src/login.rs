use crate::client::{ConnectionTask, GameData};
use crate::{
    constants::*,
    loading::Loading,
    ui::{font::Font, input_field::InputField},
};
use piston_window::*;

use crate::{
    constants::{WINDOW_HEIGHT, WINDOW_WIDTH},
    game::Game,
    states::GameState,
};

pub struct Login {
    login: bool,
    pub font: Font,
    username: InputField,
    password: InputField,
    server_data: Vec<GameData>,
    margin: Size,
    background: [f64; 4],
}

const FIELD_WIDTH: f64 = 300.0;
const FIELD_HEIGHT: f64 = 35.0;
const FIELD_RADIUS: f64 = 8.0;
const FIELD_FONT_SIZE: u32 = 20;
const TITLE: &str = "Login";
const TITLE_FONT_SIZE: u32 = 28;
const LOGIN_TITLE_POS: [f64; 2] = [WINDOW_WIDTH_CENTER as f64, WINDOW_HEIGHT as f64 / 2.5];
const USERNAME_POS: [f64; 2] = [
    LOGIN_TITLE_POS[0] - FIELD_WIDTH as f64 / 2.0,
    LOGIN_TITLE_POS[1] + 20.0,
];
const PASSWORD_POS: [f64; 2] = [USERNAME_POS[0], USERNAME_POS[1] + FIELD_HEIGHT + 10.0];
const MESSAGE_FONT_SIZE: u32 = 18;
const MESSAGE_POS: [f64; 2] = [
    PASSWORD_POS[0] + FIELD_WIDTH as f64 / 2.0,
    PASSWORD_POS[1] + FIELD_HEIGHT + 50.0,
];

impl Login {
    pub fn new() -> Self {
        Login {
            login: false,
            server_data: Vec::new(),
            font: Font::new(),
            username: InputField::new(USERNAME_POS, FIELD_FONT_SIZE, FIELD_WIDTH as f64)
                .set_radius(FIELD_RADIUS)
                .set_height(FIELD_HEIGHT),
            password: InputField::new(PASSWORD_POS, FIELD_FONT_SIZE, FIELD_WIDTH as f64)
                .set_radius(FIELD_RADIUS)
                .set_height(FIELD_HEIGHT)
                .masked(true),
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            background: [0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64],
        }
    }

    pub fn connect_user(&self) -> bool {
        // Dummy check, should connect to server
        !self.username.get_content().is_empty() && !self.password.get_content().is_empty()
    }
}

impl GameState for Login {
    fn pass_server_data(&mut self, data: Vec<GameData>) {
        self.server_data = data;
        println!("{:?}", self.server_data)
    }

    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        if self.login {
            let mut game_state = Game::new(window);
            game_state.font.load(window);

            let input_login = self.username.get_content();
            let input_password = self.password.get_content();

            let mut loading_state = Loading::new(
                Box::new(game_state),
                ConnectionTask::new(input_login, input_password),
            );
            loading_state.font.load(window);
            loading_state.resize_window(&ResizeArgs {
                window_size: window.size().into(),
                draw_size: window.draw_size().into(),
            });
            return Box::new(loading_state);
        }
        self
    }

    fn render(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            clear(color::BLACK, gl);

            let rect = Rectangle::new([1.0; 4]).color(color::hex("2d2d2d"));
            let mut final_position = self.background.clone();
            final_position[0] += self.margin.width;
            final_position[1] += self.margin.height;
            rect.draw(final_position, &_ctx.draw_state, _ctx.transform, gl);
        });

        let title_width = self.font.get().width(TITLE_FONT_SIZE, TITLE).unwrap();

        self.font.render_text(
            TITLE,
            TITLE_FONT_SIZE,
            evnt,
            window,
            color::WHITE,
            [LOGIN_TITLE_POS[0] - title_width / 2.0, LOGIN_TITLE_POS[1]],
            Some(&self.margin),
        );

        let message: String = self
            .server_data
            .iter()
            .map(|e| match e {
                GameData::Message(msg) => msg.clone() + "\n",
                _ => String::default(),
            })
            .collect();

        if !message.is_empty() {
            let msg_width = self.font.get().width(MESSAGE_FONT_SIZE, &message).unwrap();
            self.font.render_text(
                &message,
                MESSAGE_FONT_SIZE,
                evnt,
                window,
                color::YELLOW,
                [MESSAGE_POS[0] - msg_width / 2.0, MESSAGE_POS[1]],
                Some(&self.margin),
            );
        }
        self.username.render(evnt, window, &mut self.font);
        self.password.render(evnt, window, &mut self.font);
    }

    fn update(&mut self, _args: &UpdateArgs, delta_ts: u128) {
        self.username.update(delta_ts);
        self.password.update(delta_ts);
    }

    fn text_input(&mut self, args: &String) {
        self.username.text_input(&args, &mut self.font);
        self.password.text_input(&args, &mut self.font);
    }

    fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.username.mouse_cursor_args(args);
        self.password.mouse_cursor_args(args);
    }

    fn key_press(&mut self, args: &Button) {
        self.username.key_press(args, &mut self.font);
        self.password.key_press(args, &mut self.font);

        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Tab => {
                    if self.username.is_focus() {
                        self.username.set_focus(false);
                        self.password.set_focus(true);
                    } else if self.password.is_focus() {
                        self.password.set_focus(false);
                        self.username.set_focus(true);
                    } else {
                        self.username.set_focus(true);
                    }
                }
                piston::Key::Escape => std::process::exit(0),
                piston::Key::Return => self.login = self.connect_user(),
                _ => {}
            }
        }
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> (Login) Resized: {window_width}x{window_height}");

        self.margin = self.handle_resize(
            Size {
                width: window_width,
                height: window_height,
            },
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );

        self.username.resize(&self.margin);
        self.password.resize(&self.margin);
    }
}
