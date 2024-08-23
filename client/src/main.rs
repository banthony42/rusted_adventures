use piston_window::*;

mod constants;
mod game;
mod entity;
mod world;
mod client;
mod utils;
mod interface;
mod ui;
mod chat;

use game::Game;

fn run_game() {
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: PistonWindow =  match WindowSettings::new("rpg", [constants::WINDOW_WIDTH as u32, constants::WINDOW_HEIGHT as u32])
        .graphics_api(opengl)
        .fullscreen(false)
        .exit_on_esc(true)
        .resizable(true)
        .build() {
            Ok(window) => window,
            Err(window_error) => {
                println!("Fail to create Glutin Window: {}", window_error);
                std::process::exit(2);
            }
        };

    // Create a new game and run it.
    let mut game = Game::new(&mut window);

    game.font.load(&mut window);
    game.resize_window(&ResizeArgs {
        window_size: window.size().into(),
        draw_size: window.draw_size().into()
    });

    while let Some(e) = window.next() {

        game.render(&e, &mut window);

        if let Some(args) = e.press_args() {
            game.key_press(&args);
        }

        if let Some(args) = e.release_args() {
            game.key_release(&args);
        }

        if let Some(args) = e.resize_args() {
            game.resize_window(&args);
        }

        if let Some(args) = e.update_args() {
            game.update(&args);
        }

        if let Some(args) = e.text_args() {
            game.text_input(args);
        }

        if let Some(args) = e.mouse_cursor_args() {
            game.mouse_cursor_args(args);
        }

        if let Some(args) = e.mouse_scroll_args() {
            game.mouse_scroll_args(args);
        }
    }
}


fn main() {
    run_game();
}