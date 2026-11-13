use common::constants::{Species, SERVER_ENDPOINT};
use common::grpc_codegen::rpg_entity_client::RpgEntityClient;
use common::grpc_codegen::server_entity_event::Event::{Despawn, Move, Spawn};
use common::grpc_codegen::{
    ClientEntityEvent, EntityDespawn, EntityMove, EntitySpawn, ServerEntityEvent,
};
use common::rpc_extentions::{RpcCoordExtension, RpcLocationExtension};
use common::CellCoord;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status, Streaming};

use crate::entities::model::{EntityModel, IEntity};
use crate::states::game::GameCredentials;

type ResponseStreamingServerEntityEvent = Response<Streaming<ServerEntityEvent>>;

enum ServerError {
    EventError,
    StreamClosed,
    SenderStream,
}

pub struct EntityClient {
    _runtime: Runtime,
    credentials: GameCredentials,
    tx: Option<Sender<ClientEntityEvent>>,
}

impl EntityClient {
    pub fn new(credentials: GameCredentials) -> Self {
        let _runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Fail to invoke async context.");

        Self {
            _runtime,
            credentials,
            tx: None,
        }
    }

    pub fn send_client_event(&self, event: ClientEntityEvent) {
        if let Some(tx) = &self.tx {
            if let Err(error) = tx.try_send(event) {
                println!("EntityClient: error while transmitting event: {error:?}");
            }
        } else {
            println!("EntityClient: error while sending ClientEvent, transmitter doesn't exist.");
        }
    }

    pub fn run(&mut self, entities: Arc<Mutex<Vec<Box<dyn IEntity>>>>) {
        let credentials = self.credentials.clone();
        let (tx, mut rx) = mpsc::channel::<ClientEntityEvent>(10);
        let connection_loop = async move {
            loop {
                println!("EntityClient: EntityEventBus: try to connect ...");

                if let Ok((stream_tx, response)) = Self::connect(credentials.clone()).await {
                    println!("EntityClient: EntityEventBus connection succeed.");
                    let mut stream = response.into_inner();
                    loop {
                        select! {
                            server_event = stream.message() => {
                                if Self::handle_server_messages(server_event, &entities).await.is_err() {
                                    break;
                                }
                            },
                            client_event = rx.recv() => {
                                if  Self::handle_client_messages(&stream_tx, client_event).await.is_err() {
                                    break;
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
        };
        self._runtime.spawn(connection_loop);
        self.tx = Some(tx);
    }

    async fn entity_move_event(
        entity_move: EntityMove,
        entities: &Arc<Mutex<Vec<Box<dyn IEntity>>>>,
    ) {
        // println!("Client: EntityController: EntityMoveEvent: {:?}", entity_move);
        let mut entities = entities.lock().await;
        entities
            .iter_mut()
            .filter(|entity| entity_move.uuid.eq(entity.get_uuid()))
            // Set a new destination therefore the new path will be compute at the next update call
            .for_each(|entity| {
                if let Some(location) = entity_move.new_location {
                    if let Some(map) = location.cell {
                        entity.set_destination(CellCoord { x: map.x, y: map.y });
                    }
                }
            });
    }

    async fn entity_spawn_event(
        entity_spawn: EntitySpawn,
        entities: &Arc<Mutex<Vec<Box<dyn IEntity>>>>,
    ) {
        println!(
            "Client: EntityController: EntitySpawnEvent: {:?}",
            entity_spawn
        );
        if let Some(new_entity) = entity_spawn.new_entity {
            if let Some(family) = new_entity.family {
                match Species::try_from(family) {
                    Ok(species) => {
                        let mut instance =
                            EntityModel::new(new_entity.name, new_entity.uuid, species);
                        if let Some(location) = new_entity.location {
                            if let Some((cell_rpc_coord, map_rpc_coord)) = location.into_cell_map()
                            {
                                instance.set_cell(cell_rpc_coord.into_cell());
                                instance.set_map(map_rpc_coord.into_map());
                                instance.set_path(Vec::new(), None);
                                let mut entities = entities.lock().await;
                                entities.push(Box::new(instance));
                            }
                        }
                    }
                    Err(err) => println!(
                        "Client: EntityController: EntitySpawnEvent: Failed to parse Spefies: {:?}",
                        err
                    ),
                }
            }
        }
    }

    async fn entity_despawn_event(
        entity_despawn: EntityDespawn,
        entities: &Arc<Mutex<Vec<Box<dyn IEntity>>>>,
    ) {
        println!(
            "Client: EntityController: EntityDespawnEvent: {:?}",
            entity_despawn
        );
        let mut entities = entities.lock().await;
        entities.retain(|entity| !entity.get_uuid().eq(&entity_despawn.uuid));
    }

    async fn handle_server_messages(
        server_message: Result<Option<ServerEntityEvent>, Status>,
        entities: &Arc<Mutex<Vec<Box<dyn IEntity>>>>,
    ) -> Result<(), ServerError> {
        match server_message {
            Ok(Some(server_event)) => match server_event.event {
                Some(Move(em)) => Self::entity_move_event(em, entities).await,
                Some(Spawn(es)) => Self::entity_spawn_event(es, entities).await,
                Some(Despawn(ed)) => Self::entity_despawn_event(ed, entities).await,
                None => {}
            },
            Ok(None) => {
                println!("EntityEventBus RPC Stream closed by the server.");
                return Err(ServerError::StreamClosed);
            }
            Err(error) => {
                println!("EntityEventBus receive gRPC error from server: {:?}", error);
                return Err(ServerError::EventError);
            }
        };

        Ok(())
    }

    async fn handle_client_messages(
        tx: &Sender<ClientEntityEvent>,
        rx: Option<ClientEntityEvent>,
    ) -> Result<(), ServerError> {
        if let Some(client_entity_event) = rx {
            if let Err(error) = tx.send(client_entity_event).await {
                println!("Entity stream tx error, maybe the rx has been dropped (grpc_codegen side): {:?}", error);
                return Err(ServerError::SenderStream);
            }
        }
        Ok(())
    }

    async fn connect(
        credentials: GameCredentials,
    ) -> Result<
        (
            Sender<ClientEntityEvent>,
            ResponseStreamingServerEntityEvent,
        ),
        Box<dyn Error + Send + Sync>,
    > {
        let endpoint = Endpoint::from_static(SERVER_ENDPOINT).connect().await?;

        let mut client = RpgEntityClient::with_interceptor(
            endpoint,
            Self::auth_interceptor(credentials.login, credentials.token),
        );

        // Pass the channel rx therefore we can easily write to the stream using tx
        let (tx, rx) = mpsc::channel::<ClientEntityEvent>(10);
        let response = client.entity_event_bus(ReceiverStream::new(rx)).await?;

        Ok((tx, response))
    }

    fn auth_interceptor(
        login: String,
        token: String,
    ) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let login_md: MetadataValue<_> = login
                .parse()
                .map_err(|err| Status::invalid_argument(format!("Login: {}", err)))?;

            let token_md: MetadataValue<_> = token
                .parse()
                .map_err(|err| Status::invalid_argument(format!("Token: {}", err)))?;

            req.metadata_mut().insert("login", login_md);
            req.metadata_mut().insert("authorization", token_md);
            Ok(req)
        }
    }
}
