use std::collections::HashMap;

use piston::{Button, MouseButton, Size};

use crate::{
    constants::{TILEMAP_HEIGHT, TILEMAP_WIDTH},
    entities::path_finding::PathFinder,
    import::assets::{EntityAssets, GameAsset},
    world::{Coord_tmp, MapData},
};

use super::{client::EntityClient, model::IEntity, view::EntityView};

pub struct EntityController {
    player: Box<dyn IEntity>,
    mouse_pos: [f64; 2],
    entities: Vec<Box<dyn IEntity>>,
    client: EntityClient,
    view: EntityView,
    pub margin: Size,
}

impl EntityController {
    pub fn new(login: String, token: String, assets: HashMap<EntityAssets, GameAsset>) -> Self {
        let client = EntityClient::new(login, token);
        let player = client.fetch_player();
        EntityController {
            entities: client.fetch_entities(player.get_world()),
            player,
            mouse_pos: [0.0, 0.0],
            client,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            view: EntityView::new(assets),
        }
    }

    pub fn player_world_pos(&self) -> Coord_tmp {
        self.player.get_world().clone()
    }

    pub fn render(&mut self, evnt: &piston::Event, window: &mut piston_window::PistonWindow) {
        self.view.render(evnt, window, &self.player);
        for entity in &self.entities {
            self.view.render(evnt, window, entity);
        }
    }

    pub fn update(&mut self, delta_ts: u128) {
        self.view.update(delta_ts, &mut self.player);
        for entity in &mut self.entities {
            self.view.update(delta_ts, entity);
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = margin.clone();
        self.view.resize(margin);
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.mouse_pos = args.clone();
    }

    pub fn key_press(&mut self, args: &Button, world_map: &HashMap<Coord_tmp, MapData>) {
        if let Button::Mouse(MouseButton::Left) = args {
            let x = self.mouse_pos[0] - self.margin.width;
            let y = self.mouse_pos[1] - self.margin.height;
            // TODO: maybe just compute the start point of the cell is enough
            let destination = Coord_tmp {
                x: (x as u32 / 64).min(TILEMAP_WIDTH - 1) as i32,
                y: (y as u32 / 64).min(TILEMAP_HEIGHT - 1) as i32,
            };
            let pf = PathFinder::new(
                self.player.get_map(),
                destination,
                world_map
                    .get(&self.player.get_world())
                    .unwrap()
                    .colliders
                    .clone(),
            );
            let path = pf.compute();
            self.player.set_path(path);
        }

        // TEMPORARY MOVEMENT
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Up => self
                    .player
                    .set_map(self.player.get_map() + Coord_tmp { x: 0, y: -1 }),
                piston::Key::Down => self
                    .player
                    .set_map(self.player.get_map() + Coord_tmp { x: 0, y: 1 }),
                piston::Key::Left => self
                    .player
                    .set_map(self.player.get_map() + Coord_tmp { x: -1, y: 0 }),
                piston::Key::Right => self
                    .player
                    .set_map(self.player.get_map() + Coord_tmp { x: 1, y: 0 }),
                _ => {}
            }
        }
    }
}
