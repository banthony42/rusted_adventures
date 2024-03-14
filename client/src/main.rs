extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::clone;

use glutin_window::GlutinWindow as Window;
use graphics::{clear, Context, DrawState, Graphics, Image, Transformed, ImageSize};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, Button};
use piston::window::WindowSettings;
use piston::{PressEvent, ReleaseEvent, ResizeArgs, ResizeEvent};

pub mod constants;
pub mod aseprite_export_tilemap;
pub mod world;

use world::{Coord, TileMapData, Tileset, World};

pub struct Game {
    gl: GlGraphics, // OpenGL drawing backend.
    map_img: Image,
    ui_img: Image,
    ui_txt: Texture,
    world: World,
    map_x_centered: f64,
    map_y_centered: f64
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
            let tileset = &map_data.tilesets[map_data.map.tileset_index];
            map_data.map.draw(tileset, &mut |src_rect, x, y| {
                    self.map_img.src_rect(*src_rect).draw(
                        &tileset.tileset,
                        &DrawState::default(),
                        ctx.transform.trans(self.map_x_centered + *x as f64 * tileset.tile_width as f64, self.map_y_centered + *y as f64 *  tileset.tile_height as f64),
                        gl);
            });

            let tileset = &map_data.tilesets[map_data.collider.tileset_index];
            map_data.collider.draw(tileset, &mut |src_rect, x, y| {
                    self.map_img.src_rect(*src_rect).draw(
                        &tileset.tileset,
                        &DrawState::default(),
                        ctx.transform.trans(self.map_x_centered + *x as f64 * tileset.tile_width as f64, self.map_y_centered + *y as f64 *  tileset.tile_height as f64),
                        gl);
            });

            let tileset = &map_data.tilesets[map_data.sprites.tileset_index];
            map_data.sprites.draw(tileset, &mut |src_rect, x, y| {
                    self.map_img.src_rect(*src_rect).draw(
                        &tileset.tileset,
                        &DrawState::default(),
                        ctx.transform.trans(self.map_x_centered + *x as f64 * tileset.tile_width as f64, self.map_y_centered + *y as f64 *  tileset.tile_height as f64),
                        gl);
            });
        });
    }

    // Not working because it need to take self reference
    // and self reference is already in use within the self.gl.draw closure because of gl pass to the closure
    // and rust doesn't differ between &self and &self.gl (both consider reference on self)
    // Therefore TileMapData.draw should be implem wich will take closure
    // src_rect and texture will be pass to the closure 
    // In addition maybe we should move tileset directly into the struct TileMapData instead of tileset_index
    // map_data.map.draw(tileset, |src_rect, texture| {
    //     self.map_img.src_rect(src_rect).draw(
    //         &tileset.tileset,
    //         &DrawState::default(),
    //         ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
    //         gl);
    // })

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

        self.map_x_centered = ((window_width - constants::MAP_WIDTH) / 2) as f64;
        self.map_y_centered = ((window_height - constants::GAME_HEIGHT) / 2) as f64;

        let gui_x_centered = ((window_width - constants::GUI_WIDTH) / 2) as f64;
        self.ui_img = Image::new().rect([gui_x_centered, self.map_y_centered + constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]);
    }

    fn update(&mut self, args: &UpdateArgs) {
    }
}



fn run_game() {
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("rpg", [constants::WINDOW_WIDTH as u32, constants::WINDOW_HEIGHT as u32])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        map_img: Image::new(),
        map_x_centered: constants::MAP_WIDTH_CENTER as f64,
        map_y_centered: constants::MAP_HEIGHT_CENTER as f64,
        ui_img: Image::new().rect([constants::GUI_WIDTH_CENTER as f64, constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]),
        ui_txt: Texture::from_path("../assets/v2/interface_1024x192_grid16.png", &TextureSettings::new()).unwrap(),
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