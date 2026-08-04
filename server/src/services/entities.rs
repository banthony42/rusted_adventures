use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use common::character::CharacterHandler;
use common::database::db::Database;
use common::database::model::account::{Account, UpdateAccount};
use common::database::model::character::Character;
use common::database::model::monster::Monster;
use common::grpc_codegen::{
    client_entity_event::Event::PlayerMoveEvent, entity::Family, rpg_entity_server::RpgEntity,
    ClientEntityEvent, EmptyRequest, Entities, Entity as RpcEntity, LocationType, PlayerData,
    PlayerMove, ServerEntityEvent as EntityEvent,
};
use common::rpc_extentions::{RpcCoordExtension, RpcLocationExtension, ServerEntityEventExtension};
use tracing::instrument;

use crate::generics::match_for_io_error;
use crate::services::utils::login_from_metadata;
use crate::world::engine::WorldEvent;

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

async fn send_entity_event(client: &EntityEventSender, event: EntityEvent) -> bool {
    if let Err(err) = client.send(Ok(event)).await {
        tracing::error!("RpgEntityService: send_entity_event: {err}");
        return false;
    }
    true
}

async fn broadcast_player_on_map(
    sender: &String,
    event: EntityEvent,
    clients: &ArcMutexHashMapClient,
    char_handler: &mut CharacterHandler,
) {
    let players = match char_handler.players_on_same_map() {
        Ok(players) => players,
        Err(err) => {
            tracing::error!("RpgEntityService: PlayerMove: Get recipients for {sender}: {err}");
            return;
        }
    };

    tracing::info!("RpgEntityService: PlayerMove: {sender} broadcast {event:?} to: {players:?}");
    {
        let clts = clients.lock().await;

        for player in players.iter().filter(|name| sender.ne(*name)) {
            if let Some(client_channel) = clts.get(player) {
                send_entity_event(client_channel, event.clone()).await;
            }
        }
    }
}

async fn player_move(sender: String, move_event: PlayerMove, clients: ArcMutexHashMapClient) {
    let Some(new_location) = move_event.new_location else {
        tracing::warn!("RpgEntityService: PlayerMove: new_location is none");
        return;
    };

    let mut character = match CharacterHandler::new(&sender) {
        Ok(handler) => handler,
        Err(e) => {
            tracing::error!("RpgEntityService: PlayerMove: Load character for {sender} : {e}");
            return;
        }
    };

    tracing::info!("RpgEntityService: PlayerMove: {move_event:?}");

    match move_event.location_type() {
        // Player has changed cell
        LocationType::Update => {
            _ = character.update_location(new_location).inspect_err(|err| {
                tracing::error!("RpgEntityService: PlayerMove: Update location: {err}")
            })
        }
        LocationType::NewCell => {
            // Player has changed destination
            if let Some(new_destination) = new_location.into_update_destination() {
                if let Err(err) = character.update_destination(new_destination) {
                    tracing::error!("RpgEntityService: PlayerMove: Update destination: {err}");
                    return;
                }
                // broadcast all clients on the same new map with EntityMove embedding the new destination
                let move_event = EntityEvent::movement(character.identifier(), new_location);
                broadcast_player_on_map(&sender, move_event, &clients, &mut character).await;
            }
        }
        LocationType::NewMap => {
            // Player has changed map
            // Broadcast all players on last map with a despawn event
            let despawn_event = EntityEvent::despawn(character.identifier());
            broadcast_player_on_map(&sender, despawn_event, &clients, &mut character).await;

            if let Err(err) = character.update_location(new_location) {
                tracing::error!("RpgEntityService: PlayerMove: Update location: {err}");
                return;
            }

            let entity = match character.as_rpc_entity() {
                Ok(entity) => entity,
                Err(e) => {
                    tracing::error!("RpgEntityService: PlayerMove: Character as RpcEntity: {e}");
                    return;
                }
            };

            let sender_spawn = EntityEvent::spawn(entity);
            if let Ok(entities_on_map) = character.entities_on_map() {
                let clts = clients.lock().await;
                let Some(sender_tx) = clts.get(&sender) else {
                    tracing::error!("RpgEntityService: PlayerMove: Fail to get {sender} from connected clients.");
                    return;
                };

                // Parse all entities on the map (players and monsters)
                for (entity, entity_destination) in entities_on_map.iter() {
                    // Send each entity on the new map, to this player (sender)
                    let entity_spawn = EntityEvent::spawn(entity.clone());
                    if send_entity_event(sender_tx, entity_spawn).await {
                        // Also send move event for this entity if needed
                        if let Some(dest) = entity_destination {
                            let move_event =
                                EntityEvent::movement(entity.uuid.clone(), dest.into_destination());
                            send_entity_event(sender_tx, move_event).await;
                        }
                    }

                    // If entity is another player, warn him that sender has spawn here
                    match (entity.family, clts.get(&entity.name)) {
                        (Some(Family::Class(_)), Some(entity_tx)) => {
                            send_entity_event(entity_tx, sender_spawn.clone()).await;
                        }
                        (Some(Family::Class(_)), None) => tracing::error!(
                            "RpgEntityService: PlayerMove: Fail to get {:?} stream from connected clients",
                            entity.name
                        ),
                        _ => {}
                    };
                }
            }
        }
    }
}

