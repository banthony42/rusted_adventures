use std::collections::HashMap;
use graphics::{clear, DrawState, Image, Transformed};
use piston_window::*;

use crate::{
    client::GameData,
    constants,
    entity::GameTexture,
    entity::Name,
    font::Font,
    world::{
        Coord,
        MapData,
        World
    }
};

use std::time::{SystemTime, UNIX_EPOCH};


fn get_timestamp() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
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

pub struct Game {
    pub map_img: Image,
    pub ui_img: Image,
    pub hard_textures: HashMap<GameTexture, G2dTexture>,
    pub world: World,
    pub map_x_centered: f64,
    pub map_y_centered: f64,
    pub gui_x_centered: f64,
    pub fetched_data: GameData,
    pub ts: u128,
    pub delta_ts: u128,
    pub font: Font
}

impl Game {

    pub fn new(window: &mut PistonWindow) -> Self {
        let g_data = match GameData::get_data_from_server() {
            Ok(data) => data,
            Err(error) => {
                // TODO: should not exit
                println!("{error}");
                std::process::exit(1);
            }
        };

        return Game {
            map_img: Image::new(),
            map_x_centered: constants::MAP_WIDTH_CENTER as f64,
            map_y_centered: constants::MAP_HEIGHT_CENTER as f64,
            gui_x_centered: 0.0,
            ui_img: Image::new().rect([constants::GUI_WIDTH_CENTER as f64, constants::MAP_HEIGHT as f64, constants::GUI_WIDTH as f64, constants::GUI_HEIGHT as f64]),
            hard_textures: load_hard_drown_assets(window),
            world: World::new(window),
            fetched_data: g_data,
            ts: get_timestamp(),
            delta_ts: 0,
            font: Font::new()
        }
    }

    pub fn render(&mut self, evnt : &Event, window: &mut PistonWindow) {

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
                    (tile_number as u32 % (sprite_texture.get_width() / constants::TILE_WIDTH) * constants::TILE_WIDTH) as f64,
                    (tile_number as u32 / (sprite_texture.get_width() / constants::TILE_WIDTH) * constants::TILE_HEIGHT) as f64,
                    constants::TILE_WIDTH as f64,
                    constants::TILE_HEIGHT as f64,
                ];

                let x = (sprite.frames[sprite.frame_index].tilemap_index as u32 % constants::TILEMAP_WIDTH) as f64;
                let y = (sprite.frames[sprite.frame_index].tilemap_index as u32 / constants::TILEMAP_WIDTH) as f64;

                self.map_img.src_rect(src_rect).draw(
                    sprite_texture,
                    &DrawState::default(),
                    ctx.transform.trans(self.map_x_centered + x as f64 * constants::TILE_WIDTH as f64, self.map_y_centered + y as f64 * constants::TILE_HEIGHT as f64),
                    gl);
   
            }).collect::<Vec<_>>();

            // Render Map text informations
            let map_coord_txt = format!("{}\nCoordonnées: {}, {}", map_data.info, self.fetched_data.player.world_coord.x, self.fetched_data.player.world_coord.y);
            self.render_text(map_coord_txt.as_str(), &ctx, gl, device, constants::WHITE, Coord { x: 5, y: 17 });

            // Draw players
            let trans = ctx.transform.trans(
                self.map_x_centered + self.fetched_data.player.map_coord.x as f64 * 64.0,
                self.map_y_centered + (self.fetched_data.player.map_coord.y as f64 * 64.0) - 64.0);

            let player_img = Image::new();
            player_img.draw(&self.hard_textures[&self.fetched_data.player.texture], &DrawState::default(),trans, gl);
            let name_coord = Coord {
                x: (self.fetched_data.player.map_coord.x as f64 * 64.0) as i32,
                y: ((self.fetched_data.player.map_coord.y as f64 * 64.0) - 64.0) as i32
            };
            self.render_text(self.fetched_data.player.get_name().as_str(), &ctx, gl, device, constants::BLACK, name_coord);

            // Draw Entities
            for entity in self.fetched_data.entities.iter() {
                match self.hard_textures.get(&entity.texture) {
                    Some(entity_texture) => {
                        let trans = ctx.transform.trans(
                            self.map_x_centered + entity.map_coord.x as f64 * 64.0,
                            self.map_y_centered + (entity.map_coord.y as f64 * 64.0) - 64.0);
            
                        let entity_img = Image::new();
                        entity_img.draw(entity_texture, &DrawState::default(),trans, gl);
                        // let name_coord = Coord {
                        //     x: entity.map_coord.x,
                        //     y: entity.map_coord.y - 5
                        // };
                        // self.render_text(entity.get_name().as_str(), &ctx, gl, device, constants::BLACK, name_coord);
                    },
                    None => {}
                }
            }
                       
            // TMP: Chat text font test
            self.render_text("[14:30:01]: Salut les amis!", &ctx, gl, device, constants::BLACK, Coord { x: 16 + 5, y: 928 - 10});

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

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.delta_ts = get_timestamp() - self.ts;
        self.ts = get_timestamp();
    }

    pub fn key_press(&mut self, args: &Button) {
        let map_data: &MapData = &self.world.world[&self.fetched_data.player.world_coord];
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::W | piston::Key::Up => {
                    self.fetched_data.player.move_y(-1, &map_data.sprites, &self.world);
                },
                piston::Key::S | piston::Key::Down => {
                    self.fetched_data.player.move_y(1, &map_data.sprites, &self.world);
                },
                piston::Key::A | piston::Key::Left => {
                    self.fetched_data.player.move_x(-1, &map_data.sprites, &self.world);
                },
                piston::Key::D | piston::Key::Right => {
                    self.fetched_data.player.move_x(1, &map_data.sprites, &self.world);
                },
                _ => {}
           }
        }
    }

    pub fn key_release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                _ => {}
           }
        }
    }

    pub fn handle_resize(&mut self, new_size: Size) {
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

    pub fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> Resized: {window_width}x{window_height}");

        self.handle_resize(Size { width: window_width, height: window_height });
    }
}