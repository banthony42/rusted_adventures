extern crate piston_window;

use std::path::Path;
use piston_window::*;

const WINDOW_WIDTH:u32 = 1200;
const WINDOW_HEIGHT:u32 = 800;

// Add check after loading map and gui, if size differ from const stop the program
const MAP_WIDTH:u32 = 800;
const MAP_HEIGHT:u32 = 600;

const GUI_WIDTH:u32 = 800;
const GUI_HEIGHT:u32 = 200;

fn main() {
	let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Sufod", [WINDOW_WIDTH, WINDOW_HEIGHT])
    .exit_on_esc(true)
    .graphics_api(opengl)
    .resizable(true)
    .build()
    .unwrap();

	//Create the image object and attach a square Rectangle object inside.
    let map_x_centered = ((WINDOW_WIDTH - MAP_WIDTH) / 2) as f64;
    let gui_x_centered = ((WINDOW_WIDTH - GUI_WIDTH) / 2) as f64;
	let map   = Image::new().rect([map_x_centered, 0.0, MAP_WIDTH as f64, MAP_HEIGHT as f64]);
	let gui   = Image::new().rect([gui_x_centered, MAP_HEIGHT as f64, GUI_WIDTH as f64, GUI_HEIGHT as f64]);

    //A texture to use with the image
    let ref mut texture_context = window.create_texture_context();
	let texture = Texture::from_path(texture_context, Path::new("../assets/v1/export/tileset.png"), Flip::None, &piston_window::TextureSettings::new()).unwrap();
	let gui_texture = Texture::from_path(texture_context, Path::new("../assets/v1/export/interface.png"), Flip::None, &piston_window::TextureSettings::new()).unwrap();
 
    let black = [0.0, 0.0, 0.0, 1.0];
	//Main loop
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            clear(black, g);

            if let Some(resize_args) = e.resize_args() {
                println!("===> Resize: {}, {}", resize_args.window_size[0], resize_args.window_size[1]);
            }

            // e.mouse_scroll(|d| println!("Scrolled mouse '{}, {}'", d[0], d[1]));
            // e.mouse_relative(|d| println!("Relative mouse moved '{} {}'", d[0], d[1]));
            // e.resize(|args| println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]));
            //Draw the image with the texture
            map.draw(&texture, &DrawState::default(), c.transform, g);
            gui.draw(&gui_texture, &DrawState::default(), c.transform, g);
        });
    };
}