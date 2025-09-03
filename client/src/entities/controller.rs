use std::{collections::HashMap, sync::Arc};

use piston::{Button, MouseButton, Size};

use crate::{
    constants::{MAP_CHANGE_LIMIT, MAP_EAST_LIMIT, MAP_SOUTH_LIMIT},
    entities::{
        model::{Bestiary, EntityModel, Orientation},
        path_finding::PathFinder,
    },
    import::assets::{EntityAssets, GameAsset},
    world::{MapCoord, MapData, World, WorldCoord},
};

use super::{
    client::EntityClient,
    model::IEntity,
    path_finding::{astar::AStar, PathFindingStrategy},
    view::EntityView,
};

use common::grpc_codegen::ClientEntityEvent;
use common::grpc_codegen::{
    client_entity_event::Event::PlayerMoveEvent,
    server_entity_event::Event::{EntityDespawnEvent, EntityMoveEvent, EntitySpawnEvent},
    Bestiary as RpcBestiary, Coord, Location, LocationType, PlayerMove,
};
use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

enum EntityOperation {
    IDLE,
    CLEAR_ENTITIES,
}

// TODO: transform player/entities into struct EntityModel
// protected by an ArcMutex therefore both the eventbus thread and main thread could use it
pub struct EntityController {
    player: Option<Box<dyn IEntity>>,
    entities: Option<Arc<Mutex<Vec<Box<dyn IEntity>>>>>,
    operations: EntityOperation,
    path_finder: PathFinder<AStar>,
    view: EntityView,
    mouse_pos: [f64; 2],
    pub margin: Size,
    tx: Option<Sender<ClientEntityEvent>>,
    _runtime: Runtime,
}

