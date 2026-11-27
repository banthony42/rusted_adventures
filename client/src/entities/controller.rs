use std::{collections::HashMap, sync::Arc};

use piston::{Button, MouseButton, Size};
use piston_window::PistonWindow;

use crate::{
    entities::model::{ClientEntity, UIEntityModel},
    states::{game::GameCredentials, GameEntities},
    ui::font::Font,
    world::{MapData, World},
};

use super::{client::EntityClient, view::EntityView};

use common::{
    constants::*,
    entities::{player::PlayerType, traits::GameEntity},
    path_finding::{astar::AStar, PathFinder, PathFindingStrategy},
    CellCoord, MapCoord,
};

use tokio::sync::Mutex;

// TODO: Ensure all channels used are handling correctly Disconnected Error

// TODO: remove this entity.client should not have access
// Server input should be transmit to controller here through channels
pub type SharedEntities = Arc<Mutex<GameEntities>>;

pub struct EntityController {
    player: ClientEntity,
    entities: SharedEntities,
    path_finder: PathFinder<AStar>,
    view: EntityView,
    mouse_pos: [f64; 2],
    pub margin: Size,
    client: EntityClient,
}

impl EntityController {
    pub fn new(
        credentials: GameCredentials,
        player: ClientEntity,
        entities: GameEntities,
        window: &mut PistonWindow,
    ) -> Self {
        let am_entities = Arc::new(Mutex::new(entities));

        let mut client = EntityClient::new(credentials);
        client.run(am_entities.clone());

        EntityController {
            client,
            path_finder: PathFinder::new(AStar::new()),
            mouse_pos: [0.0, 0.0],
            view: EntityView::new(window),
            entities: am_entities,
            player,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn font(&mut self) -> Vec<&mut Font> {
        self.view.font()
    }

    pub fn player_map(&self) -> MapCoord {
        self.player.model().position.map
    }

    /// Foreach entities handled in this controller, gather data for interface rendering.
    ///
    /// This function is best effort, if a data can't be retrieve for an entity
    /// then it will be no entry in the HashMap for this entity.
    pub fn entities_ui_models(&self) -> HashMap<String, UIEntityModel> {
        let mut ui_models =
            HashMap::from([(self.player.model().name().clone(), self.player.ui_model())]);

        if let Ok(entities) = self.entities.try_lock() {
            for entity in entities.iter() {
                ui_models.insert(entity.model().name().clone(), entity.ui_model());
            }
        }
        ui_models
    }

    pub fn render(&mut self, evnt: &piston::Event, window: &mut PistonWindow) {
        self.view.render(evnt, window, &self.player);

        if let Ok(entities) = self.entities.try_lock() {
            entities.iter().for_each(|entity| {
                self.view.render(evnt, window, entity);
            });
        } else {
            dbg!("EntityController::render : fail to obtain entities model lock ...");
        }
    }

    pub fn update(&mut self, delta_ts: u128, world: &World) {
        self.player.update(delta_ts);
        self.view.update(delta_ts, &mut self.player);

        if let Ok(mut entities) = self.entities.try_lock() {
            entities.iter_mut().for_each(|entity| {
                entity.update(delta_ts);
                self.view.update(delta_ts, entity);

                if let Some(map_data) = world.world.get(&self.player.model().position.map) {
                    if let Some(new_destination) = entity.model_mut().destination.take() {
                        if self.path_finder.compute(
                            entity.model().position.cell,
                            new_destination.cell,
                            &map_data.collider_map,
                        ) {
                            entity.model_mut().path = Some(self.path_finder.get_path());
                        }
                    }
                }
            });
        } else {
            dbg!("EntityController::update : fail to obtain entities model lock ...");
        }
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = *margin;
        self.view.resize(margin);
    }

    pub fn mouse_cursor_args(&mut self, args: &[f64; 2]) {
        self.mouse_pos = *args;
    }

    pub fn key_press(&mut self, args: &Button, world: &HashMap<MapCoord, MapData>) {
        if let &Button::Keyboard(key) = args {
            // Call simple gRPC to request server on spell launch
            // succeess -> feedback or simulate spell
            // failure -> feedback
            match key {
                piston::Key::D1 => match self.player.class() {
                    Ok(PlayerType::Mage(m)) => m.classes.fireball(),
                    Ok(PlayerType::Warrior(w)) => w.classes.charge(),
                    Err(err) => println!("{err}"),
                },
                piston::Key::D2 => match self.player.class() {
                    Ok(PlayerType::Mage(m)) => m.classes.ice_trap(),
                    Ok(PlayerType::Warrior(w)) => w.classes.absorption(),
                    Err(err) => println!("{err}"),
                },
                _ => {}
            }
        }

        if let Button::Mouse(MouseButton::Left) = args {
            let mouse_x = (self.mouse_pos[0] - self.margin.width) as i64;
            let mouse_y = (self.mouse_pos[1] - self.margin.height) as i64;

            if !MAP_WIDTH_RANGE.contains(&mouse_x) || !MAP_HEIGHT_RANGE.contains(&mouse_y) {
                return;
            }

            let destination = CellCoord {
                x: mouse_x / 64,
                y: mouse_y / 64,
            }
            .limit();
            let position = self.player.model().position.clone();
            if let Some(map_data) = world.get(&position.map) {
                if self
                    .path_finder
                    .compute(position.cell, destination, &map_data.collider_map)
                {
                    // Request server with new destination
                    self.client.request_path(position, destination);
                    // Meanwhile, set computed path to start the move animation (client prediction)
                    self.player.model_mut().path = Some(self.path_finder.get_path());
                }
            }
        }
    }
}
