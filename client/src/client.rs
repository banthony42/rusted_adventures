use serde::{Deserialize, Serialize};
use crate::{entity::Player, world::Coord};


#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub player: Player,
}

impl GameData {

    fn fetch_entities_from_server(world_coord: Coord) -> &'static str {
        // Simulate server game data response
        return r#"{
            "entities": [
                {
                    "type" : "player",
                    "name" : "Walter White",
                    "class" : "chemist",
                    "map_coord": {
                        "x": 1,
                        "y": 1
                    }
                },
                {
                    "type" : "bouftou",
                    "map_coord": {
                        "x": 8,
                        "y": 8
                    }                  
                }
            ]
        }"#
    }

    fn fetch_data_from_server() -> &'static str {
        // Simulate server game data response
        return r#"{
            "player": {
                "base": {
                    "name": "John Snow",
                    "texture": "Character",
                    "map_coord": {
                        "x": 8,
                        "y": 8,
                        "label": "Mountain"
                    }
                },
                "world_coord": {
                    "x": 0,
                    "y": 0
                }
            }
        }"#
    }

    pub fn get_data_from_server() -> Result<GameData, String> {
        let json_game_data = Self::fetch_data_from_server();

        return match serde_json::from_str::<GameData>(json_game_data) {
            Ok(game_data) => {
                println!("=====================\n{:?}", game_data);
                Ok(game_data)
            },
            Err(error) => {
                Err(format!("client: get_data_from_server: Error while deserializing data. {error}"))
            }
        }
    }
}