use super::task::{GameData, TaskData, TaskInterface};
use authentication::authenticate_client::AuthenticateClient;
use authentication::LogoutRequest;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use tonic::async_trait;

pub mod authentication {
    include!("../../../common/GRPC_codegen/authentication.rs");
}

pub struct LogoutTask {
    data: Arc<Mutex<TaskData>>,
    timeout: u64,
    login: String,
    token: String,
}

#[async_trait]
impl TaskInterface for LogoutTask {
    fn get_timeout(&self) -> u64 {
        self.timeout
    }

    fn get_shared_data(&self) -> Arc<Mutex<TaskData>> {
        self.data.clone()
    }

    async fn task(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.logout().await {
            Ok(_) => {
                let mut locked_task = self.data.lock().unwrap();
                locked_task.success = true;
                Ok(())
            }
            Err(e) => {
                let mut locked_task = self.data.lock().unwrap();
                let tonic_error_msg = tonic::Status::from_error(e);
                locked_task
                    .data
                    .push(GameData::Message(tonic_error_msg.message().to_string()));
                Err(tonic_error_msg.into())
            }
        }
    }
}

impl LogoutTask {
    pub fn new(login: String, token: String) -> Self {
        let mut data = TaskData::default();
        data.steps = 1;

        LogoutTask {
            data: Arc::new(Mutex::new(data)),
            timeout: 5000,
            login,
            token,
        }
    }

    async fn logout(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut client = AuthenticateClient::connect("http://127.0.0.1:2121").await?;
        let request = LogoutRequest {
            login: self.login.clone(),
            token: self.token.clone(),
        };
        client.logout(request).await?;

        let mut locked_task = self.data.lock().unwrap();
        locked_task.step = 1;
        locked_task.data.push(GameData::Message(format!(
            "User: {} disconnected.",
            self.login
        )));
        Ok(())
    }
}
