use crate::{entity::Entity, world::Coord_tmp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FakeGameData {
    pub player: Entity,
    pub token: String,
    pub entities: Vec<Entity>,
}

impl FakeGameData {
    fn fetch_entities_data(_world_coord: &Coord_tmp) -> &'static str {
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
                        "x": 450,
                        "y": 450
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
                    "x": 544,
                    "y": 544,
                    "label": "Mountain"
                },
                "world_coord": {
                    "x": 1,
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
            token: String::default(),
        });
    }
}
