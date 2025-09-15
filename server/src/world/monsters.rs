use common::{constants::Species, WorldCoord};

use crate::world::engine::WorldEngineComponent;

pub struct Spawner {
    species: Species,
    world: WorldCoord,
    number: u8,
    spawn_timer: u128,
    update_timer: u128,
}

impl Spawner {
    pub fn new(species: Species, world: WorldCoord, number: u8) -> Self {
        // Spawn `number` of `species` in the DB
        Self {
            species,
            world,
            number,
            spawn_timer: 0,
            update_timer: 0,
        }
    }
}

impl WorldEngineComponent for Spawner {
    fn update(&mut self, delta_ts: u128) {
        self.spawn_timer += delta_ts;
        self.update_timer += delta_ts;

        // when `spawn_timer` >= SPAWN_TIME
        // Check if there still `number` of `species` on `world`
        // Complete if some enitty are missing
        // reset `spawn timer`

        // when `update_timer` >= UPDATE_TIME
        // Simulate entity movements
        // reset `update_timer`
    }
}
