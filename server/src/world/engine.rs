use std::time::Duration;

use crate::world::behaviour::BehaviourHandler;

use super::spawner::Spawner;
use common::{
    database::model::bestiary::PgSpecies,
    grpc_codegen::{Coord as RpcCoord, Location as RpcLocation},
    monster::MonsterHandler,
    utils::get_timestamp,
    world::WorldImport,
    MapCoord,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub trait WorldEngineComponent {
    fn update(
        &mut self,
        delta_ts: u128,
        handler: &mut MonsterHandler,
        tx: &Sender<WorldEvent>,
        world_importer: &WorldImport,
    );
}

pub struct MonsterSpawn {
    pub map: MapCoord,
    pub monster_id: i32,
}

pub struct MonsterMove {
    pub identifier: String,
    pub destination: RpcLocation,
    pub map: RpcCoord,
}

pub enum WorldEvent {
    MonsterSpawn(MonsterSpawn),
    MonsterMove(MonsterMove),
}

pub struct WorldEngine {
    world_importer: WorldImport,
    wtx_to_net: Sender<WorldEvent>,
    monster_handler: MonsterHandler,
    wpl: Duration,
    spawners: Vec<Spawner>,
    behaviours: BehaviourHandler,
    last_update_ts: u128,
    network_rx: Receiver<WorldEvent>,
}

// Limit the WorldEngine infinite loop rate
const UPDATE_PER_SECOND: u64 = 60;
const WAIT_PER_LOOP: u64 = 1000 / UPDATE_PER_SECOND;

impl WorldEngine {
    pub fn new() -> (Self, Receiver<WorldEvent>, Sender<WorldEvent>) {
        let (wtx_to_net, wrx_to_net) = mpsc::channel::<WorldEvent>(10);
        let (ntx_to_world, nrx_to_world) = mpsc::channel::<WorldEvent>(10);
        (
            WorldEngine {
                network_rx: nrx_to_world,
                world_importer: WorldImport::new(),
                behaviours: BehaviourHandler::new(),
                wtx_to_net,
                monster_handler: MonsterHandler::new(),
                spawners: vec![
                    Spawner::new(vec![PgSpecies::Bouftou], MapCoord { x: 0, y: 0 }),
                    Spawner::new(vec![PgSpecies::Crabedoeuf], MapCoord { x: 1, y: 0 }),
                ],
                wpl: Duration::from_millis(WAIT_PER_LOOP),
                last_update_ts: get_timestamp(),
            },
            wrx_to_net,
            ntx_to_world,
        )
    }

    pub fn run(&mut self) {
        loop {
            let now = get_timestamp();
            let delta_ts = now - self.last_update_ts;
            self.last_update_ts = now;

            // Handle NetworkEvent
            self.update(delta_ts);
            self.wait();
        }
    }

    fn update(&mut self, delta_ts: u128) {
        for spawner in self.spawners.iter_mut() {
            spawner.update(
                delta_ts,
                &mut self.monster_handler,
                &self.wtx_to_net,
                &self.world_importer,
            );
        }
        self.behaviours.update(
            delta_ts,
            &mut self.monster_handler,
            &self.wtx_to_net,
            &self.world_importer,
        );
    }

    pub fn wait(&mut self) {
        std::thread::sleep(self.wpl);
    }
}
