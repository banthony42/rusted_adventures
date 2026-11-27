use common::constants::SERVER_ENDPOINT;
use common::entities::traits::GameEntity;
use common::grpc_codegen::rpg_entity_client::RpgEntityClient;
use common::grpc_codegen::server_entity_event::Event::{AcceptedPath, Despawn, Move, Spawn};
use common::grpc_codegen::{
    ClientEntityEvent, EntityDespawn, EntityMove, EntitySpawn, ServerEntityEvent,
};

use common::network::conversion::IntoClientEntityEvent;
use common::network::events::{EntityEvent, EntityEventSequences, EventKind};
use common::{CellCoord, Position};
use std::error::Error;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status, Streaming};

use crate::entities::controller::SharedEntities;
use crate::entities::model::ClientEntity;
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
    tx: Option<Sender<EntityEvent>>,
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

    /// Spawn a task that connect to the server and handle communication.
    ///
    /// Then the client can send a message using `EntityClient::send_event`
    ///
    /// For now, messages receive from the server is handle fully in `EntityClient`
    /// updating `entities: Arc<Mutex<Vec<Box<dyn IEntity>>>>` passed in parameter.
    pub fn run(&mut self, entities: SharedEntities) {
        let credentials = self.credentials.clone();
        let (tx, mut rx) = mpsc::channel::<EntityEvent>(10);
        let connection_loop = async move {
            loop {
                println!("EntityClient: EntityEventBus: try to connect ...");
                if let Ok((stream_tx, response)) = Self::connect(credentials.clone()).await {
                    println!("EntityClient: EntityEventBus connection succeed.");
                    let mut stream = response.into_inner();
                    let mut sequences = EntityEventSequences::default();
                    loop {
                        select! {
                            server_event = stream.message() => {
                                if Self::handle_server_messages(&sequences, server_event, &entities).await.is_err() {
                                    break;
                                }
                            },
                            client_event = rx.recv() => {
                                let Some(event) = client_event else {
                                    break;
                                };
                                if  Self::handle_client_messages(&mut sequences, &stream_tx, event).await.is_err() {
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

    pub fn request_path(&mut self, location: Position, destination: CellCoord) {
        self.send_event(EntityEvent::RequestPath(location, destination));
    }

    fn send_event(&self, event: EntityEvent) {
        if let Some(tx) = &self.tx {
            if let Err(error) = tx.try_send(event) {
                println!("EntityClient: error while transmitting event: {error:?}");
            }
        } else {
            println!("EntityClient: error while sending ClientEvent, transmitter doesn't exist.");
        }
    }

    async fn entity_move_event(entity_move: EntityMove, entities: &SharedEntities) {
        println!(
            "Client: EntityController: EntityMoveEvent: {:?}",
            entity_move
        );
        let mut entities = entities.lock().await;
        entities
            .iter_mut()
            .filter(|entity| entity_move.uuid.eq(entity.model().uuid()))
            // Set a new destination therefore the new path will be compute at the next update call
            .for_each(|entity| {
                entity.model_mut().destination = entity_move.new_location.try_into().ok();
            });
    }

    async fn entity_spawn_event(spawn: EntitySpawn, entities: &SharedEntities) {
        let prefix = "Client: EntityController: EntitySpawnEvent: ";

        let Some(new_entity) = spawn.new_entity else {
            println!("{prefix}nothing to spawn");
            return;
        };

        match ClientEntity::try_from(new_entity) {
            Err(err) => println!("{prefix}{err}"),
            Ok(entity) => {
                println!("{prefix}{entity:?}");
                entities.lock().await.push(entity);
            }
        }
    }

    async fn entity_despawn_event(entity_despawn: EntityDespawn, entities: &SharedEntities) {
        println!(
            "Client: EntityController: EntityDespawnEvent: {:?}",
            entity_despawn
        );
        let mut entities = entities.lock().await;
        entities.retain(|entity| !entity.model().uuid().eq(&entity_despawn.uuid));
    }

    async fn handle_server_messages(
        sequences: &EntityEventSequences,
        server_message: Result<Option<ServerEntityEvent>, Status>,
        entities: &SharedEntities,
    ) -> Result<(), ServerError> {
        // TODO: we should only do pre-processing here (handling sequences, etc..)
        // then server inputs should be send through channel to controller
        // therefore the controller will handle server inputs
        // and will update game state
        match server_message {
            Ok(Some(server_event)) => match server_event.event {
                Some(Move(em)) => Self::entity_move_event(em, entities).await,
                Some(Spawn(es)) => Self::entity_spawn_event(es, entities).await,
                Some(Despawn(ed)) => Self::entity_despawn_event(ed, entities).await,
                Some(AcceptedPath(accepted_path)) => {
                    if accepted_path.sequence_number < sequences.get(EventKind::RequestPath) {
                        // When client-side sequence number is higher than this request
                        // it mean this AcceptedPath is outdated, we can ignore it.
                        return Ok(());
                    }
                    todo!()
                }
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
        sequences: &mut EntityEventSequences,
        stream_tx: &Sender<ClientEntityEvent>,
        event: EntityEvent,
    ) -> Result<(), ServerError> {
        let net_event = event
            .into_net_event(sequences)
            .map_err(|_| ServerError::EventError)?;

        if let Err(error) = stream_tx.send(net_event).await {
            println!(
                "Entity stream tx error, maybe the rx has been dropped (grpc_codegen side): {:?}",
                error
            );
            return Err(ServerError::SenderStream);
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
