extern crate piston_window;
extern crate gfx;

use std::path::Path;
use piston_window::*;
use piston::input::{Button, Key, PressEvent};

const WINDOW_WIDTH:u32 = 1200;
const WINDOW_HEIGHT:u32 = 800;

// Add check after loading map and gui, if size differ from const stop the program
const MAP_WIDTH:u32 = 800;
const MAP_HEIGHT:u32 = 600;

const GUI_WIDTH:u32 = 800;
const GUI_HEIGHT:u32 = 200;

const GAME_HEIGHT:u32 = MAP_HEIGHT + GUI_HEIGHT;

fn main() {

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Sufod", [WINDOW_WIDTH, WINDOW_HEIGHT])
    .exit_on_esc(true)
    .graphics_api(opengl)
    .resizable(false)
    .build()
    .unwrap();

	//Create the image object and attach a square Rectangle object inside.
    let map_x_centered = ((WINDOW_WIDTH - MAP_WIDTH) / 2) as f64;
    let gui_x_centered = ((WINDOW_WIDTH - GUI_WIDTH) / 2) as f64;
    let mut map   = Image::new().rect([map_x_centered, 0.0, MAP_WIDTH as f64, MAP_HEIGHT as f64]);
    let mut gui   = Image::new().rect([gui_x_centered, MAP_HEIGHT as f64, GUI_WIDTH as f64, GUI_HEIGHT as f64]);

    //A texture to use with the image
    let ref mut texture_context = window.create_texture_context();
    let texture = Texture::from_path(texture_context, Path::new("../assets/v1/export/tileset.png"), Flip::None, &piston_window::TextureSettings::new()).unwrap();
    let gui_texture = Texture::from_path(texture_context, Path::new("../assets/v1/export/interface.png"), Flip::None, &piston_window::TextureSettings::new()).unwrap();
 
     
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(r) = e.resize_args() {
            let window_width = r.window_size[0] as u32;
            let window_height = r.window_size[1] as u32;
            let map_x_centered = ((window_width - MAP_WIDTH) / 2) as f64;
            let gui_x_centered = ((window_width - GUI_WIDTH) / 2) as f64;

            let map_y_centered = ((window_height - GAME_HEIGHT) / 2) as f64;

            map = Image::new().rect([map_x_centered, map_y_centered, MAP_WIDTH as f64, MAP_HEIGHT as f64]);
            gui = Image::new().rect([gui_x_centered, map_y_centered + MAP_HEIGHT as f64, GUI_WIDTH as f64, GUI_HEIGHT as f64]);
            println!("Resize : {} x {}", window_width, window_height);
            println!("Resize : new map_x: {}", map_x_centered);
        }

        if let Some(input) = e.press_args() {
            if let Button::Keyboard(key) = input {
                match key {
                    any => {
                        println!("Key pressed: {:?}", any);
                    }
               }
            }
        }

        if let Some(r) = e.render_args() {

            let black = [0.0, 0.0, 0.0, 1.0];
            if let Some(e) = window.next() {
                window.draw_2d(&e, |c, g, _| {
                    clear(black, g);
    
                    //Draw the image with the texture
                    map.draw(&texture, &DrawState::default(), c.transform, g);
                    gui.draw(&gui_texture, &DrawState::default(), c.transform, g);
                });
            }
        }
    }
}