use crate::entities::model::{EntityModel, IEntity};
use common::constants::{Species, SERVER_ENDPOINT};

use super::task::{GameData, TaskData, TaskInterface};
use common::grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use common::grpc_codegen::rpg_entity_client::RpgEntityClient;
use common::grpc_codegen::{AuthReply, AuthRequest, EmptyRequest, Entities, Entity, PlayerData};
use common::{CellCoord, WorldCoord};
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::async_trait;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Status};

#[derive(Clone)]
pub struct ConnectionTask {
    data: Arc<Mutex<TaskData>>,
    login: String,
    password: String,
    timeout: u64,
}

#[async_trait]
impl TaskInterface for ConnectionTask {
    fn get_timeout(&self) -> u64 {
        self.timeout
    }

    fn get_shared_data(&self) -> Arc<Mutex<TaskData>> {
        self.data.clone()
    }

    async fn task(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let token = match self.connect_user().await {
            Ok(t) => t,
            Err(e) => {
                let mut locked_task = self.data.lock().unwrap();
                let tonic_error_msg = tonic::Status::from_error(e);
                locked_task
                    .data
                    .push(GameData::Message(tonic_error_msg.message().to_string()));
                return Err(tonic_error_msg.into());
            }
        };

        // Simulate additional request to server
        let _ = tokio::join!(
            self.fetch_player_request(&token),
            self.fetch_entity_request(&token)
        );

        let mut locked_task = self.data.lock().unwrap();
        locked_task.success = true;
        Ok(())
    }
}

impl ConnectionTask {
    pub fn new(login: String, password: String) -> Self {
        let mut data = TaskData::default();
        data.steps = 3;

        ConnectionTask {
            data: Arc::new(Mutex::new(data)),
            login: login,
            password: password,
            timeout: 10000,
        }
    }

    async fn connect_user(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        if self.login.eq("offline") {
            // TODO: offline mode make user enter in game mode but player entity is empty
            return Ok(String::from("offline-token"));
        }

        let mut client = RpgAuthenticateClient::connect(SERVER_ENDPOINT).await?;

        let request = tonic::Request::new(AuthRequest {
            login: self.login.clone(),
            password: self.password.clone(),
        });

        let response: tonic::Response<AuthReply> = client.authenticate_user(request).await?;
        let token = response.into_inner().token.clone();

        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Token(token.clone()));
        locked_task.data.push(GameData::Login(self.login.clone()));
        Ok(token)
    }

    // TODO: duplicate code from ChatClient and EntityClient
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

    async fn fetch_player_request(
        &self,
        token: &String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let endpoint = Endpoint::from_static(SERVER_ENDPOINT).connect().await?;

        let mut client = RpgEntityClient::with_interceptor(
            endpoint,
            Self::auth_interceptor(self.login.clone(), token.clone()),
        );

        let request = tonic::Request::new(EmptyRequest {});
        let response: tonic::Response<PlayerData> = client.get_player(request).await?;
        let response_player_data = response.into_inner().entity;

        if let Some(entity) = response_player_data {
            let mut locked_task = self.data.lock().unwrap();
            locked_task.step += 1;
            locked_task
                .data
                .push(GameData::Player((&entity).try_into().unwrap()));
        }
        Ok(())
    }

    async fn fetch_entity_request(
        &self,
        token: &String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let endpoint = Endpoint::from_static(SERVER_ENDPOINT).connect().await?;

        let mut client = RpgEntityClient::with_interceptor(
            endpoint,
            Self::auth_interceptor(self.login.clone(), token.clone()),
        );

        let request = tonic::Request::new(EmptyRequest {});
        let response: tonic::Response<Entities> = client.get_entities(request).await?;
        let response_player_data = response.into_inner().entities;
        let entities_data: Vec<EntityModel> = response_player_data
            .iter()
            .map(|e| e.try_into().unwrap())
            .collect();

        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Entities(entities_data));
        Ok(())
    }
}

impl TryInto<EntityModel> for &Entity {
    type Error = &'static str;

    fn try_into(self) -> Result<EntityModel, Self::Error> {
        let species = Species::from(self.family.unwrap());
        let mut entity_model = EntityModel::new(self.name.clone(), self.uuid.clone(), species);
        // For now i don't find the tonic / gRPC syntax or trick to force a field to not be an rust Option
        // entity.proto Location.world and Location.map should be always defined
        // That's why here i use massively unwrap() for now, i want the code to fail explictly here
        let rpc_world = self.location.unwrap().world.unwrap();
        let rpc_map = self.location.unwrap().cell.unwrap();
        entity_model.set_world(WorldCoord {
            x: rpc_world.x as i8, // protobuf smallest int type is i32
            y: rpc_world.y as i8, // protobuf smallest int type is i32
        });
        entity_model.set_cell(CellCoord {
            x: rpc_map.x,
            y: rpc_map.y,
        });
        Ok(entity_model)
    }
}
