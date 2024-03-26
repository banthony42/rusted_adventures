use serde::{Deserialize, Serialize};
use crate::entity::Player;


#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub player: Player,
}

impl GameData {

    fn fetch_data_from_server() -> &'static str {
        // Simulate server game data response
        return r#"{
            "player": {
                "base": {
                    "name": "John Snow",
                    "texture": "Character",
                    "map_coord": {
                        "x": 8,
                        "y": 8
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
            Ok(game_data) => Ok(game_data),
            Err(error) => {
                Err(format!("client: get_data_from_server: Error while deserializing data. {error}"))
            }
        }
    }
}