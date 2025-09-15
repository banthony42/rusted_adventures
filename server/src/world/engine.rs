use std::time::Duration;

use super::monsters::Spawner;
use common::{constants::Species, utils::get_timestamp, WorldCoord};

pub trait WorldEngineComponent {
    fn update(&mut self, delta_ts: u128);
}

pub struct WorldEngine {
    wps: Duration,
    spawners: Vec<Spawner>,
    ts: u128,
}

impl WorldEngine {
    pub fn new() -> Self {
        WorldEngine {
            spawners: vec![
                Spawner::new(Species::Bouftou, WorldCoord { x: 0, y: 0 }, 3),
                Spawner::new(Species::Crabedoeuf, WorldCoord { x: 1, y: 0 }, 3),
            ],
            // Compute wait per seconds, wps
            wps: Duration::from_millis(1000 / 60),
            ts: get_timestamp(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let now = get_timestamp();
            let delta_ts = now - self.ts;
            self.ts = now;

            self.update(delta_ts);
            self.wait();
        }
    }

    fn update(&mut self, delta_ts: u128) {
        for spawner in self.spawners.iter_mut() {
            spawner.update(delta_ts);
        }
    }

    pub fn wait(&mut self) {
        std::thread::sleep(self.wps);
    }
}
