use super::model::{Bestiary, EntityModel, IEntity};
use crate::constants::SERVER_ENDPOINT;
use crate::world::{MapCoord, WorldCoord};
use common::grpc_codegen::rpg_entity_client::RpgEntityClient;
use common::grpc_codegen::server_entity_event::Event::{
    EntityDespawnEvent, EntityMoveEvent, EntitySpawnEvent,
};
use common::grpc_codegen::{ClientEntityEvent, ServerEntityEvent};
use std::error::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{sleep, Duration};
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status, Streaming};

type ResponseStreamingServerEntityEvent = Response<Streaming<ServerEntityEvent>>;
struct EntityClientConnection {
    tx: Sender<ClientEntityEvent>,
    response: ResponseStreamingServerEntityEvent,
}

pub struct EntityClient {
    tx: Sender<ClientEntityEvent>,
    response: ResponseStreamingServerEntityEvent,
}

impl EntityClient {
    // TODO: chat/client.rs duplication code here
    fn auth_interceptor(
        login: String,
        token: String,
    ) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        return move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let login_md: MetadataValue<_> = login.parse().unwrap();
            let token_md: MetadataValue<_> = token.parse().unwrap();

            req.metadata_mut().insert("login", login_md);
            req.metadata_mut().insert("authorization", token_md);
            Ok(req)
        };
    }

    /// Consumes `self` returning the parts of the chat connexion.
    pub fn into_parts(
        self,
    ) -> (
        Sender<ClientEntityEvent>,
        ResponseStreamingServerEntityEvent,
    ) {
        (self.tx, self.response)
    }

    pub async fn connect(
        login: String,
        token: String,
    ) -> Result<EntityClient, Box<dyn Error + Send + Sync>> {
        let endpoint = Endpoint::from_static(SERVER_ENDPOINT).connect().await?;

        let mut client =
            RpgEntityClient::with_interceptor(endpoint, Self::auth_interceptor(login, token));

        // Pass the channel rx therefore we can easily write to the stream using tx
        let (tx, rx) = mpsc::channel::<ClientEntityEvent>(10);
        let response = client.entity_event_bus(ReceiverStream::new(rx)).await?;

        Ok(EntityClient { tx, response })
    }
}
