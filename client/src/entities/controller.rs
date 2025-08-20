use std::collections::HashMap;

use piston::{Button, MouseButton, Size};

use crate::{
    constants::{MAP_CHANGE_LIMIT, MAP_EAST_LIMIT, MAP_SOUTH_LIMIT},
    entities::{
        model::{EntityModel, Orientation},
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

use common::grpc_codegen::{
    client_entity_event::Event::PlayerMoveEvent,
    server_entity_event::Event::{EntityDespawnEvent, EntityMoveEvent, EntitySpawnEvent},
};
use common::grpc_codegen::{ClientEntityEvent, ServerEntityEvent};
use std::error::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{sleep, Duration};

pub struct EntityController {
    player: Box<dyn IEntity>,
    entities: Vec<Box<dyn IEntity>>,
    path_finder: PathFinder<AStar>,
    view: EntityView,
    mouse_pos: [f64; 2],
    pub margin: Size,
    tx: Sender<ClientEntityEvent>,
    _runtime: Runtime,
}

impl EntityController {
    pub fn new(login: String, token: String, assets: HashMap<EntityAssets, GameAsset>) -> Self {
        let (controller_tx, mut controller_rx) = mpsc::channel::<ClientEntityEvent>(10);
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        runtime.spawn(async move {
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
                                                EntityMoveEvent(entity_move) => todo!(),
                                                EntitySpawnEvent(entity_spawn) => todo!(),
                                                EntityDespawnEvent(entity_despawn) => todo!(),
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
                            client_entity_event = controller_rx.recv() => {
                                if let Some(cee) = client_entity_event {
                                    if let Err(error) =  stream_tx.send(cee).await {
                                        println!("Chat stream tx error, maybe the rx has been dropped (grpc_codegen side): {:?}", error);
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

        let player = EntityClient::fetch_player();
        EntityController {
            _runtime: runtime,
            tx: controller_tx.clone(),
            path_finder: PathFinder::new(AStar::new()),
            mouse_pos: [0.0, 0.0],
            view: EntityView::new(assets),
            entities: EntityClient::fetch_entities(player.get_world()),
            player,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn set_player(&mut self, player_data: &EntityModel) {
        self.player = Box::new(player_data.clone());
    }

    pub fn set_entities(&mut self, entities: &Vec<EntityModel>) {
        self.entities = entities
            .iter()
            .map(|e| -> Box<dyn IEntity> { Box::new(e.clone()) })
            .collect();
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
