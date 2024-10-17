use crate::game::Game;
use crate::{entity::Entity, world::Coord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FakeGameData {
    pub player: Entity,
    pub entities: Vec<Entity>,
}

impl FakeGameData {
    fn fetch_entities_data(_world_coord: &Coord) -> &'static str {
        // Simulate server game data response
        return r#"[
                {
                    "name" : "fealhach",
                    "type" : "Player",
                    "race" : "Character",
                    "state": "Idle",
                    "map_coord": {
                        "x": 300,
                        "y": 300
                    },
                    "world_coord": {
                        "x": 0,
                        "y": 0
                    }
                },
                {
                    "name" : "-smirnof-",
                    "type" : "Player",
                    "race" : "Character",
                    "state": "Idle",
                    "map_coord": {
                        "x": 364,
                        "y": 364
                    },
                    "world_coord": {
                        "x": 0,
                        "y": 0
                    }
                },
                {
                    "name": "Bouftou",
                    "type" : "Monster",
                    "race": "Bouftou",
                    "state": "Idle",
                    "map_coord": {
                        "x": 750,
                        "y": 550
                    },
                    "world_coord": {
                        "x": 0,
                        "y": 0
                    }
                }
            ]
        "#;
    }

    fn fetch_player_data() -> &'static str {
        // Simulate server game data response
        return r#"{
                "name": "Sulfurel",
                "type": "Player",
                "race": "Character",
                "state": "Idle",
                "map_coord": {
                    "x": 500,
                    "y": 500,
                    "label": "Mountain"
                },
                "world_coord": {
                    "x": 0,
                    "y": 0
                }
        }"#;
    }

    pub fn get_data_from_server() -> Result<FakeGameData, String> {
        let json_player_data = Self::fetch_player_data();

        let p_data = match serde_json::from_str::<Entity>(json_player_data) {
            Ok(game_data) => game_data,
            Err(error) => {
                return Err(format!(
                    "client: get_data_from_server: Error while deserializing data. {error}"
                ))
            }
        };

        let json_entities_data = Self::fetch_entities_data(&p_data.world_coord);
        let e_data = match serde_json::from_str::<Vec<Entity>>(json_entities_data) {
            Ok(game_data) => game_data,
            Err(error) => {
                return Err(format!(
                    "client: get_data_from_server: Error while deserializing data. {error}"
                ))
            }
        };

        return Ok(FakeGameData {
            player: p_data,
            entities: e_data,
        });
    }
}

use authentication::authenticate_client::AuthenticateClient;
use authentication::{AuthReply, AuthRequest};
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
pub mod authentication {
    include!("../../common/GRPC_codegen/authentication.rs");
}

#[derive(Debug, Clone)]
pub enum GameData {
    Token(String),
    Message(String),
    Entities(Vec<bool>),
}

pub struct TaskData {
    pub steps: u16,
    pub step: u16,
    pub success: bool,
    pub data: Vec<GameData>,
}

#[derive(Clone)]
pub struct ConnectionTask {
    data: Arc<Mutex<TaskData>>,
    login: String,
    password: String,
    timeout: u64,
}

pub trait TaskInterface {
    fn get_timeout(&self) -> u64;
    fn get_shared_data(&self) -> Arc<Mutex<TaskData>>;
    async fn task(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

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
                locked_task
                    .data
                    .push(GameData::Message(e.to_string()));
                return Err(e);
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
        let data = TaskData {
            steps: 3,
            step: 0,
            success: false,
            data: Vec::new(),
        };

        ConnectionTask {
            data: Arc::new(Mutex::new(data)),
            login: login,
            password: password,
            timeout: 10000,
        }
    }

    async fn connect_user(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        println!("connect_user begin");
        let mut client = AuthenticateClient::connect("http://127.0.0.1:2121").await?;

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
