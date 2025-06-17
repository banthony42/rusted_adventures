use std::collections::HashMap;

use piston::{Button, MouseButton, Size};

use crate::{
    constants::{MAP_CHANGE_LIMIT, MAP_EAST_LIMIT, MAP_HEIGHT, MAP_SOUTH_LIMIT, MAP_WIDTH},
    entities::{model::Orientation, path_finding::PathFinder},
    import::assets::{EntityAssets, GameAsset},
    world::{MapCoord, MapData, World, WorldCoord},
};

use super::{
    client::EntityClient,
    model::IEntity,
    path_finding::{astar::AStar, PathFindingStrategy},
    view::EntityView,
};

pub struct EntityController {
    player: Box<dyn IEntity>,
    mouse_pos: [f64; 2],
    entities: Vec<Box<dyn IEntity>>,
    path_finder: PathFinder<AStar>,
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
            path_finder: PathFinder::new(AStar::new()),
            player,
            client,
            mouse_pos: [0.0, 0.0],
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
            view: EntityView::new(assets),
        }
    }

    pub fn player_world(&self) -> WorldCoord {
        self.player.get_world().clone()
    }

    pub fn render(&mut self, evnt: &piston::Event, window: &mut piston_window::PistonWindow) {
        self.view.render(evnt, window, &self.player);
        for entity in &self.entities {
            self.view.render(evnt, window, entity);
        }
    }

    pub fn update(&mut self, delta_ts: u128, world: &World) {
        self.player.update(delta_ts, world);
        self.view.update(delta_ts, &mut self.player);
        for entity in &mut self.entities {
            entity.update(delta_ts, world); // TODO: separate more entities from player (world is needed to change map, mobs will not change map)
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

    pub fn key_press(&mut self, args: &Button, world_map: &HashMap<WorldCoord, MapData>) {
        if let Button::Mouse(MouseButton::Left) = args {
            let mouse_x = (self.mouse_pos[0] - self.margin.width) as i64;
            let mouse_y = (self.mouse_pos[1] - self.margin.height) as i64;

            let destination = MapCoord {
                x: mouse_x / 64,
                y: mouse_y / 64,
            }
            .limit();

            let next_map_x: Option<Orientation> = match mouse_x {
                x if x > MAP_EAST_LIMIT as i64 => Some(Orientation::Est),
                x if x < MAP_CHANGE_LIMIT as i64 => Some(Orientation::West),
                _ => None,
            };
            let next_map_y: Option<Orientation> = match mouse_y {
                y if y > MAP_SOUTH_LIMIT as i64 => Some(Orientation::South),
                y if y < MAP_CHANGE_LIMIT as i64 => Some(Orientation::North),
                _ => None,
            };

            if let Some(map_data) = world_map.get(&self.player.get_world()) {
                self.path_finder
                    .compute(self.player.get_map(), destination, &map_data.colliders);
                self.player
                    .set_path(self.path_finder.get_path(), next_map_x.or(next_map_y));
            }
        }

        if let Button::Mouse(MouseButton::Right) = args {
            println!("Simulate entities serveur new position received.");

            let mouse_x = (self.mouse_pos[0] - self.margin.width) as i64;
            let mouse_y = (self.mouse_pos[1] - self.margin.height) as i64;

            let destination = MapCoord {
                x: mouse_x / 64,
                y: mouse_y / 64,
            }
            .limit();

            if let Some(map_data) = world_map.get(&self.player.get_world()) {
                for entity in &mut self.entities {
                    self.path_finder
                        .compute(entity.get_map(), destination, &map_data.colliders);
                    entity.set_path(self.path_finder.get_path(), None);
                }
            }
        }
    }
}