type EntityEventSender = Sender<Result<EntityEvent, Status>>;
type ArcMutexHashMapClient = Arc<Mutex<HashMap<String, EntityEventSender>>>;

#[derive(Debug)]
pub struct RpgEntityService {
    clients: ArcMutexHashMapClient,
    event_tx: Sender<RpgEntityEvent>,
}

impl RpgEntityService {
    pub fn new(mut world_rx: Receiver<WorldEvent>) -> Self {
        let (event_tx, mut event_rx) = mpsc::channel::<RpgEntityEvent>(10);

        let clients = Arc::new(Mutex::new(HashMap::<String, EntityEventSender>::new()));

        let clts = clients.clone();
        // This task loop on the RpgEntityEvent receive channel to handle
        // RpgEntityEvent for all connected clients.
        tokio::spawn(async move {
            while let Some(receive) = event_rx.recv().await {
                match receive.event.event {
                    Some(PlayerMoveEvent(me)) => {
                        player_move(receive.sender, me, clts.clone()).await
                    }
                    None => {}
                };
            }
        });

        let clts_ref = clients.clone();
        // This task loop on the WorldEvent receive channel to transmit
        // WorldEvent on each concerned connected players.
        tokio::spawn(async move {
            let mut connection = Database::new().establish_connection();
            while let Some(receive) = world_rx.recv().await {
                match receive {
                    WorldEvent::MonsterMove(data) => {
                        let players = match Character::read_by_map(&mut connection, data.map.into())
                        {
                            Ok(players) => players,
                            Err(err) => {
                                tracing::error!("RpgEntityService: WorldEvent: MonsterMove: Get players on map: {err}");
                                return;
                            }
                        };

                        let move_event = EntityEvent::movement(data.identifier, data.destination);
                        {
                            let clts = clts_ref.lock().await;
                            // Broadcast all concerned players with the monster move
                            for player in players.iter() {
                                if let Some(client_sender) = clts.get(&player.name) {
                                    send_entity_event(client_sender, move_event.clone()).await;
                                }
                            }
                        }
                    }
                    WorldEvent::MonsterSpawn(data) => {
                        // Retrieve players located where the event occured
                        let players = match Character::read_by_map(&mut connection, data.map.into())
                        {
                            Ok(players) => players,
                            Err(err) => {
                                tracing::error!("RpgEntityService: WorldEvent: MonsterSpawn: Get players on map: {err}");
                                return;
                            }
                        };

                        // Retrieve the Monster data and create Rpc EntitySpawn event
                        let monster = match Monster::read_info(&mut connection, &data.monster_id) {
                            Ok(info) => info,
                            Err(err) => {
                                tracing::error!("RpgEntityService: WorldEvent: MonsterSpawn: Load monster: {err}");
                                return;
                            }
                        };

                        let rpc_spawn_event = EntityEvent::spawn(monster.into());
                        tracing::info!(
                            "RpgEntityService: WorldEvent: MonsterSpawn: {rpc_spawn_event:?}"
                        );
                        {
                            let clts = clts_ref.lock().await;
                            // Broadcast all concerned players with the monster spawn
                            for player in players.iter() {
                                if let Some(client_sender) = clts.get(&player.name) {
                                    send_entity_event(client_sender, rpc_spawn_event.clone()).await;
                                }
                            }
                        }
                    }
                }
            }
        });
        Self { clients, event_tx }
    }
}

