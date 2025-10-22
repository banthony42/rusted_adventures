use crate::entities::model::{EntityModel, IEntity};
use crate::states::states::GameData;
use common::constants::{Species, SERVER_ENDPOINT};
use common::rpc_extentions::{RpcCoordExtension, RpcLocationExtension};

use super::task::{TaskData, TaskInterface};
use common::grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use common::grpc_codegen::rpg_entity_client::RpgEntityClient;
use common::grpc_codegen::{AuthReply, AuthRequest, EmptyRequest, Entities, Entity, PlayerData};
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
            let data = GameData::Player((&entity).try_into()?);
            locked_task.data.push(data);
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
            .filter_map(|entity| entity.try_into().ok())
            .collect();

        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Entities(entities_data));
        Ok(())
    }
}

impl TryInto<EntityModel> for &Entity {
    type Error = String;

    fn try_into(self) -> Result<EntityModel, Self::Error> {
        let family = self
            .family
            .ok_or_else(|| "TryInto<EntityModel> for &Entity: Failed because family is None.")?;

        let mut entity_model = EntityModel::new(
            self.name.clone(),
            self.uuid.clone(),
            Species::try_from(family)?,
        );

        let (cell_rpc_coord, map_rpc_coord) = self
            .location
            .ok_or_else(|| "TryInto<EntityModel> for &Entity: Failed because location is None.")?
            .into_cell_map()
            .ok_or_else(|| "TryInto<EntityModel> for &Entity: Failed because location.cell and location.map should exist is None.")?;

        entity_model.set_map(map_rpc_coord.into_map());
        entity_model.set_cell(cell_rpc_coord.into_cell());
        Ok(entity_model)
    }
}
