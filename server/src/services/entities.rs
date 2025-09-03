use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use common::character::CharacterAccountHandler;
use common::database::model::entity::Entity;
use common::database::model::location::{Location, UpdateLocationDestination};
use common::grpc_codegen::{
    client_entity_event::Event::PlayerMoveEvent, rpg_entity_server::RpgEntity,
    Bestiary as RpcBestiary, ClientEntityEvent, Coord as RpcCoord, EmptyRequest, Entities,
    Entity as RpcEntity, Location as RpcLocation, PlayerData, PlayerMove, ServerEntityEvent,
};
use common::grpc_codegen::{
    server_entity_event::Event::{EntityDespawnEvent, EntityMoveEvent, EntitySpawnEvent},
    EntityMove, EntitySpawn,
};
use common::grpc_codegen::{EntityDespawn, LocationType};

use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::generics::match_for_io_error;

trait ServerEntityEventExtension {
    fn new_move(uuid: String, world: RpcCoord, map: RpcCoord) -> Self;
    fn new_spawn(
        uuid: String,
        name: String,
        family: RpcBestiary,
        world: RpcCoord,
        map: RpcCoord,
    ) -> Self;
    fn new_spawn_from_entity(entity: RpcEntity) -> Self;
    fn new_despawn(uuid: String) -> Self;
}

impl ServerEntityEventExtension for ServerEntityEvent {
    fn new_move(uuid: String, world: RpcCoord, map: RpcCoord) -> Self {
        let entity = EntityMove {
            uuid,
            new_location: Some(RpcLocation {
                world: Some(world),
                map: Some(map),
            }),
        };

        ServerEntityEvent {
            event: Some(EntityMoveEvent(entity)),
        }
    }

    fn new_spawn(
        uuid: String,
        name: String,
        family: RpcBestiary,
        world: RpcCoord,
        map: RpcCoord,
    ) -> Self {
        let entity_spawn = EntitySpawnEvent(EntitySpawn {
            new_entity: Some(RpcEntity {
                uuid,
                name,
                family: family.into(),
                location: Some(RpcLocation {
                    world: Some(world),
                    map: Some(map),
                }),
            }),
        });

        ServerEntityEvent {
            event: Some(entity_spawn),
        }
    }

    fn new_spawn_from_entity(entity: RpcEntity) -> Self {
        ServerEntityEvent {
            event: Some(EntitySpawnEvent(EntitySpawn {
                new_entity: Some(entity),
            })),
        }
    }

    fn new_despawn(uuid: String) -> Self {
        ServerEntityEvent {
            event: Some(EntityDespawnEvent(EntityDespawn { uuid })),
        }
    }
}

#[derive(Debug)]
struct RpgEntityEvent {
    sender: String,
    event: ClientEntityEvent,
}

impl RpgEntityEvent {
    fn new(sender: String, event: ClientEntityEvent) -> Self {
        Self { sender, event }
    }
}

