use crate::constants::SERVER_ENDPOINT;

use super::task::{GameData, TaskData, TaskInterface};
use common::grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use common::grpc_codegen::{AuthReply, AuthRequest};
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::async_trait;

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
            self.simulate_player_request(&token),
            self.simulate_entity_request(&token)
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
        Ok(token)
    }

    async fn simulate_player_request(
        &self,
        _token: &String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Message(self.login.clone()));
        Ok(())
    }

    async fn simulate_entity_request(
        &self,
        _token: &String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Entities(Vec::new()));
        Ok(())
    }
}
