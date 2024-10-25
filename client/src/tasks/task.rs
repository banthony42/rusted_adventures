use std::error::Error;
use std::sync::{Arc, Mutex};
use tonic::async_trait;

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

#[async_trait]
pub trait TaskInterface: Send + Sync {
    fn get_timeout(&self) -> u64;
    fn get_shared_data(&self) -> Arc<Mutex<TaskData>>;
    async fn task(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
