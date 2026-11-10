use std::{collections::HashMap, sync::Arc};

use piston::{Button, MouseButton, Size};
use piston_window::PistonWindow;

use crate::{
    entities::{
        model::{EntityModel, Player, PlayerType, UIEntityModel, UpdateInformation},
        path_finding::PathFinder,
    },
    import::assets::{EntityAssets, GameAsset},
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
        server_entity_event::Event::{Despawn, Move, Spawn},
        PlayerNewCell, PlayerNewDest, PlayerNewMap,
    },
    rpc_extentions::{RpcCoordExtension, RpcLocationExtension},
    CellCoord, Orientation,
};
use common::{
    grpc_codegen::{ClientEntityEvent, Coord, Location},
    MapCoord,
};
use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

enum EntityOperation {
    Idle,
    ClearEntities,
}

pub struct EntityController {
    player: PlayerType,
    entities: Arc<Mutex<Vec<Box<dyn IEntity>>>>,
    operations: EntityOperation,
    path_finder: PathFinder<AStar>,
    view: EntityView,
    mouse_pos: [f64; 2],
    pub margin: Size,
    tx: Option<Sender<ClientEntityEvent>>,
    _runtime: Runtime,
}

impl EntityController {
    pub fn new(
        assets: HashMap<EntityAssets, GameAsset>,
        player: PlayerType,
        entities: Vec<EntityModel>,
    ) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        let am_entities = Arc::new(Mutex::new(
            entities
                .iter()
                .map(|e| -> Box<dyn IEntity> { Box::new(e.clone()) })
                .collect(),
        ));

        EntityController {
            _runtime: runtime,
            operations: EntityOperation::Idle,
            tx: None,
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

    pub fn init(&mut self, login: String, token: String) {
        let (controller_tx, mut controller_rx) = mpsc::channel::<ClientEntityEvent>(10);
        self.tx = Some(controller_tx);
        let some_entities = self.entities.clone();

        self._runtime.spawn(async move {
            loop {
                println!("Client: EntityController: EntityEventBus: try to connect ...");

                if let Ok(connexion) = EntityClient::connect(login.clone(), token.clone()).await {
                    println!("Client: EntityController: EntityEventBus connection succeed.");
                    let (stream_tx, response) = connexion.into_parts();
                    let mut stream = response.into_inner();
                    loop {
                        select! {
                            data = stream.message() => {
                                match data {
                                    Ok(Some(server_entity_event)) => {
                                        if let Some(se) = server_entity_event.event {
                                            match se {
                                                Move(entity_move) => {
                                                    // println!("Client: EntityController: EntityMoveEvent: {:?}", entity_move);
                                                        let mut entities = some_entities.lock().await;
                                                        entities.iter_mut()
                                                        .filter(|entity| entity_move.uuid.eq(entity.get_uuid()))
                                                        // Set a new destination therefore the new path will be compute at the next update call
                                                        .for_each(|entity| {
                                                            if let Some(location) = entity_move.new_location {
                                                                if let Some(map) = location.cell {
                                                                    entity.set_destination(CellCoord { x: map.x, y: map.y });
                                                                }
                                                            }
                                                        });
                                                },
                                                Spawn(entity_spawn) => {
                                                    println!("Client: EntityController: EntitySpawnEvent: {:?}", entity_spawn);
                                                    if let Some(new_entity) = entity_spawn.new_entity {
                                                        if let Some(family) = new_entity.family {
                                                            match Species::try_from(family) {
                                                                Ok(species) => {
                                                                    let mut instance = EntityModel::new(new_entity.name, new_entity.uuid, species);
                                                                    if let Some(location)= new_entity.location {
                                                                        if let Some((cell_rpc_coord, map_rpc_coord)) = location.into_cell_map() {
                                                                            instance.set_cell(cell_rpc_coord.into_cell());
                                                                            instance.set_map(map_rpc_coord.into_map());
                                                                            instance.set_path(Vec::new(), None);
                                                                            let mut entities = some_entities.lock().await;
                                                                            entities.push(Box::new(instance));
                                                                        }
                                                                    }
                                                                },
                                                                Err(err) => println!("Client: EntityController: EntitySpawnEvent: Failed to parse Spefies: {:?}", err),
                                                            }
                                                        }
                                                    }
                                                },
                                                Despawn(entity_despawn) => {
                                                    println!("Client: EntityController: EntityDespawnEvent: {:?}",entity_despawn );
                                                    let mut entities = some_entities.lock().await;
                                                    entities.retain(|entity| !entity.get_uuid().eq(&entity_despawn.uuid));
                                                },
                                            }
                                        }
                                    },
                                    Ok(None) => {
                                        println!("EntityEventBus RPC Stream closed by the server.");
                                        break
                                    },
                                    Err(error) => println!("EntityEventBus receive gRPC error from server: {:?}", error)
                                }
                            },
                            // Handle ClientEntityEvent, and send them to the stream
                            some_client_entity_event = controller_rx.recv() => {
                                if let Some(client_entity_event) = some_client_entity_event {
                                    if let Err(error) =  stream_tx.send(client_entity_event).await {
                                        println!("Entity stream tx error, maybe the rx has been dropped (grpc_codegen side): {:?}", error);
                                        break
                                    }
                                }
                            }
                        }
                    }
                    println!("Client: EntityController: EntityEventBus: connection failed.");
                }
                // The EntityEventBus connexion has failed, or has been shutdown
                // Wait some time before trying to reconnect
                sleep(Duration::from_millis(5000)).await;
            }
        });
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

    fn send_player_move_event(
        tx: &Option<Sender<ClientEntityEvent>>,
        map: MapCoord,
        cell: CellCoord,
        update_info: UpdateInformation,
    ) {
        if let Some(tx) = tx {
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
                UpdateInformation::NewMap => PNewMap(PlayerNewMap {
                    map: Some(location),
                }),
                UpdateInformation::NewCell => PNewCell(PlayerNewCell {
                    cell: Some(location),
                }),
                UpdateInformation::NewDest => PNewDest(PlayerNewDest {
                    destination: Some(location),
                }),
            };

            if let Err(error) = tx.try_send(ClientEntityEvent { event: Some(event) }) {
                println!("Entity controller tx error: {:?}", error);
            }
        }
    }

    pub fn update(&mut self, delta_ts: u128, world: &World) {
        self.view.update(delta_ts, self.player.entity_mut());

        if let Some(update_info) = self.player.entity_mut().update(delta_ts, world) {
            if update_info == UpdateInformation::NewMap {
                self.operations = EntityOperation::ClearEntities;
            }
            // Send a LocationType::NewMap to the server
            // Server will reply with all new entities present on new map (EntitySpawnEvent)
            Self::send_player_move_event(
                &self.tx,
                self.player.entity().get_map(),
                self.player.entity().get_cell(),
                update_info,
            );
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
                    Self::send_player_move_event(
                        &self.tx,
                        self.player.entity().get_map(),
                        destination,
                        UpdateInformation::NewDest,
                    );
                }
            }
        }
    }
}