#[tonic::async_trait]
impl RpgEntity for RpgEntityService {
    type EntityEventBusStream = ReceiverStream<Result<EntityEvent, Status>>;

    #[instrument(level = "debug")]
    async fn entity_event_bus(
        &self,
        request: Request<Streaming<ClientEntityEvent>>,
    ) -> Result<Response<Self::EntityEventBusStream>, Status> {
        let (metadata, _, mut client_stream) = request.into_parts();
        let login = login_from_metadata(metadata)?;
        let login_clone = login.clone();
        let (server_event_tx, server_event_rx) = mpsc::channel::<Result<EntityEvent, Status>>(10);

        // For each ClientEntityEvent request receive from the stream,
        // send it through the RpgEntityEvent channel
        // Therefore it will be process by the RpgEntityEvent receive task
        let event_tx = self.event_tx.clone();
        let cl = self.clients.clone();
        let stx = server_event_tx.clone();
        tokio::spawn(async move {
            let mut char_handler = match CharacterHandler::new(&login) {
                Ok(handler) => handler,
                Err(err) => {
                    tracing::error!("RpgEntityService entity_event_bus: {err}");
                    return;
                }
            };

            let entity = match char_handler.as_rpc_entity() {
                Ok(entity) => entity,
                Err(err) => {
                    let _ = stx.send(Err(Status::not_found(err.to_string()))).await;
                    return;
                }
            };

            let spawn_event = EntityEvent::spawn(entity);
            broadcast_player_on_map(&login, spawn_event, &cl, &mut char_handler).await;

            // Loop to handle each client entity events
            while let Some(entity_event) = client_stream.next().await {
                if let Err(status) = entity_event.as_ref() {
                    if let Some(io_err) = match_for_io_error(&status) {
                        if io_err.kind() == ErrorKind::BrokenPipe {
                            tracing::error!(
                                "RpgEntityService entity_event_bus: client {login}: broken pipe"
                            );
                            break;
                        }
                    }
                    tracing::info!("RpgEntityService entity_event_bus: client {login}: {status}");
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

            tracing::info!("RpgEntityService entity_event_bus: client {login} disconnected");
            let entity_despawn = EntityEvent::despawn(char_handler.identifier());
            broadcast_player_on_map(&login, entity_despawn, &cl, &mut char_handler).await;
            {
                cl.lock().await.remove(&login);
            }
            // Revoke the token
            let _ = Account::update(
                &mut char_handler.connection,
                &login,
                &UpdateAccount {
                    login: None,
                    password: None,
                    session_token: Some(None),
                },
            );
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

    #[instrument(level = "debug")]
    async fn get_player(
        &self,
        request: tonic::Request<EmptyRequest>,
    ) -> Result<tonic::Response<PlayerData>, tonic::Status> {
        let (metadata, _, _) = request.into_parts();
        let login = login_from_metadata(metadata)?;

        let mut char_handler = match CharacterHandler::new(&login) {
            Ok(handler) => handler,
            Err(err) => {
                tracing::error!("RpgEntityService get_player: {err}");
                return Err(tonic::Status::not_found(err.to_string()));
            }
        };

        let entity = match char_handler.as_rpc_entity() {
            Ok(entity) => entity,
            Err(err) => return Err(tonic::Status::not_found(err.to_string())),
        };

        Ok(tonic::Response::new(PlayerData {
            entity: Some(entity),
        }))
    }

    #[instrument(level = "debug")]
    async fn get_entities(
        &self,
        request: tonic::Request<EmptyRequest>,
    ) -> Result<tonic::Response<Entities>, tonic::Status> {
        let (metadata, _, _) = request.into_parts();
        let login = login_from_metadata(metadata)?;

        let mut char_handler = match CharacterHandler::new(&login) {
            Ok(handler) => handler,
            Err(err) => return Err(tonic::Status::not_found(err.to_string())),
        };

        let entities: Vec<RpcEntity> = match char_handler.entities_on_map() {
            Ok(data) => data.iter().map(|(ent, _)| ent.clone()).collect(),
            Err(err) => {
                tracing::error!("RpgEntityService get_entities: {err}");
                // TODO: investigate how to handle this in a better way
                // Which to reply to gRPC client on DB error ?
                Vec::new()
            }
        };
        Ok(tonic::Response::new(Entities { entities }))
    }
}
