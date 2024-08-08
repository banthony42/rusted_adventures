use serde::{Deserialize, Serialize};
use crate::{
    entity::Entity,
    world::Coord
};


#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub player: Entity,
    pub entities: Vec<Entity>
}

impl GameData {

    fn fetch_entities_data(_world_coord: &Coord) -> &'static str {
        // Simulate server game data response
        return r#"[
                {
                    "name" : "Walter White",
                    "type" : "Player",
                    "texture" : "Character",
                    "map_coord": {
                        "x": 4,
                        "y": 4
                    },
                    "world_coord": {
                        "x": 0,
                        "y": 0
                    }
                },
                {
                    "name": "Bouftou",
                    "type" : "Monster",
                    "texture": "Bouftou",
                    "map_coord": {
                        "x": 10,
                        "y": 10
                    },
                    "world_coord": {
                        "x": 0,
                        "y": 0
                    }
                }
            ]
        "#
    }

    fn fetch_player_data() -> &'static str {
        // Simulate server game data response
        return r#"{
                "name": "Sulfurel",
                "type": "Player",
                "texture": "Character",
                "map_coord": {
                    "x": 8,
                    "y": 8,
                    "label": "Mountain"
                },
                "world_coord": {
                    "x": 0,
                    "y": 0
                }
        }"#
    }

    pub fn get_data_from_server() -> Result<GameData, String> {
        let json_player_data = Self::fetch_player_data();

        let p_data = match serde_json::from_str::<Entity>(json_player_data) {
            Ok(game_data) => game_data,
            Err(error) => return Err(format!("client: get_data_from_server: Error while deserializing data. {error}"))
        };

        let json_entities_data = Self::fetch_entities_data(&p_data.world_coord);
        let e_data = match serde_json::from_str::<Vec<Entity>>(json_entities_data) {
            Ok(game_data) => game_data,
            Err(error) => return Err(format!("client: get_data_from_server: Error while deserializing data. {error}"))
        };

        return Ok(GameData { player: p_data, entities: e_data });
    }
}