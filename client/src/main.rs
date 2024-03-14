extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use graphics::{clear, DrawState, Image, Transformed};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, Button};
use piston::window::WindowSettings;
use piston::{PressEvent, ReleaseEvent, ResizeArgs, ResizeEvent};

pub mod constants;
pub mod aseprite_export_tilemap;
pub mod world;

use world::{Coord, World};

pub struct Game {
    gl: GlGraphics, // OpenGL drawing backend.
    map_img: Image,
    ui_img: Image,
    ui_txt: Texture,
    world: World,
    map_x_centered: f64,
    map_y_centered: f64,
    gui_x_centered: f64
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {

        self.gl.draw(args.viewport(), |ctx, gl| {
            // Clear the screen.
            clear(constants::BLACK, gl);

            // Draw hardsaved PNG map and UI
            self.ui_img.draw(&self.ui_txt, &DrawState::default(), ctx.transform, gl);

            // Draw map based on tiles
            let coord = Coord { x:0, y:0 };
            let map_data: &world::MapData = &self.world.world[&coord];

            // ------- Map -------
            map_data.map.draw(&mut |src_rect, tileset, x, y| {
                    self.map_img.src_rect(*src_rect).draw(
                        &tileset.tileset,
                        &DrawState::default(),
                        ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                        gl);
            });

            map_data.collider.draw( &mut |src_rect, tileset, x, y| {
                    self.map_img.src_rect(*src_rect).draw(
                        &tileset.tileset,
                        &DrawState::default(),
                        ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                        gl);
            });

            map_data.sprites.draw( &mut |src_rect, tileset, x, y| {
                    self.map_img.src_rect(*src_rect).draw(
                        &tileset.tileset,
                        &DrawState::default(),
                        ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                        gl);
            });
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
        let window_width = args.window_size[0] as usize;
        let window_height = args.window_size[1] as usize;

        if window_width >= constants::MAP_WIDTH {
            self.map_x_centered = ((window_width - constants::MAP_WIDTH) / 2) as f64;
            self.gui_x_centered = ((window_width - constants::GUI_WIDTH) / 2) as f64;
        } else {
            self.map_x_centered = 0.0;
            self.gui_x_centered = 0.0;
        }

        if window_height >= constants::GAME_HEIGHT {
            self.map_y_centered = ((window_height - constants::GAME_HEIGHT) / 2) as f64;
        } else {
            self.map_y_centered = 0.0;
        }

        self.ui_img = Image::new().rect([self.gui_x_centered, self.map_y_centered + constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]);
    }

    fn update(&mut self, _args: &UpdateArgs) {
    }
}



fn run_game() {
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window =  match WindowSettings::new("rpg", [constants::WINDOW_WIDTH as u32, constants::WINDOW_HEIGHT as u32])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(true)
        .build() {
            Ok(window) => window,
            Err(window_error) => {
                println!("Fail to create Glutin Window: {}", window_error);
                std::process::exit(2);
            }
        };

    // Load whole hard drown PNG interface
    let interface_texture = match Texture::from_path("../assets/v2/interface_1024x192_grid16.png", &TextureSettings::new()) {
        Ok(texture) => texture,
        Err(texture_error) => {
            println!("Fail to load texture (interface PNG): {}", texture_error);
            std::process::exit(2);
        }
    };

    // Create a new game and run it.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        map_img: Image::new(),
        map_x_centered: constants::MAP_WIDTH_CENTER as f64,
        map_y_centered: constants::MAP_HEIGHT_CENTER as f64,
        gui_x_centered: 0.0,
        ui_img: Image::new().rect([constants::GUI_WIDTH_CENTER as f64, constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]),
        ui_txt: interface_texture,
        world: World::new()
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

fn main() {
    run_game();
}