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
            let map_data = &self.world.world[&coord];

            // ------- Map -------
            // self.draw_tilemap(map_data.map, &map_data.tilesets, ctx, gl);

            let _ = map_data.map.tiles.iter().enumerate().map(|(index, tile_number)| {

                if *tile_number == 0 {
                    return
                }

                let x = index as u32 % map_data.map.width;
                let y = index as u32 / map_data.map.width;

                let tileset = &map_data.tilesets[map_data.map.tileset_index];

                let src_rect = [
                    (*tile_number % (tileset.tileset.get_width() / tileset.tile_width ) * tileset.tile_width) as f64 ,
                    (*tile_number / (tileset.tileset.get_width() / tileset.tile_width) * tileset.tile_height) as f64,
                    tileset.tile_width as f64,
                    tileset.tile_height as f64,
                ];

                self.map_img.src_rect(src_rect).draw(
                    &map_data.tilesets[map_data.map.tileset_index].tileset,
                    &DrawState::default(),
                    ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                    gl);
            }).collect::<Vec<_>>();

            // ------- Collider -------
            let _ = map_data.collider.tiles.iter().enumerate().map(|(index, tile_number)| {

                if *tile_number == 0 {
                    return
                }

                let x = index as u32 % map_data.collider.width;
                let y = index as u32 / map_data.collider.width;

                let tileset = &map_data.tilesets[map_data.collider.tileset_index];

                let src_rect = [
                    (*tile_number % (tileset.tileset.get_width() / tileset.tile_width ) * tileset.tile_width) as f64 ,
                    (*tile_number / (tileset.tileset.get_width() / tileset.tile_width) * tileset.tile_height) as f64,
                    tileset.tile_width as f64,
                    tileset.tile_height as f64,
                ];

                self.map_img.src_rect(src_rect).draw(
                    &map_data.tilesets[map_data.collider.tileset_index].tileset,
                    &DrawState::default(),
                    ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                    gl);
            }).collect::<Vec<_>>();

            // ------- Sprites -------
            let _ = map_data.sprites.tiles.iter().enumerate().map(|(index, tile_number)| {

                if *tile_number == 0 {
                    return
                }

                let x = index as u32 % map_data.sprites.width;
                let y = index as u32 / map_data.sprites.width;

                let tileset = &map_data.tilesets[map_data.sprites.tileset_index];

                let src_rect = [
                    (*tile_number % (tileset.tileset.get_width() / tileset.tile_width ) * tileset.tile_width) as f64 ,
                    (*tile_number / (tileset.tileset.get_width() / tileset.tile_width) * tileset.tile_height) as f64,
                    tileset.tile_width as f64,
                    tileset.tile_height as f64,
                ];

                self.map_img.src_rect(src_rect).draw(
                    &map_data.tilesets[map_data.sprites.tileset_index].tileset,
                    &DrawState::default(),
                    ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                    gl);
            }).collect::<Vec<_>>();
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
    fn draw_tilemap<G>(&mut self, tm: TileMapData, tilesets: &Vec<Tileset>, ctx: Context, gl: &mut G) where
        G : Graphics<Texture = Texture>,
        {
        let _ = tm.tiles.iter().enumerate().map(|(index, tile_number)| {
    
            if *tile_number == 0 {
                return
            }
    
            let x = index as u32 % tm.width;
            let y = index as u32 / tm.width;
            let tileset = &tilesets[tm.tileset_index];

            let src_rect = [
                (*tile_number % (tileset.tileset.get_width() / tileset.tile_width ) * tileset.tile_width) as f64 ,
                (*tile_number / (tileset.tileset.get_width() / tileset.tile_width) * tileset.tile_height) as f64,
                tileset.tile_width as f64,
                tileset.tile_height as f64,
            ];
    
            println!("==> map[{}] : {} | Map(x: {} y: {}) | TilesetSize{:?} TilesetTileSize: {}x{}| DrawRect: {:?} ", 
                        index,
                        tile_number,
                        x,
                        y,
                        tileset.tileset.get_size(),
                        tileset.tile_width,
                        tileset.tile_height,
                        src_rect);
    
            self.map_img.src_rect(src_rect).draw(
                &tileset.tileset,
                &DrawState::default(),
                ctx.transform.trans(self.map_x_centered + x as f64 * tileset.tile_width as f64, self.map_y_centered + y as f64 *  tileset.tile_height as f64),
                gl);
        }).collect::<Vec<_>>();
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