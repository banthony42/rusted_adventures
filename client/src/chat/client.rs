use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status, Streaming};

use common::grpc_codegen::rpg_chat_client::RpgChatClient;
use common::grpc_codegen::ClientChatEvent;
use common::grpc_codegen::ServerChatEvent;
use std::error::Error;

use common::constants::CHAT_SERVER_ENDPOINT;

type ResponseStreamingServerChatEvent = Response<Streaming<ServerChatEvent>>;
pub struct ChatClient {
    tx: Sender<ClientChatEvent>,
    response: ResponseStreamingServerChatEvent,
}

impl ChatClient {
    fn auth_interceptor(
        login: String,
        token: String,
    ) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        return move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let login_md: MetadataValue<_> = login
                .parse()
                .map_err(|err| Status::invalid_argument(format!("Login: {}", err)))?;

            let token_md: MetadataValue<_> = token
                .parse()
                .map_err(|err| Status::invalid_argument(format!("Token: {}", err)))?;

            req.metadata_mut().insert("login", login_md);
            req.metadata_mut().insert("authorization", token_md);
            Ok(req)
        };
    }

    /// Consumes `self` returning the parts of the chat connexion.
    pub fn into_parts(self) -> (Sender<ClientChatEvent>, ResponseStreamingServerChatEvent) {
        (self.tx, self.response)
    }

    pub async fn connect(
        login: String,
        token: String,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let endpoint = Endpoint::from_static(CHAT_SERVER_ENDPOINT)
            .connect()
            .await?;

        let mut client =
            RpgChatClient::with_interceptor(endpoint, ChatClient::auth_interceptor(login, token));

        // Pass the channel rx therefore we can easily write to the stream using tx
        let (tx, rx) = mpsc::channel::<ClientChatEvent>(10);
        let response = client.chat(ReceiverStream::new(rx)).await?;

        Ok(ChatClient { tx, response })
    }
}
