use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use common::grpc_codegen::{
    client_entity_event::Event::PlayerMoveEvent, rpg_entity_server::RpgEntity,
    Bestiary as RpcBestiary, ClientEntityEvent, Coord as RpcCoord, EmptyRequest, Entities,
    Entity as RpcEntity, Location, PlayerData, PlayerMove, ServerEntityEvent,
};
use common::grpc_codegen::{server_entity_event, EntityMove};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::generics::match_for_io_error;

#[derive(Debug)]
struct RpgEntityEvent {
    sender: String,
    event: ClientEntityEvent,
}

impl RpgEntityEvent {
    fn new(sender: String, event: ClientEntityEvent) -> Self {
        Self { sender, event }
    }

    /// Consumes `self` returning the parts of the event.
    fn into_parts(self) -> (String, ClientEntityEvent) {
        (self.sender, self.event)
    }
}

async fn player_move(move_event: PlayerMove, clients: ArcMutexHashMapClient) {
    println!("===> PLAYER MOVE: {:?}", move_event);
    // TODO: Update DB

    // TODO: If world map has change:
    //      broadcast all clients on the last map with EntityDespawn
    //      broadcast all clients on the new map with EntitySpawn
    // else (world map has not change:)
    //      If this move event has type: LocationType::New
    //          broadcast all clients on the same map with EntityMove
    //          (avoid to update all clients at each cell update, only when player change destination on the map)
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
                    Some(PlayerMoveEvent(me)) => player_move(me, clts.clone()).await,
                    None => todo!(),
                };
            }
        });

        // Temporary to test the system : (entity movements)
        // Every 10 seconds, simulate deplacement for hardcoded `entity_1uuid`
        let cclts = clients.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(10000)).await;
                let entity_move_event = ServerEntityEvent {
                    event: Some(server_entity_event::Event::EntityMoveEvent(EntityMove {
                        uuid: "entity_1uuid".into(),
                        new_location: Some(Location {
                            world: Some(RpcCoord { x: 1, y: 0 }),
                            map: Some(RpcCoord {
                                x: rand::random_range(0..=15),
                                y: rand::random_range(0..=11),
                            }),
                        }),
                    })),
                };

                {
                    let clts = cclts.lock().await;
                    // Send the EntityMove event to each clients
                    for (_, server_event_tx) in clts.iter() {
                        if let Err(err) = server_event_tx.send(Ok(entity_move_event.clone())).await
                        {
                            println!("Error: chat broadcast: {:?}", err);
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
    type EntityEventBusStream = ReceiverStream<Result<ServerEntityEvent, Status>>;

    async fn entity_event_bus(
        &self,
        request: Request<Streaming<ClientEntityEvent>>,
    ) -> Result<Response<Self::EntityEventBusStream>, Status> {
        let (metadata, _, mut client_stream) = request.into_parts();
        let login = metadata.get("login").unwrap().to_str().unwrap().to_string();

        let (server_event_tx, server_event_rx) =
            mpsc::channel::<Result<ServerEntityEvent, Status>>(10);

        self.clients
            .lock()
            .await
            .insert(login.clone(), server_event_tx);

        // For each ClientEntityEvent request receive from the stream,
        // send it through the RpgEntityEvent channel
        // Therefore it will be process by the RpgEntityEvent receive task
        let event_tx = self.event_tx.clone();
        let cl = self.clients.clone();
        tokio::spawn(async move {
            // Get character info from DB
            // TODO: Broadcast all clients with EntitySpawn
            // Update DB ? player is connected ?
            while let Some(chat_event) = client_stream.next().await {
                if let Err(status) = chat_event.as_ref() {
                    if let Some(io_err) = match_for_io_error(&status) {
                        if io_err.kind() == ErrorKind::BrokenPipe {
                            println!("RpgEntityService client: {:?} : broken pipe", login);
                            break;
                        }
                    }
                    println!("RpgEntityService: client {:?} : {:?}", login, status);
                }

                if let Some(event) = chat_event.ok() {
                    if let Err(_) = event_tx
                        .send(RpgEntityEvent::new(login.clone(), event))
                        .await
                    {
                        break;
                    }
                }
            }
            println!("RpgEntityService: client: {:?} disconnected", login);
            // TODO: Broadcast all clients with EntityDespawn
            // Update DB ? player is disconnected ?
            cl.lock().await.remove(&login);
        });

        // The ServerChatEvent rx channel is passed therefore
        // any data send through tx will be received by the gRPC codegen
        // and transmit to the client through gRPC request
        Ok(Response::new(ReceiverStream::new(server_event_rx)))
    }

    async fn get_player(
        &self,
        request: tonic::Request<EmptyRequest>,
    ) -> Result<tonic::Response<PlayerData>, tonic::Status> {
        // Temporary hardcoded player to test rpc communication
        // next step get data from database
        let data = PlayerData {
            entity: Some(RpcEntity {
                uuid: "playeruuid".into(),
                name: "SulfurelHardcoded".into(),
                family: RpcBestiary::Human.into(),
                location: Some(Location {
                    world: Some(RpcCoord { x: 1, y: 0 }),
                    map: Some(RpcCoord { x: 2, y: 3 }),
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
        // Temporary hardcoded entities to test rpc communication
        // next step get data from database
        let entity_1 = RpcEntity {
            uuid: "entity_1uuid".into(),
            name: "Bouftou1Hardcoded".into(),
            family: RpcBestiary::Bouftou.into(),
            location: Some(Location {
                world: Some(RpcCoord { x: 1, y: 0 }), // Consider only one character per account so retrieve this location using login/token metadata
                map: Some(RpcCoord { x: 2, y: 4 }),
            }),
        };

        let response = tonic::Response::new(Entities {
            entities: vec![entity_1],
        });

        Ok(response)
    }
}
