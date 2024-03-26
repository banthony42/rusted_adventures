use std::collections::HashMap;
use graphics::{clear, DrawState, Image, Transformed};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::Window;
use piston_window::{
    PistonWindow,
    WindowSettings,
    Size,
    Events,
    EventSettings,
    ResizeEvent,
    ResizeArgs,
    RenderEvent,
    RenderArgs,
    UpdateEvent,
    UpdateArgs,
    PressEvent,
    ReleaseEvent,
    Button,
};

use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
}

pub mod constants;
pub mod aseprite_export_tilemap;
pub mod world;
pub mod entity;

use entity::GameTexture;
use world::{Coord, World};

struct GameData {
    player: entity::Player,
}

pub struct Game {
    gl: GlGraphics, // OpenGL drawing backend.
    map_img: Image,
    ui_img: Image,
    hard_textures: HashMap<GameTexture, Texture>,
    world: World,
    map_x_centered: f64,
    map_y_centered: f64,
    gui_x_centered: f64,
    fetched_data: GameData,
    ts: u128
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {

        self.gl.draw(args.viewport(), |ctx, gl| {
            // Clear the screen.
            clear(constants::BLACK, gl);

            // Draw hardsaved PNG map and UI
            self.ui_img.draw(&self.hard_textures[&GameTexture::Interface] , &DrawState::default(), ctx.transform, gl);

            // Draw map based on tiles
            let map_data: &world::MapData = &self.world.world[&self.fetched_data.player.world_coord];

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

            // Draw players
            let trans = ctx.transform.trans(
                self.map_x_centered + self.fetched_data.player.map_coord.x as f64 * 64.0,
                self.map_y_centered + (self.fetched_data.player.map_coord.y as f64 * 64.0) - 64.0);

            let player_img = Image::new();
            player_img.draw(&self.hard_textures[&self.fetched_data.player.texture], &DrawState::default(),trans, gl);
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        if (get_timestamp() - self.ts) > 1000 {
            self.ts = get_timestamp();
            println!("==> update: player: {:?}", self.fetched_data.player.map_coord);
        }
    }

    fn key_press(&mut self, args: &Button) {
        let map_data: &world::MapData = &self.world.world[&self.fetched_data.player.world_coord];
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::W | piston::Key::Up => {
                    self.fetched_data.player.move_player_y(-1, &map_data.collider);
                },
                piston::Key::S | piston::Key::Down => {
                    self.fetched_data.player.move_player_y(1, &map_data.collider);
                },
                piston::Key::A | piston::Key::Left => {
                    self.fetched_data.player.move_player_x(-1, &map_data.collider);
                },
                piston::Key::D | piston::Key::Right => {
                    self.fetched_data.player.move_player_x(1, &map_data.collider);
                },
                _ => {}
           }
        }
    }

    fn key_release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                _ => {}
           }
        }
    }

    fn handle_resize(&mut self, new_size: Size) {
        if new_size.width as usize >= constants::MAP_WIDTH {
            self.map_x_centered = ((new_size.width as usize - constants::MAP_WIDTH) / 2) as f64;
            self.gui_x_centered = ((new_size.width as usize - constants::GUI_WIDTH) / 2) as f64;
        } else {
            self.map_x_centered = 0.0;
            self.gui_x_centered = 0.0;
        }

        if new_size.height as usize >= constants::GAME_HEIGHT {
            self.map_y_centered = ((new_size.height as usize - constants::GAME_HEIGHT) / 2) as f64;
        } else {
            self.map_y_centered = 0.0;
        }
        self.ui_img = Image::new().rect([self.gui_x_centered, self.map_y_centered + constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]);        
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> Resized: {window_width}x{window_height}");

        self.handle_resize(Size { width: window_width, height: window_height });
    }
}

fn load_hard_drown_assets() -> HashMap<GameTexture, Texture> {
    // Load whole hard drown PNG interface
    let assets: Vec<&str> = vec![
        "../assets/v2/interface_1024x192_grid16.png",
        "../assets/v3/character.png"
    ];

    let loaded_assets : HashMap<GameTexture, Texture> = assets.iter().map(|path| {
        let text = match Texture::from_path(path, &TextureSettings::new()) {
            Ok(texture) => texture,
            Err(texture_error) => {
                println!("Fail to load hard drown texture : {} : {}", path, texture_error);
                std::process::exit(2);
            }
        };
        return match path.split("/").last().unwrap() {
            "interface_1024x192_grid16.png" => (GameTexture::Interface, text),
            "character.png" => (GameTexture::Character, text),
            _ => todo!()
        };
    }).collect();
    return loaded_assets;
}

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

    // Simulate initial server game data response
    let g_data = GameData {
        player: entity::Player::new(Coord { x:0, y:0}, Coord {x:8, y:8}),
    };

    // Create a new game and run it.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        map_img: Image::new(),
        map_x_centered: constants::MAP_WIDTH_CENTER as f64,
        map_y_centered: constants::MAP_HEIGHT_CENTER as f64,
        gui_x_centered: 0.0,
        ui_img: Image::new().rect([constants::GUI_WIDTH_CENTER as f64, constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]),
        hard_textures: load_hard_drown_assets(),
        world: World::new(),
        fetched_data: g_data,
        ts: get_timestamp()
    };

    game.handle_resize(window.size());

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