async fn player_move(sender: String, move_event: PlayerMove, clients: ArcMutexHashMapClient) {
    println!("===> PLAYER MOVE: {:?}", move_event);

    match move_event.location_type() {
        LocationType::NewMap => {
            // (new cell destination)
            // update DB with the new dest
            let mut char_handler = CharacterAccountHandler::new(&sender);
            let char_info_result = char_handler.get_character_info();
            if let Err(err) = char_info_result {
                println!("Server: player move: {:?}", err);
                return;
            }

            if move_event.new_location.is_none() {
                println!("Server: player move: LocationType::NewMap: new_location is none.");
                return;
            }

            let char_info = char_info_result.unwrap();
            let new_dest_event = move_event.new_location.unwrap();

            if let Err(err) = Location::update_destination(
                &mut char_handler.connection,
                &char_info.eid,
                UpdateLocationDestination::new(
                    new_dest_event.map.unwrap().x as f64,
                    new_dest_event.map.unwrap().y as f64,
                ),
            ) {
                println!("Server: player move: update_destination: {:?}", err);
                return;
            }

            // broadcast all clients on the same new map with EntityMove with the new destination
            let result_players = char_handler.get_all_player_on_world(char_info.world);
            if let Err(err) = result_players {
                println!("Server: player move: get_all_player_on_world: {:?}", err);
                return;
            }
            let players = result_players.unwrap();
            println!("======> MOVE EVENT: players same map: {:?}", players);
            let move_event = ServerEntityEvent::new_move(
                char_info.uuid,
                new_dest_event.world.unwrap(),
                new_dest_event.map.unwrap(),
            );
            {
                let clts = clients.lock().await;
                for (clogin, setx) in clts.iter().filter(|(login, _)| players.contains(login)) {
                    if let Err(err) = setx.send(Ok(move_event.clone())).await {
                        println!("Error: entity move event broadcast: {:?}", err);
                    } else {
                        println!(" Send spawn {:?} for {:?}", sender, clogin);
                    }
                }
            }
        }
        LocationType::NewWorld => {
            // LocationType::NewWorld (player has change map)

            // TODO: same for all LocationType
            if move_event.new_location.is_none() {
                println!("Server: player move: LocationType::Update: new_location is none.");
                return;
            }
            let new_dest_event = move_event.new_location.unwrap();

            let mut char_handler = CharacterAccountHandler::new(&sender);
            let char_info = char_handler.get_character_info().unwrap();

            // Broadcast all players on last world with a despawn event
            let despawn_event = ServerEntityEvent::new_despawn(char_info.uuid.clone());
            if let Ok(players_on_last_world) = char_handler.get_players_on_same_world() {
                let clts = clients.lock().await;
                for (clogin, setx) in clts
                    .iter()
                    .filter(|(login, _)| players_on_last_world.contains(login) && sender.ne(*login))
                {
                    if let Err(err) = setx.send(Ok(despawn_event.clone())).await {
                        println!("Error: entity despawn event broadcast: {:?}", err);
                    } else {
                        println!(" Send spawn {:?} for {:?}", sender, clogin);
                    }
                }
            }

            // Update the new world and map in DB
            char_handler.update_location(new_dest_event);

            // Broadcast all players on the world map with a spawn event
            let sender_spawn_event = ServerEntityEvent::new_spawn(
                char_info.uuid.clone(),
                sender.clone(),
                char_info.class.into(),
                new_dest_event.world.unwrap(),
                new_dest_event.map.unwrap(),
            );

            if let Ok(entities_on_new_world) = char_handler.get_entities_on_same_world() {
                let clts = clients.lock().await;
                let sender_tx = clts.get(&sender).unwrap();
                for (entity, entity_dest) in entities_on_new_world.iter() {
                    let entity_spawn_event =
                        ServerEntityEvent::new_spawn_from_entity(entity.clone());
                    if let Err(err) = sender_tx.send(Ok(entity_spawn_event)).await {
                        println!(
                            "Error: sending entities spawn event: ({:?}) for {:?} : {:?}",
                            entity.name, sender, err
                        );
                    } else {
                        println!(" Send spawn {:?} for {:?}", sender, entity.name);
                        if let Some(dest) = entity_dest {
                            let entity_move_event = ServerEntityEvent::new_move(
                                entity.uuid.clone(),
                                entity.location.unwrap().world.unwrap().into(),
                                *dest,
                            );
                            if let Err(err) = sender_tx.send(Ok(entity_move_event)).await {
                                println!(
                                "Error: sending entities move event after spawn: ({:?}) for {:?} : {:?}",
                                entity.name, sender, err
                            );
                            }
                        }
                    }

                    if entity.family() == RpcBestiary::Human {
                        let entity_tx = clts.get(&entity.name).unwrap();
                        if let Err(err) = entity_tx.send(Ok(sender_spawn_event.clone())).await {
                            println!(
                                "Error: sending sender spawn event: ({:?}) for {:?} : {:?}",
                                sender, entity.name, err
                            );
                        } else {
                            println!(" Send spawn {:?} for {:?}", sender, entity.name);
                        }
                    }
                }
            }
        }
        LocationType::Update => {
            // (player has change cell)
            // Just update the DB with the new location
            if move_event.new_location.is_none() {
                println!("Server: player move: LocationType::Update: new_location is none.");
                return;
            }
            let new_dest_event = move_event.new_location.unwrap();
            let mut char_handler = CharacterAccountHandler::new(&sender);
            char_handler.update_location(new_dest_event);
        }
    }
}

