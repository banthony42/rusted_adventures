use std::time::Duration;

use crate::world::behaviour::BehaviourHandler;

use super::spawner::Spawner;
use common::{
    character::CharacterHandler,
    database::model::bestiary::PgSpecies,
    grpc_codegen::{Coord as RpcCoord, Location as RpcLocation},
    monster::MonsterHandler,
    path_finding::{astar::AStar, PathFinder, PathFindingStrategy},
    utils::get_timestamp,
    world::WorldImport,
    MapCoord, Position,
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
    pub destination: Position,
    pub map: MapCoord,
}

pub enum WorldEvent {
    MonsterSpawn(MonsterSpawn),
    MonsterMove(MonsterMove),
}

trait HandleNetEvent {
    fn proceed(&self, sender: String);
}

pub enum NetEvent {
    RequestPath(Position),
}

impl HandleNetEvent for NetEvent {
    fn proceed(&self, sender: String) {
        let char_handler = CharacterHandler::new(&sender);
        match self {
            NetEvent::RequestPath(cell_coord) => {
                // Receive destination request.
                // Validate player position and destination.
                // Compute authoritative A* path.
                // Update authoritative movement state.
                // Broadcast movement information.
                todo!()
            }
        }
    }
}

pub struct EntityNetEvent {
    sender: String,
    event: NetEvent,
}

impl EntityNetEvent {
    pub fn new(sender: String, event: NetEvent) -> Self {
        Self { sender, event }
    }
}

pub struct WorldEngine {
    world_importer: WorldImport,
    path_finder: PathFinder<AStar>,
    wtx_to_net: Sender<WorldEvent>,
    monster_handler: MonsterHandler,
    wpl: Duration,
    spawners: Vec<Spawner>,
    behaviours: BehaviourHandler,
    last_update_ts: u128,
    net_rx: Receiver<EntityNetEvent>,
}

// Limit the WorldEngine infinite loop rate
const UPDATE_PER_SECOND: u64 = 60;
const WAIT_PER_LOOP: u64 = 1000 / UPDATE_PER_SECOND;

impl WorldEngine {
    pub fn new() -> (Self, Receiver<WorldEvent>, Sender<EntityNetEvent>) {
        let (world_tx, world_rx) = mpsc::channel::<WorldEvent>(10);
        let (net_tx, net_rx) = mpsc::channel::<EntityNetEvent>(10);
        (
            WorldEngine {
                net_rx,
                world_importer: WorldImport::new(),
                path_finder: PathFinder::new(AStar::new()),
                behaviours: BehaviourHandler::new(),
                wtx_to_net: world_tx,
                monster_handler: MonsterHandler::new(),
                spawners: vec![
                    Spawner::new(vec![PgSpecies::Bouftou], MapCoord { x: 0, y: 0 }),
                    Spawner::new(vec![PgSpecies::Crabedoeuf], MapCoord { x: 1, y: 0 }),
                ],
                wpl: Duration::from_millis(WAIT_PER_LOOP),
                last_update_ts: get_timestamp(),
            },
            world_rx,
            net_tx,
        )
    }

    pub fn run(&mut self) {
        loop {
            let now = get_timestamp();
            let delta_ts = now - self.last_update_ts;
            self.last_update_ts = now;

            // Handle NetworkEvent
            // self.receive_events();
            self.update(delta_ts);
            self.wait();
        }
    }

    fn receive_events(&mut self) {
        match self.net_rx.try_recv() {
            Ok(net) => net.event.proceed(net.sender),
            Err(err) => {
                tracing::error!("WorldEngine: error trying to receive EntityNetEvent: {err}")
            }
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
