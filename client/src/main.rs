extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, Button};
use piston::window::WindowSettings;
use piston::{PressEvent, ReleaseEvent, ResizeArgs, ResizeEvent};

use graphics::*;

const WINDOW_WIDTH:u32 = 1024;
const WINDOW_HEIGHT:u32 = 768+192;

// Add check after loading map and gui, if size differ from const stop the program
const MAP_WIDTH:u32 = 1024;
const MAP_HEIGHT:u32 = 768;
const MAP_WIDTH_CENTER:u32 = (WINDOW_WIDTH - MAP_WIDTH) / 2;

const GUI_WIDTH:u32 = 1024;
const GUI_HEIGHT:u32 = 192;
const GUI_WIDTH_CENTER:u32 = (WINDOW_WIDTH - GUI_WIDTH) / 2;
const GAME_HEIGHT:u32 = MAP_HEIGHT + GUI_HEIGHT;

// Colors
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub struct Game {
    gl: GlGraphics, // OpenGL drawing backend.
    map_img: Image,
    ui_img: Image,
    map_txt: Texture,
    ui_txt: Texture
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            self.map_img.draw(&self.map_txt, &DrawState::default(), c.transform, gl);
            self.ui_img.draw(&self.ui_txt, &DrawState::default(), c.transform, gl);
        });
    }

    fn key_press(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                any => {
                    println!("Key pressed: {:?}", any);
                }
           }
        }
    }

    fn key_release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                any => {
                    println!("Key released: {:?}", any);
                }
           }
        }
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0] as u32;
        let window_height = args.window_size[1] as u32;
        let map_x_centered = ((window_width - MAP_WIDTH) / 2) as f64;
        let gui_x_centered = ((window_width - GUI_WIDTH) / 2) as f64;

        let map_y_centered = ((window_height - GAME_HEIGHT) / 2) as f64;

        self.map_img = Image::new().rect([map_x_centered, map_y_centered, MAP_WIDTH as f64, MAP_HEIGHT as f64]);
        self.ui_img = Image::new().rect([gui_x_centered, map_y_centered + MAP_HEIGHT as f64, GUI_WIDTH as f64, GUI_HEIGHT as f64]);
        println!("Resize : {} x {}", window_width, window_height);
        println!("Resize : new map_x: {}", map_x_centered);
    }

    fn update(&mut self, args: &UpdateArgs) {
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("rpg", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        map_img: Image::new().rect([MAP_WIDTH_CENTER as f64, 0.0, MAP_WIDTH as f64, MAP_HEIGHT as f64]),
        ui_img: Image::new().rect([GUI_WIDTH_CENTER as f64, MAP_HEIGHT as f64, GUI_WIDTH as f64, GUI_HEIGHT as f64]),
        map_txt: Texture::from_path("../assets/v2/map_1024x768_grid64.png", &TextureSettings::new()).unwrap(),
        ui_txt: Texture::from_path("../assets/v2/interface_1024x192_grid16.png", &TextureSettings::new()).unwrap()
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

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