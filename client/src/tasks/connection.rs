use super::task::{GameData, TaskData, TaskInterface};
use grpc_codegen::rpg_authenticate_client::RpgAuthenticateClient;
use grpc_codegen::{AuthReply, AuthRequest};
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::async_trait;

pub mod grpc_codegen {
    include!("../../../common/GRPC_codegen/rpg.package.rs");
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
                let tonic_error_msg = tonic::Status::from_error(e);
                locked_task
                    .data
                    .push(GameData::Message(tonic_error_msg.message().to_string()));
                return Err(tonic_error_msg.into());
            }
        };
        let _ = tokio::join!(
            self.dummy_api_request_1(&token),
            self.dummy_api_request_2(&token)
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
        let mut client = RpgAuthenticateClient::connect("http://127.0.0.1:2121").await?;

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

    async fn dummy_api_request_1(
        &self,
        token: &String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Message(self.login.clone()));
        Ok(())
    }

    async fn dummy_api_request_2(
        &self,
        token: &String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        let mut locked_task = self.data.lock().unwrap();
        locked_task.step += 1;
        locked_task.data.push(GameData::Entities(Vec::new()));
        Ok(())
    }
}