type ArcMutexHashMapClient = Arc<Mutex<HashMap<String, Sender<Result<ServerEntityEvent, Status>>>>>;

pub struct RpgEntityService {
    clients: ArcMutexHashMapClient,
    event_tx: Sender<RpgEntityEvent>,
}

impl RpgEntityService {
    pub fn new() -> Self {
        let (event_tx, mut event_rx) = mpsc::channel::<RpgEntityEvent>(10);

        let clients = Arc::new(Mutex::new(HashMap::<
            String,
            Sender<Result<ServerEntityEvent, Status>>,
        >::new()));

        let clts = clients.clone();
        // This task loop on the ChatEvent receive channel to handle
        // ChatEvent for all connected clients.
        tokio::spawn(async move {
            while let Some(receive) = event_rx.recv().await {
                match receive.event.event {
                    Some(PlayerMoveEvent(me)) => {
                        player_move(receive.sender, me, clts.clone()).await
                    }
                    None => todo!(),
                };
            }
        });

        // TODO: Temporary to test the system : (entity movements)
        // Every 10 seconds, simulate deplacement for hardcoded `entity_1uuid`
        // let cclts = clients.clone();
        // tokio::spawn(async move {
        //     loop {
        //         sleep(Duration::from_millis(10000)).await;
        //         let entity_move_event = ServerEntityEvent::new_move(
        //             "entity_1uuid".into(),
        //             RpcCoord { x: 1, y: 0 },
        //             RpcCoord {
        //                 x: rand::random_range(0..=15),
        //                 y: rand::random_range(0..=11),
        //             },
        //         );

        //         {
        //             let clts = cclts.lock().await;
        //             // Send the EntityMove event to each clients
        //             for (_, server_event_tx) in clts.iter() {
        //                 if let Err(err) = server_event_tx.send(Ok(entity_move_event.clone())).await
        //                 {
        //                     println!("Error: entity move event broadcast: {:?}", err);
        //                 }
        //             }
        //         }
        //     }
        // });

        Self { clients, event_tx }
    }
}

#[tonic::async_trait]
impl RpgEntity for RpgEntityService {
    type EntityEventBusStream = ReceiverStream<Result<ServerEntityEvent, Status>>;

