use login::Login;
use piston_window::*;
use states::GameState;
use utils::get_timestamp;

mod assets;
mod chat;
mod client;
mod constants;
mod entity;
mod game;
mod interface;
mod loading;
mod login;
mod states;
mod ui;
mod utils;
mod world;

fn run_game() {
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: PistonWindow = match WindowSettings::new(
        "rpg",
        [
            constants::WINDOW_WIDTH as u32,
            constants::WINDOW_HEIGHT as u32,
        ],
    )
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

    let mut login_state = Login::new();
    login_state.font.load(&mut window);
    let mut state: Box<dyn GameState> = Box::new(login_state);
    let mut ts: u128 = 0;

    while let Some(e) = window.next() {
        state.render(&e, &mut window);

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
            state.update(&args, get_timestamp() - ts);
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