impl EntityController {
    pub fn new(assets: HashMap<EntityAssets, GameAsset>) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        EntityController {
            _runtime: runtime,
            operations: EntityOperation::IDLE,
            tx: None,
            path_finder: PathFinder::new(AStar::new()),
            mouse_pos: [0.0, 0.0],
            view: EntityView::new(assets),
            entities: None,
            player: None,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn set_player(&mut self, player_data: &EntityModel) {
        self.player = Some(Box::new(player_data.clone()));
    }

    pub fn set_entities(&mut self, entities: &Vec<EntityModel>) {
        self.entities = Some(Arc::new(Mutex::new(
            entities
                .iter()
                .map(|e| -> Box<dyn IEntity> { Box::new(e.clone()) })
                .collect(),
        )));
    }

    pub fn init(&mut self, login: String, token: String) {
        let (controller_tx, mut controller_rx) = mpsc::channel::<ClientEntityEvent>(10);
        self.tx = Some(controller_tx);
        let some_entities = self.entities.clone().unwrap(); // shame on me

        self._runtime.spawn(async move {
            loop {
                println!("===> EntityEventBus: try to connect ...");

                if let Ok(connexion) = EntityClient::connect(login.clone(), token.clone()).await {
                    println!("===> EntityEventBus: connection succeed.");
                    let (stream_tx, response) = connexion.into_parts();
                    let mut stream = response.into_inner();
                    loop {
                        select! {
                            data = stream.message() => {
                                match data {
                                    Ok(Some(server_entity_event)) => {
                                        if let Some(se) = server_entity_event.event {
                                            match se {
                                                EntityMoveEvent(entity_move) => {
                                                    println!("====> EntityMoveEvent: {:?}", entity_move);
                                                        let mut entities = some_entities.lock().await;
                                                        let _ = entities.iter_mut()
                                                        .filter(|entity| entity_move.uuid.eq(entity.get_uuid()))
                                                        // Set a new destination therefore the new path will be compute at the next update call
                                                        .map(|entity| {
                                                            if let Some(location) = entity_move.new_location {
                                                                if let Some(map) = location.map {
                                                                    entity.set_destination(MapCoord { x: map.x, y: map.y });
                                                                }
                                                            }
                                                        })
                                                        .collect::<Vec<_>>();
                                                },
                                                EntitySpawnEvent(entity_spawn) => {
                                                    println!("====> EntitySpawnEvent: {:?}", entity_spawn);
                                                    if let Some(new_entity) = entity_spawn.new_entity {
                                                        let mut instance = match new_entity.family() {
                                                            RpcBestiary::Human => EntityModel::new(new_entity.name, new_entity.uuid, Bestiary::Human),
                                                            RpcBestiary::Bouftou => EntityModel::new(new_entity.name, new_entity.uuid, Bestiary::Bouftou),
                                                        };
                                                        let m = new_entity.location.unwrap().map.unwrap();
                                                        let w = new_entity.location.unwrap().world.unwrap();
                                                        instance.set_map(MapCoord {x: m.x, y: m.y});
                                                        instance.set_world(WorldCoord {x: w.x as i8, y: w.y as i8});
                                                        instance.set_path(Vec::new(), None);
                                                        let mut entities = some_entities.lock().await;
                                                        entities.push(Box::new(instance));
                                                    }
                                                },
                                                EntityDespawnEvent(entity_despawn) => {
                                                    println!("====> EntityDespawnEvent: {:?}",entity_despawn );
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
                    println!("===> EntityEventBus: connection failed.");
                }
                // The EntityEventBus connexion has failed, or has been shutdown
                // Wait some time before trying to reconnect
                sleep(Duration::from_millis(5000)).await;
            }
        });
    }

    pub fn player_world(&self) -> WorldCoord {
        match &self.player {
            Some(player) => player.get_world().clone(),
            None => WorldCoord::default(),
        }
    }

    pub fn render(&mut self, evnt: &piston::Event, window: &mut piston_window::PistonWindow) {
        if let Some(player) = &self.player {
            self.view.render(evnt, window, &player);
        }
        if let Some(am_entities) = &self.entities {
            if let Ok(entities) = am_entities.try_lock() {
                let _ = entities
                    .iter()
                    .map(|entity| {
                        self.view.render(evnt, window, entity);
                    })
                    .collect::<Vec<_>>();
            } else {
                dbg!("EntityController::render : fail to obtain entities model lock ...");
            }
        }
    }

    fn send_player_move_event(
        tx: &Option<Sender<ClientEntityEvent>>,
        world: WorldCoord,
        map: MapCoord,
        ltype: LocationType,
    ) {
        if let Some(tx) = tx {
            let player_move = PlayerMove {
                location_type: ltype.into(),
                new_location: Some(Location {
                    world: Some(Coord {
                        x: world.x as i64,
                        y: world.y as i64,
                    }),
                    map: Some(Coord { x: map.x, y: map.y }),
                }),
            };
            let event = ClientEntityEvent {
                event: Some(PlayerMoveEvent(player_move)),
            };
            if let Err(error) = tx.try_send(event) {
                println!("Entity controller tx error: {:?}", error);
            } else {
                // println!("Entity controller tx: send: {:?}", event);
            }
        }
    }

    pub fn update(&mut self, delta_ts: u128, world: &World) {
        if let Some(player) = &mut self.player {
            self.view.update(delta_ts, player);

            if let Some(location_type) = player.update(delta_ts, world) {
                Self::send_player_move_event(
                    &self.tx,
                    player.get_world(),
                    player.get_map(),
                    location_type,
                );

                if location_type == LocationType::NewWorld {
                    self.operations = EntityOperation::CLEAR_ENTITIES;
                }
            }

            if let Some(am_entities) = &mut self.entities {
                if let Ok(mut entities) = am_entities.try_lock() {
                    match self.operations {
                        EntityOperation::CLEAR_ENTITIES => {
                            entities.clear();
                            self.operations = EntityOperation::IDLE;
                        }
                        _ => {}
                    }

                    let _ = entities
                        .iter_mut()
                        .map(|entity| {
                            // TODO: separate more entities from player (world is needed to change map, mobs will not change map)
                            entity.update(delta_ts, world);
                            self.view.update(delta_ts, entity);

                            if let Some(map_data) = world.world.get(&player.get_world()) {
                                if let Some(new_destination) = entity.consume_destination() {
                                    self.path_finder.compute(
                                        entity.get_map(),
                                        new_destination,
                                        &map_data.colliders,
                                    );
                                    entity.set_path(self.path_finder.get_path(), None);
                                }
                            }
                        })
                        .collect::<Vec<_>>();
                } else {
                    dbg!("EntityController::update : fail to obtain entities model lock ...");
                }
            }
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
        if let Some(player) = &mut self.player {
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

                if let Some(map_data) = world_map.get(&player.get_world()) {
                    let path_found = self.path_finder.compute(
                        player.get_map(),
                        destination,
                        &map_data.colliders,
                    );
                    if path_found {
                        player.set_path(self.path_finder.get_path(), next_map_x.or(next_map_y));
                        Self::send_player_move_event(
                            &self.tx,
                            player.get_world(),
                            destination,
                            LocationType::NewMap,
                        );
                    }
                }
            }
        }
    }
}
