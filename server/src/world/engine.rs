use std::time::Duration;

use crate::world::behaviour::BehaviourHandler;

use super::spawner::Spawner;
use common::{
    character::CharacterHandler,
    database::model::bestiary::PgSpecies,
    monster::MonsterHandler,
    network::events::EntityEvent,
    path_finding::{astar::AStar, PathFinder, PathFindingStrategy},
    utils::get_timestamp,
    world::WorldMap,
    MapCoord, Position,
};
use tokio::sync::mpsc::{self, error::TryRecvError, Receiver, Sender};

pub trait WorldEngineComponent {
    fn update(
        &mut self,
        delta_ts: u128,
        handler: &mut MonsterHandler,
        tx: &Sender<WorldEvent>,
        world_map: &WorldMap,
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
    fn proceed<T: PathFindingStrategy>(
        &self,
        sender: String,
        world_map: &WorldMap,
        path_finder: &mut PathFinder<T>,
    );
}

impl HandleNetEvent for EntityEvent {
    fn proceed<T: PathFindingStrategy>(
        &self,
        sender: String,
        world_map: &WorldMap,
        path_finder: &mut PathFinder<T>,
    ) {
        let mut char_handler = CharacterHandler::new(&sender).unwrap();
        match self {
            EntityEvent::RequestPath(position, destination) => {
                let Ok(player_position) = char_handler.location() else {
                    tracing::error!("failed to retrieve player location");
                    return;
                };

                if player_position != *position {
                    tracing::error!("client and server player location mismatch");
                    // TODO: Send Update ?
                    return;
                }

                let Some(map) = world_map.atlas.get(&player_position.map) else {
                    tracing::error!(
                        "player map not found in world atlas: {:?}",
                        player_position.map
                    );
                    return;
                };

                // Compute authoritative A* path.
                match path_finder.compute(player_position.cell, *destination, &map.collider_map) {
                    true => {
                        // TODO: Evaluate if the end of the path lead to another map (map adjacency or portal)
                        // TODO: Launch player movements simulation
                        // TODO: Send AcceptedPath to the sender containing :
                        // - authoritative full path
                        // - Next map to reach at the end of the path (Optional)
                        // TODO: Broadcast movement event
                    }
                    false => {
                        // TODO: Send RejectedPath to the sender
                    }
                }
            }
        }
    }
}

pub struct EntityNetEvent {
    sender: String,
    event: EntityEvent,
}

impl EntityNetEvent {
    pub fn new(sender: String, event: EntityEvent) -> Self {
        Self { sender, event }
    }
}

pub struct WorldEngine {
    world_map: WorldMap,
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
                world_map: WorldMap::new(),
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
            self.receive_events();
            self.update(delta_ts);
            self.wait();
        }
    }

    fn receive_events(&mut self) {
        match self.net_rx.try_recv() {
            Ok(net) => net
                .event
                .proceed(net.sender, &self.world_map, &mut self.path_finder),
            Err(TryRecvError::Empty) => { /* Don't log on empty channel */ }
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
                &self.world_map,
            );
        }
        self.behaviours.update(
            delta_ts,
            &mut self.monster_handler,
            &self.wtx_to_net,
            &self.world_map,
        );
    }

    pub fn wait(&mut self) {
        std::thread::sleep(self.wpl);
    }
}
