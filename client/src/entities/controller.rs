use std::collections::HashMap;

use piston::{Button, MouseButton, Size};

use crate::{
    entities::path_finding::PathFinder,
    import::assets::{Animations, EntityAssets, GameAsset},
    world::{MapCoord, MapData, WorldCoord},
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

    pub fn update(&mut self, delta_ts: u128) {
        self.player.update(delta_ts);
        self.view.update(delta_ts, &mut self.player);
        for entity in &mut self.entities {
            // entity.update(delta_ts);
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
            let destination = MapCoord {
                x: (self.mouse_pos[0] - self.margin.width) as i64 / 64,
                y: (self.mouse_pos[1] - self.margin.height) as i64 / 64,
            }
            .limit();

            self.path_finder.compute(
                self.player.get_map(),
                destination,
                &world_map.get(&self.player.get_world()).unwrap().colliders,
            );
            self.player.set_path(self.path_finder.get_path());
        }

        // TEMPORARY MOVEMENT
        // if let &Button::Keyboard(key) = args {
        //     match key {
        //         piston::Key::Up => self
        //             .player
        //             .set_map(self.player.get_map() + MapCoord { x: 0, y: -1 }),
        //         piston::Key::Down => self
        //             .player
        //             .set_map(self.player.get_map() + MapCoord { x: 0, y: 1 }),
        //         piston::Key::Left => self
        //             .player
        //             .set_map(self.player.get_map() + MapCoord { x: -1, y: 0 }),
        //         piston::Key::Right => self
        //             .player
        //             .set_map(self.player.get_map() + MapCoord { x: 1, y: 0 }),
        //         _ => {}
        //     }
        // }
    }
}
