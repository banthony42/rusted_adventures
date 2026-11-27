use crate::{CellCoord, Position};

#[derive(Debug, Clone)]
pub struct EntityModel {
    name: String,
    uuid: String,
    pub position: Position,
    pub destination: Option<Position>,
    pub path: Option<Vec<CellCoord>>,
}

impl EntityModel {
    pub fn new(name: String, uuid: String, position: Position) -> Self {
        Self {
            name,
            uuid,
            position,
            destination: None,
            path: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn uuid(&self) -> &String {
        &self.uuid
    }
}
