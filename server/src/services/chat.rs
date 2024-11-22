use std::collections::HashMap;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use super::grpc_codegen::rpg_chat_server::RpgChat;
use super::grpc_codegen::{ClientChatEvent, ServerChatEvent};

#[derive(Debug)]
pub struct RpgChatService {
    client_event_tx: Sender<ClientChatEvent>,
    shared_clients: Arc<Mutex<HashMap<String, Sender<Result<ServerChatEvent, Status>>>>>,
}

impl RpgChatService {
    pub fn new(
        tx: Sender<ClientChatEvent>,
        shared_clients: Arc<Mutex<HashMap<String, Sender<Result<ServerChatEvent, Status>>>>>,
    ) -> Self {
        return Self {
            client_event_tx: tx,
            shared_clients: shared_clients,
        };
    }
}

#[tonic::async_trait]
impl RpgChat for RpgChatService {
    type ChatStream = ReceiverStream<Result<ServerChatEvent, Status>>;

    async fn chat(
        &self,
        request: Request<Streaming<ClientChatEvent>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        let mut client_stream_event = request.into_inner();

        let (tx_client, rx_client) = mpsc::channel::<Result<ServerChatEvent, Status>>(10);

        // For now we have to process the first message from the client
        // to get back it's name ...
        // store tx_client_channel in SHARED
        if let Some(msg) = client_stream_event.message().await.unwrap() {
            println!("==> {:?}", msg);
            self.shared_clients
                .lock()
                .await
                .insert(msg.login.clone(), tx_client);
            self.client_event_tx.clone().send(msg).await;
        }

        // task: loop on client_stream_event from the request
        //       on msg send it to SERVER_TX_CHANNEL
        let sender = self.client_event_tx.clone();
        tokio::spawn(async move {
            println!("===> clien task: run ...");
            // Read client stream and at each data receive we should
            // send it to the master channel of the server
            while let Some(event) = client_stream_event.next().await {
                match event {
                    Ok(e) => match sender.send(e).await {
                        Ok(_) => println!("===> clien task: send mpsc: success"),
                        Err(err) => println!("====> client task: send mpsc: error: {:?}", err),
                    },
                    Err(status) => println!("===> client task: error: {:?}", status),
                }
            }
        });

        // return the rx_client_channel as grpc response
        // (server will send event to tx_client_channel and any data on rx is receive by grpc and transmit to client)
        Ok(Response::new(ReceiverStream::new(rx_client)))

        // Outside of this (main.rs):
        // task : loop on server_rx_channel
        //        retrieve all users concerned by the event
        //        use the shared.tx_client_channel of each user to send them the event
    }
}
