use crate::states::{GameData, GameEntities};
use common::constants::SERVER_ENDPOINT;
use common::network::conversion::NetConversionError;

use super::task::{TaskData, TaskInterface};
use common::grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use common::grpc_codegen::rpg_entity_client::RpgEntityClient;
use common::grpc_codegen::{AuthReply, AuthRequest, EmptyRequest, Entities};
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::async_trait;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::{Request, Status};

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Net Transport: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("Net Status: {0}")]
    Status(#[from] tonic::Status),

    #[error("No data found for player")]
    MissingPlayerEntity,

    #[error("Unexpected player data: {0}")]
    ParseEntity(#[from] NetConversionError),
}

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
                locked_task.data.push(GameData::Message(e.to_string()));
                return Err(Box::new(e));
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
        let data = TaskData {
            steps: 3,
            ..Default::default()
        };

        ConnectionTask {
            data: Arc::new(Mutex::new(data)),
            login,
            password,
            timeout: 10000,
        }
    }

    async fn connect_user(&self) -> Result<String, ConnectionError> {
        let mut client = RpgAuthenticateClient::connect(SERVER_ENDPOINT).await?;

        let request = tonic::Request::new(AuthRequest {
            login: self.login.clone(),
            password: self.password.clone(),
        });

        let response: tonic::Response<AuthReply> = client.authenticate_user(request).await?;
        let token = response.into_inner().token;

        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Token(token.clone()));
        locked_task.data.push(GameData::Login(self.login.clone()));
        Ok(token)
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

    async fn fetch_player_request(&self, token: &str) -> Result<(), ConnectionError> {
        let mut client = RpgEntityClient::with_interceptor(
            Endpoint::from_static(SERVER_ENDPOINT).connect().await?,
            Self::auth_interceptor(self.login.clone(), token.to_string()),
        );

        let entity = client
            .get_player(tonic::Request::new(EmptyRequest {}))
            .await?
            .into_inner()
            .entity
            .ok_or(ConnectionError::MissingPlayerEntity)?;

        let gd = GameData::Player(entity.try_into()?);

        {
            let mut locked_task = self.data.lock().unwrap();
            locked_task.step += 1;
            locked_task.data.push(gd);
        }

        Ok(())
    }

    async fn fetch_entity_request(&self, token: &str) -> Result<(), ConnectionError> {
        let endpoint = Endpoint::from_static(SERVER_ENDPOINT).connect().await?;

        let mut client = RpgEntityClient::with_interceptor(
            endpoint,
            Self::auth_interceptor(self.login.clone(), token.to_string()),
        );

        let request = tonic::Request::new(EmptyRequest {});
        let response: tonic::Response<Entities> = client.get_entities(request).await?;
        let response_player_data = response.into_inner().entities;
        let entities_data: GameEntities = response_player_data
            .into_iter()
            .filter_map(|entity| entity.try_into().ok())
            .collect();

        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Entities(entities_data));
        Ok(())
    }
}
