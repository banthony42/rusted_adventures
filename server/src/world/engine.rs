use std::time::Duration;

use super::spawner::Spawner;
use common::{
    database::model::bestiary::PgSpecies, monster::MonsterHandler, utils::get_timestamp, WorldCoord,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub trait WorldEngineComponent {
    fn update(&mut self, delta_ts: u128, handler: &mut MonsterHandler, tx: &Sender<WorldEvent>);
}

pub enum WorldEventType {
    MonsterSpawn,
}

pub struct WorldEvent {
    pub event: WorldEventType,
    pub world: WorldCoord,
    pub monster_id: i32,
}

pub struct WorldEngine {
    tx: Sender<WorldEvent>,
    monster_handler: MonsterHandler,
    wpl: Duration,
    spawners: Vec<Spawner>,
    ts: u128,
}

// Limit the WorldEngine infinite loop rate
const UPDATE_PER_SECOND: u64 = 60;
const WAIT_PER_LOOP: u64 = 1000 / UPDATE_PER_SECOND;

impl WorldEngine {
    pub fn new() -> (Self, Receiver<WorldEvent>) {
        let (tx, rx) = mpsc::channel::<WorldEvent>(10);
        (
            WorldEngine {
                tx,
                monster_handler: MonsterHandler::new(),
                spawners: vec![
                    Spawner::new(vec![PgSpecies::Bouftou], WorldCoord { x: 0, y: 0 }),
                    Spawner::new(vec![PgSpecies::Crabedoeuf], WorldCoord { x: 1, y: 0 }),
                ],
                wpl: Duration::from_millis(WAIT_PER_LOOP),
                ts: get_timestamp(),
            },
            rx,
        )
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
            spawner.update(delta_ts, &mut self.monster_handler, &self.tx);
        }
    }

    pub fn wait(&mut self) {
        std::thread::sleep(self.wpl);
    }
}
