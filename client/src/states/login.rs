use crate::tasks::connection::ConnectionTask;
use crate::tasks::task::GameData;
use crate::ui::{font::Font, input_field::InputField};
use common::constants::*;
use piston_window::*;

use common::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};

use super::loading::{Loading, LoadingNextState};
use super::states::{GameState, StateFactory};

pub struct Login {
    login: bool,
    pub font: Font,
    username: InputField,
    password: InputField,
    margin: Size,
    message: String,
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
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut font = Font::new();
        font.load(window);

        Login {
            login: false,
            font: font,
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
            message: String::default(),
            background: [0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64],
        }
    }

    pub fn should_connect_user(&mut self) -> bool {
        if self.username.get_content().is_empty() || self.password.get_content().is_empty() {
            self.message = String::from("Please, enter login and password to connect.");
            return false;
        }
        return true;
    }
}

impl GameState for Login {
    fn pass_data(&mut self, data: Vec<GameData>) {
        self.message = data
            .iter()
            .map(|e| match e {
                GameData::Message(msg) => msg.clone() + "\n",
                _ => String::default(),
            })
            .collect();
    }

    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        if self.login {
            return StateFactory::<Loading>::new(
                window,
                LoadingNextState::Game,
                Box::new(ConnectionTask::new(
                    self.username.get_content(),
                    self.password.get_content(),
                )),
            );
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

        self.font.render_centered(
            TITLE,
            TITLE_FONT_SIZE,
            evnt,
            window,
            color::WHITE,
            [LOGIN_TITLE_POS[0], LOGIN_TITLE_POS[1]],
            Some(&self.margin),
        );

        if !self.message.is_empty() {
            self.font.render_centered(
                &self.message,
                MESSAGE_FONT_SIZE,
                evnt,
                window,
                color::YELLOW,
                [MESSAGE_POS[0], MESSAGE_POS[1]],
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
                piston::Key::Return => self.login = self.should_connect_user(),
                _ => {}
            }
        }
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("Client: Login: Resized: {window_width}x{window_height}");

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
