use piston_window::*;

mod constants;
mod game;
mod entity;
mod world;
mod font;
mod client;
mod utils;
mod interface;

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
    game.handle_resize(window.size());

    while let Some(e) = window.next() {

        game.render(&e, &mut window);

        // Workaround: In render method i don't want to update any data
        // However to render text the Glyph/Piston API need to works with mutable
        // This is the uniq reason of this method, should be deleted for a better solution.
        game.render_mut(&e, &mut window);

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
    }
}


fn main() {
    run_game();
}