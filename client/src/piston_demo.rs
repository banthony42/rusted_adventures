extern crate piston_window;

use std::path::Path;
use piston_window::*;
use piston::input::{Button, Key, PressEvent};

const WINDOW_WIDTH:u32 = 1200;
const WINDOW_HEIGHT:u32 = 800;

fn main() {
	let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Sufod", [WINDOW_WIDTH, WINDOW_HEIGHT])
    .exit_on_esc(true)
    .graphics_api(opengl)
    .resizable(true)
    .build()
    .unwrap();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        
        if let Some(r) = e.resize_args() {
            println!("Resize ... {} x {}", r.window_size[0], r.window_size[1]);
            println!("------");
        }

        if let Some(input) = e.press_args() {
            if let Button::Keyboard(key) = input {
                match key {
                    Key::W => {
                        println!("-----------KEY-W----------------");
                    }
                    _ => {}
               }
            }
        }
    }
}