use std::collections::HashMap;
use constants::*;
use graphics::{clear, math::Matrix2d, DrawState, Image, Transformed};
use piston_window::*;


use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
}

pub mod constants;
pub mod entity;
pub mod client;
pub mod world;
pub mod font;

use entity::GameTexture;
use client::GameData;
use world::{Coord, World};
use font::Font;

pub struct Game {
    map_img: Image,
    ui_img: Image,
    hard_textures: HashMap<GameTexture, G2dTexture>,
    world: World,
    map_x_centered: f64,
    map_y_centered: f64,
    gui_x_centered: f64,
    fetched_data: GameData,
    ts: u128,
    delta_ts: u128,
    font: Font
}

impl Game {
    fn render(&mut self, evnt : &Event, window: &mut PistonWindow) {

        window.draw_2d(evnt, |ctx, gl, device| {
            // Clear the screen.
            clear(constants::BLACK, gl);

            // Draw hardsaved PNG map and UI
            self.ui_img.draw(&self.hard_textures[&GameTexture::Interface] , &DrawState::default(), ctx.transform, gl);         

            // Draw map based on tiles
            let map_data = self.world.world.get_mut(&self.fetched_data.player.world_coord).unwrap();
            let _ = map_data.sprites.iter_mut().map(|sprite| {

                // When the timer for the frame reach the total duration for this frame
                // Pass to the next frame.
                if sprite.timer >= (map_data.frames[sprite.frame_index]) as u128 {
                    if sprite.frame_index >= (sprite.frames.len() -1) {
                        sprite.frame_index = 0;
                    } else {
                        sprite.frame_index += 1;
                    }
                    sprite.timer = 0;
                } else {
                    sprite.timer += self.delta_ts;
                }

                let sprite_texture = &map_data.tilesets[sprite.tileset as usize];
                let tile_number = sprite.frames[sprite.frame_index].tileset_index;

                let src_rect = [
                    (tile_number as u32 % (sprite_texture.get_width() / TILE_WIDTH) * TILE_WIDTH) as f64,
                    (tile_number as u32 / (sprite_texture.get_width() / TILE_WIDTH) * TILE_HEIGHT) as f64,
                    TILE_WIDTH as f64,
                    TILE_HEIGHT as f64,
                ];

                let x = (sprite.frames[sprite.frame_index].tilemap_index as u32 % TILEMAP_WIDTH) as f64;
                let y = (sprite.frames[sprite.frame_index].tilemap_index as u32 / TILEMAP_WIDTH) as f64;

                self.map_img.src_rect(src_rect).draw(
                    sprite_texture,
                    &DrawState::default(),
                    ctx.transform.trans(self.map_x_centered + x as f64 * TILE_WIDTH as f64, self.map_y_centered + y as f64 * TILE_HEIGHT as f64),
                    gl);
   
            }).collect::<Vec<_>>();

            // Draw players
            let trans = ctx.transform.trans(
                self.map_x_centered + self.fetched_data.player.map_coord.x as f64 * 64.0,
                self.map_y_centered + (self.fetched_data.player.map_coord.y as f64 * 64.0) - 64.0);

            let player_img = Image::new();
            player_img.draw(&self.hard_textures[&self.fetched_data.player.texture], &DrawState::default(),trans, gl);

            let map_coord_txt = format!("{}\nCoordonnées: {}, {}", map_data.info, self.fetched_data.player.world_coord.x, self.fetched_data.player.world_coord.y);
            self.render_text(map_coord_txt.as_str(), &ctx, gl, device, WHITE, Coord { x: 5, y: 17 });
            self.render_text("[14:30:01]: Salut les amis!", &ctx, gl, device, BLACK, Coord { x: 16 + 5, y: 928 - 10});

        });
    }

    pub fn render_text(&mut self, text: &str, ctx: &Context, gl: &mut G2d, device: &mut GfxDevice, color: [f32;4], pos: Coord) {
        let texts : Vec<&str> = text.split("\n").collect();

        let _: Vec<_> = texts.iter().enumerate().map(|(index, text)| {
            let _ = text::Text::new_color(color, 17).draw(
                text,
                self.font.get(),
                &ctx.draw_state,
                ctx.transform.trans(self.map_x_centered + pos.x as f64, self.map_y_centered + pos.y as f64 + (index * 17) as f64 ), gl
            );
            self.font.get().factory.encoder.flush(device);
        }).collect();
    }

    fn update(&mut self, _args: &UpdateArgs) {
        self.delta_ts = get_timestamp() - self.ts;
        self.ts = get_timestamp();
    }

    fn key_press(&mut self, args: &Button) {
        let map_data: &world::MapData = &self.world.world[&self.fetched_data.player.world_coord];
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::W | piston::Key::Up => {
                    self.fetched_data.player.move_y(-1, &map_data.sprites);
                },
                piston::Key::S | piston::Key::Down => {
                    self.fetched_data.player.move_y(1, &map_data.sprites);
                },
                piston::Key::A | piston::Key::Left => {
                    self.fetched_data.player.move_x(-1, &map_data.sprites);
                },
                piston::Key::D | piston::Key::Right => {
                    self.fetched_data.player.move_x(1, &map_data.sprites);
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

fn load_hard_drown_assets(window: &mut PistonWindow) -> HashMap<GameTexture, G2dTexture> {
    // Load whole hard drown PNG interface
    let assets: Vec<&str> = vec![
        "../assets/v2/interface_1024x192_grid16.png",
        "../assets/v3/character.png"
    ];

    let loaded_assets : HashMap<GameTexture, G2dTexture> = assets.iter().map(|path| {
        let text = match Texture::from_path(&mut window.create_texture_context(), path, Flip::None, &TextureSettings::new()) {
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

    let g_data = match client::GameData::get_data_from_server() {
        Ok(data) => data,
        Err(error) => {
            // TODO: should not exit
            println!("{error}");
            std::process::exit(1);
        }
    };

    // Create a new game and run it.
    let mut game = Game {
        map_img: Image::new(),
        map_x_centered: constants::MAP_WIDTH_CENTER as f64,
        map_y_centered: constants::MAP_HEIGHT_CENTER as f64,
        gui_x_centered: 0.0,
        ui_img: Image::new().rect([constants::GUI_WIDTH_CENTER as f64, constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]),
        hard_textures: load_hard_drown_assets(&mut window),
        world: world::World::new(&mut window),
        fetched_data: g_data,
        ts: get_timestamp(),
        delta_ts: 0,
        font: Font::new()
    };

    game.font.load(&mut window);
    game.handle_resize(window.size());

    while let Some(e) = window.next() {
        game.render(&e, &mut window);

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