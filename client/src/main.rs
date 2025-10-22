use std::path::Path;

use common::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use common::utils::get_timestamp;
use piston_window::*;
use states::{
    login::Login,
    states::{GameState, StateFactory},
};

use crate::interface::load_icon;

mod chat;
mod entities;
mod import;
mod interface;
mod sprite;
mod states;
mod tasks;
mod ui;
mod world;

fn run_game() {
    let opengl = OpenGL::V4_5;

    // Create a Glutin window.
    let mut window: PistonWindow =
        match WindowSettings::new("rpg", [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32])
            .graphics_api(opengl)
            .fullscreen(false)
            .exit_on_esc(false)
            .resizable(true)
            .build()
        {
            Ok(window) => window,
            Err(window_error) => {
                println!("Fail to create Glutin Window: {}", window_error);
                std::process::exit(2);
            }
        };

    window
        .window
        .ctx
        .window()
        .set_window_icon(Some(load_icon(Path::new("../assets/interface/logo.png"))));

    let mut state: Box<dyn GameState> = StateFactory::<Login>::new(&mut window, None);
    let mut ts: u128 = 0;

    while let Some(e) = window.next() {
        state.render(&e, &mut window);
        state.font_flush(&e, &mut window);

        if let Some(args) = e.press_args() {
            state.key_press(&args);
        }

        if let Some(args) = e.release_args() {
            state.key_release(&args);
        }

        if let Some(args) = e.resize_args() {
            state.resize_window(&args);
        }

        if let Some(args) = e.update_args() {
            state.update(&args, get_timestamp().saturating_sub(ts));
            ts = get_timestamp();
        }

        if let Some(args) = e.text_args() {
            state.text_input(&args);
        }

        if let Some(args) = e.mouse_cursor_args() {
            state.mouse_cursor_args(&args);
        }

        if let Some(args) = e.mouse_scroll_args() {
            state.mouse_scroll_args(&args);
        }
        state = state.state_update(&mut window);
    }
}

fn main() {
    run_game();
}