    async fn entity_event_bus(
        &self,
        request: Request<Streaming<ClientEntityEvent>>,
    ) -> Result<Response<Self::EntityEventBusStream>, Status> {
        let (metadata, _, mut client_stream) = request.into_parts();
        let login = metadata.get("login").unwrap().to_str().unwrap().to_string();
        let login_clone = login.clone();
        let (server_event_tx, server_event_rx) =
            mpsc::channel::<Result<ServerEntityEvent, Status>>(10);

        // For each ClientEntityEvent request receive from the stream,
        // send it through the RpgEntityEvent channel
        // Therefore it will be process by the RpgEntityEvent receive task
        let event_tx = self.event_tx.clone();
        let cl = self.clients.clone();
        let stx = server_event_tx.clone();
        tokio::spawn(async move {
            let mut character_handler = CharacterAccountHandler::new(&login);
            let char_info_result = character_handler.get_character_info();
            if let Err(err) = char_info_result {
                let _ = stx.send(Err(Status::not_found(err.to_string()))).await;
                return;
            }
            let char_info = char_info_result.unwrap();

            // Broadcast client on same map with EntitySpawn
            let spawn_event = ServerEntityEvent::new_spawn(
                char_info.uuid.clone(),
                login.clone(),
                char_info.class.into(),
                char_info.world.into(),
                char_info.map.into(),
            );

            let result_players = character_handler.get_all_player_on_world(char_info.world);
            if let Err(err) = result_players {
                println!("Server: player move: get_all_player_on_world: {:?}", err);
                return;
            }
            let players = result_players.unwrap();

            {
                let cc = cl.lock().await;
                // Send the EntitySpawn event to each clients (on same map)
                for (clogin, setx) in cc
                    .iter()
                    .filter(|(l, _)| login.ne(*l) && players.contains(l))
                {
                    if let Err(err) = setx.send(Ok(spawn_event.clone())).await {
                        println!("Error: entity spawn event broadcast: {:?}", err);
                    } else {
                        println!(" Send spawn {:?} for {:?}", login, clogin);
                    }
                }
            }

            // Loop to handle each client entity events
            while let Some(entity_event) = client_stream.next().await {
                if let Err(status) = entity_event.as_ref() {
                    if let Some(io_err) = match_for_io_error(&status) {
                        if io_err.kind() == ErrorKind::BrokenPipe {
                            println!("RpgEntityService client: {:?} : broken pipe", login);
                            break;
                        }
                    }
                    println!("RpgEntityService: client {:?} : {:?}", login, status);
                }

                if let Some(event) = entity_event.ok() {
                    if let Err(_) = event_tx
                        .send(RpgEntityEvent::new(login.clone(), event))
                        .await
                    {
                        break;
                    }
                }
            }
            println!("RpgEntityService: client: {:?} disconnected", login);
            cl.lock().await.remove(&login);

            let entity_move_event = ServerEntityEvent::new_despawn(char_info.uuid);
            {
                let cc = cl.lock().await;
                // Send the EntityMove event to each clients
                // TODO: send only to player on same map
                for (clogin, setx) in cc.iter() {
                    if let Err(err) = setx.send(Ok(entity_move_event.clone())).await {
                        println!("Error: entity move event broadcast: {:?}", err);
                    } else {
                        println!(" Send spawn {:?} for {:?}", login, clogin);
                    }
                }
            }
        });

        self.clients
            .lock()
            .await
            .insert(login_clone, server_event_tx);

        // The ServerChatEvent rx channel is passed therefore
        // any data send through tx will be received by the gRPC codegen
        // and transmit to the client through gRPC request
        Ok(Response::new(ReceiverStream::new(server_event_rx)))
    }

    async fn get_player(
        &self,
        request: tonic::Request<EmptyRequest>,
    ) -> Result<tonic::Response<PlayerData>, tonic::Status> {
        let (metadata, _, _) = request.into_parts();
        let login = metadata.get("login").unwrap().to_str().unwrap().to_string();
        let mut character_handler = CharacterAccountHandler::new(&login);
        let char_info_result = character_handler.get_character_info();

        if let Err(err) = char_info_result {
            return Err(tonic::Status::not_found(err.to_string()));
        }

        let char_info = char_info_result.unwrap();
        let data = PlayerData {
            entity: Some(RpcEntity {
                uuid: char_info.uuid,
                name: login,
                family: RpcBestiary::Human.into(),
                location: Some(RpcLocation {
                    world: Some(char_info.world.into()),
                    map: Some(char_info.map.into()),
                }),
            }),
        };
        let response = tonic::Response::new(data);

        Ok(response)
    }

    async fn get_entities(
        &self,
        request: tonic::Request<EmptyRequest>,
    ) -> Result<tonic::Response<Entities>, tonic::Status> {
        let (metadata, _, _) = request.into_parts();
        let login = metadata.get("login").unwrap().to_str().unwrap().to_string();
        let mut character_handler = CharacterAccountHandler::new(&login);

        let result = character_handler.get_entities_on_same_world();
        let data = result.unwrap(); // TODO

        let entities: Vec<RpcEntity> = data.iter().map(|(ent, _)| ent.clone()).collect();
        let response = tonic::Response::new(Entities { entities });

        Ok(response)
    }
}
