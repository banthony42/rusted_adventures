use std::{collections::HashMap, sync::Arc};

use piston::{Button, MouseButton, Size};
use piston_window::PistonWindow;

use crate::{
    entities::{
        model::{EntityModel, Playable, Player, UIEntityModel, UpdateInformation},
        path_finding::PathFinder,
    },
    import::assets::{EntityAssets, GameAsset},
    states::game::GameCredentials,
    ui::font::Font,
    world::{MapData, World},
};

use super::{
    client::EntityClient,
    model::IEntity,
    path_finding::{astar::AStar, PathFindingStrategy},
    view::EntityView,
};

use common::{
    constants::*,
    grpc_codegen::{
        client_entity_event::Event::{PNewCell, PNewDest, PNewMap},
        PlayerNewCell, PlayerNewDest, PlayerNewMap,
    },
    CellCoord, Orientation,
};
use common::{
    grpc_codegen::{ClientEntityEvent, Coord, Location},
    MapCoord,
};

use tokio::sync::Mutex;

enum EntityOperation {
    Idle,
    ClearEntities,
}

pub struct EntityController {
    player: Player,
    entities: Arc<Mutex<Vec<Box<dyn IEntity>>>>,
    operations: EntityOperation,
    path_finder: PathFinder<AStar>,
    view: EntityView,
    mouse_pos: [f64; 2],
    pub margin: Size,
    client: EntityClient,
}

impl EntityController {
    pub fn new(
        assets: HashMap<EntityAssets, GameAsset>,
        credentials: GameCredentials,
        player: Player,
        entities: Vec<EntityModel>,
    ) -> Self {
        let am_entities = Arc::new(Mutex::new(
            entities
                .iter()
                .map(|e| -> Box<dyn IEntity> { Box::new(e.clone()) })
                .collect(),
        ));

        let mut client = EntityClient::new(credentials);
        client.run(am_entities.clone());

        EntityController {
            client,
            operations: EntityOperation::Idle,
            path_finder: PathFinder::new(AStar::new()),
            mouse_pos: [0.0, 0.0],
            view: EntityView::new(assets),
            entities: am_entities,
            player,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn player_map(&self) -> MapCoord {
        self.player.entity().get_map()
    }

    /// Foreach entities handled in this controller, gather data for interface rendering.
    ///
    /// This function is best effort, if a data can't be retrieve for an entity
    /// then it will be no entry in the HashMap for this entity.
    pub fn ui_entities_models(&self) -> HashMap<String, UIEntityModel> {
        let mut ui_models = HashMap::from([(
            self.player.entity().get_name().clone(),
            self.player.entity().get_ui_model(),
        )]);

        if let Ok(entities) = self.entities.try_lock() {
            for entity in entities.iter() {
                ui_models.insert(entity.get_name().clone(), entity.get_ui_model());
            }
        }
        ui_models
    }

    pub fn render(&mut self, evnt: &piston::Event, window: &mut PistonWindow, font: &mut Font) {
        self.view.render(evnt, window, self.player.entity(), font);

        if let Ok(entities) = self.entities.try_lock() {
            entities.iter().for_each(|entity| {
                self.view.render(evnt, window, entity.as_ref(), font);
            });
        } else {
            dbg!("EntityController::render : fail to obtain entities model lock ...");
        }
    }

    fn create_player_move_event(
        map: MapCoord,
        cell: CellCoord,
        update_info: UpdateInformation,
    ) -> ClientEntityEvent {
        let location = Location {
            map: Some(Coord {
                x: map.x as i64,
                y: map.y as i64,
            }),
            cell: Some(Coord {
                x: cell.x,
                y: cell.y,
            }),
        };

        let event = match update_info {
            UpdateInformation::Map => PNewMap(PlayerNewMap {
                map: Some(location),
            }),
            UpdateInformation::Cell => PNewCell(PlayerNewCell {
                cell: Some(location),
            }),
            UpdateInformation::Dest => PNewDest(PlayerNewDest {
                destination: Some(location),
            }),
        };

        ClientEntityEvent { event: Some(event) }
    }

    pub fn update(&mut self, delta_ts: u128, world: &World) {
        self.view.update(delta_ts, self.player.entity_mut());

        if let Some(update_info) = self.player.entity_mut().update(delta_ts, world) {
            if update_info == UpdateInformation::Map {
                self.operations = EntityOperation::ClearEntities;
            }
            // Send a LocationType::NewMap to the server
            // Server will reply with all new entities present on new map (EntitySpawnEvent)
            let evt = Self::create_player_move_event(
                self.player.entity().get_map(),
                self.player.entity().get_cell(),
                update_info,
            );
            self.client.send_client_event(evt);
        }

        if let Ok(mut entities) = self.entities.try_lock() {
            match self.operations {
                EntityOperation::ClearEntities => entities.clear(),
                EntityOperation::Idle => {}
            }
            self.operations = EntityOperation::Idle;

            entities.iter_mut().for_each(|entity| {
                entity.update(delta_ts, world);
                self.view.update(delta_ts, entity.as_mut());

                if let Some(map_data) = world.world.get(&self.player.entity().get_map()) {
                    if let Some(new_destination) = entity.consume_destination() {
                        self.path_finder.compute(
                            entity.get_cell(),
                            new_destination,
                            &map_data.collider_map,
                        );
                        entity.set_path(self.path_finder.get_path(), None);
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
                piston::Key::D1 => self.player.spell_1(),
                piston::Key::D2 => self.player.spell_2(),
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

            if let Some(map_data) = world.get(&self.player.entity().get_map()) {
                let path_found = self.path_finder.compute(
                    self.player.entity().get_cell(),
                    destination,
                    &map_data.collider_map,
                );
                if path_found {
                    self.player
                        .entity_mut()
                        .set_path(self.path_finder.get_path(), next_map_x.or(next_map_y));

                    let evt = Self::create_player_move_event(
                        self.player.entity().get_map(),
                        destination,
                        UpdateInformation::Dest,
                    );
                    self.client.send_client_event(evt);
                }
            }
        }
    }
}
