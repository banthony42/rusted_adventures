use std::collections::HashMap;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use super::grpc_codegen::rpg_chat_server::RpgChat;
use super::grpc_codegen::server_chat_event::Event;
use super::grpc_codegen::{ChatEventType, ClientChatEvent, ServerChatEvent};

#[derive(Debug)]
pub struct RpgChatService {
    clients: Arc<Mutex<HashMap<String, Sender<Result<ServerChatEvent, Status>>>>>,
    client_event_tx: Sender<ClientChatEvent>,
}

impl RpgChatService {
    pub fn new() -> Self {
        let (chat_event_tx, mut chat_event_rx) = mpsc::channel::<ClientChatEvent>(10);

        let clients: Arc<Mutex<HashMap<String, Sender<Result<_, Status>>>>> =
            Arc::new(Mutex::new(HashMap::<
                String,
                Sender<Result<ServerChatEvent, Status>>,
            >::new()));

        let clients_clone = clients.clone();

        tokio::spawn(async move {
            while let Some(receive) = chat_event_rx.recv().await {
                match receive.event() {
                    ChatEventType::Broadcast => {
                        println!("{}: Broadcast: {}", receive.login, receive.text);
                        // Should find all player on the same map as the sender
                        // and send them the msg

                        let serv_data = ServerChatEvent {
                            text: receive.text,
                            sender: Some(receive.login),
                            event: Some(Event::ChatEvent(ChatEventType::Broadcast as i32)),
                        };
                        let registered_clients = clients_clone.lock().await;
                        for (name, s) in registered_clients.iter() {
                            match registered_clients[name].send(Ok(serv_data.clone())).await {
                                Ok(_) => {} /* Data transmit to client with success */
                                Err(err) => println!("Error: chat broadcast: {:?}", err),
                            }
                        }
                    }
                    ChatEventType::Whisper => {
                        println!("{}: Whisper: {}", receive.login, receive.text);
                        // Should find the player in the DB and ensure he is connected
                        // If it's the case send him the msg
                        if receive.recipient.is_none() {
                            return;
                        }

                        let serv_data = ServerChatEvent {
                            text: receive.text,
                            sender: Some(receive.login),
                            event: Some(Event::ChatEvent(ChatEventType::Whisper as i32)),
                        };

                        let registered_clients = clients_clone.lock().await;
                        match registered_clients[&receive.recipient.unwrap()]
                            .send(Ok(serv_data))
                            .await
                        {
                            Ok(_) => {} /* Data transmit to client with success */
                            Err(err) => println!("Error: chat whisper: {:?}", err),
                        }
                    }
                };
            }
        });

        return Self {
            clients: clients,
            client_event_tx: chat_event_tx,
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
            println!("Client chat connection: {:?}", msg);
            self.clients
                .lock()
                .await
                .insert(msg.login.clone(), tx_client);
            self.client_event_tx.clone().send(msg).await;
        }

        // task: loop on client_stream_event from the request
        //       on msg send it to SERVER_TX_CHANNEL
        let sender = self.client_event_tx.clone();
        tokio::spawn(async move {
            // Read client stream and at each data receive we should
            // send it to the master channel of the server
            while let Some(event) = client_stream_event.next().await {
                match event {
                    Ok(e) => match sender.send(e).await {
                        Ok(_) => {} /* Data transmit to client with success */
                        Err(err) => println!("====> client task: send mpsc: error: {:?}", err), // Should not append, need to gather info how to handle that (fail on mpsc channel send)
                    },
                    Err(status) => println!("===> client task: error: {:?}", status), // We should handle client disconnection here
                }
            }
        });

        // return the rx_client_channel as grpc response
        // Any data on rx is receive by grpc and transmit to client through gRPC request)
        Ok(Response::new(ReceiverStream::new(rx_client)))

        // Within main.rs:
        // A task is waiting after any data from chat_event_rx channel
        // Any event from chat_event_rx is process
        // When necessary new data are sent to concerned clients tx_client_channel
    }
}
