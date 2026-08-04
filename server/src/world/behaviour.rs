use common::{
    database::model::{monster::Monster, EntityIdentifiable},
    grpc_codegen::{Coord as RpcCoord, Location as RpcLocation},
    monster::MonsterHandler,
    world::WorldImport,
    CellCoord, MapCoord, Orientation,
};
use tokio::sync::mpsc::Sender;

use crate::world::engine::{MonsterMove, WorldEngineComponent, WorldEvent};

type BehaviourTaskCallback = Box<
    dyn Fn(i32, &mut MonsterHandler, &Sender<WorldEvent>, &WorldImport) -> bool + Send + 'static,
>;

const LOG_PREFIX: &str = "Server: WorldEngine: Behaviour:";
const BEHAVIOUR_UPDATE_RATE: u128 = 10000;
const MOVE_BEHAVIOUR_RATE: u128 = 8000;
const MONSTER_PM: i64 = 5;

struct BehaviourTask {
    monster_id: i32,
    timer: u128,
    running: bool,
    callback: BehaviourTaskCallback,
}

impl BehaviourTask {
    fn new(monster_id: i32, callback: BehaviourTaskCallback) -> Self {
        let mut task = BehaviourTask {
            monster_id,
            timer: 0,
            running: true,
            callback,
        };
        task.reset_timer();
        task
    }

    fn reset_timer(&mut self) {
        self.timer = rand::random_range(MOVE_BEHAVIOUR_RATE / 2..MOVE_BEHAVIOUR_RATE);
    }

    fn execute(
        &mut self,
        handler: &mut MonsterHandler,
        world_tx: &Sender<WorldEvent>,
        world_importer: &WorldImport,
    ) {
        self.running = (self.callback)(self.monster_id, handler, world_tx, world_importer);
        if self.running {
            self.reset_timer();
        }
    }
}

fn move_behaviour(
    monster_id: i32,
    handler: &mut MonsterHandler,
    world_tx: &Sender<WorldEvent>,
    world_importer: &WorldImport,
) -> bool {
    let monster = match Monster::read_info(&mut handler.connection, &monster_id) {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{LOG_PREFIX} Drop behaviour task for monster id: {monster_id}: {e}");
            return false;
        }
    };

    let m = MapCoord {
        x: monster.map.0 as i8,
        y: monster.map.1 as i8,
    };

    let Some(map_data) = world_importer.atlas.get(&m) else {
        tracing::warn!("{LOG_PREFIX} Drop behaviour task for monster id: {monster_id}: {m:?} not found in atlas");
        return false;
    };

    let mut new_destination = CellCoord {
        x: monster.cell.0 as i64,
        y: monster.cell.1 as i64,
    };

    let mut last_orientation: Option<Orientation> = None;
    for _ in 0..MONSTER_PM {
        loop {
            let orientation = loop {
                match rand::random::<Orientation>() {
                    pick if last_orientation.is_none() => break pick,
                    // We don't want the entity to go back (should not be the invert of the last)
                    pick if last_orientation.is_some_and(|inner| inner.invert() != pick) => {
                        break pick
                    }
                    // TODO: Test if both lines above can be simplify by:
                    // pick if Some(pick.invert()) == last_orientation => break pick
                    _ => { /* pick is same from the last loop, retry */ }
                }
            };
            let step = match orientation {
                Orientation::North => CellCoord { x: 0, y: -1 },
                Orientation::Est => CellCoord { x: 1, y: 0 },
                Orientation::South => CellCoord { x: 0, y: 1 },
                Orientation::West => CellCoord { x: -1, y: 0 },
            };
            let cell = (new_destination + step).limit();
            if map_data
                .collider_map
                .is_not_collider(cell.y as usize, cell.x as usize)
            {
                last_orientation = Some(orientation);
                new_destination = cell;
                break;
            }
        }
    }

    // TODO: .limit() seems useless since already call in the previous loop
    new_destination = new_destination.limit();
    let new_rpc_loc = RpcLocation {
        map: Some(monster.map.into()),
        cell: Some(RpcCoord {
            x: new_destination.x,
            y: new_destination.y,
        }),
    };

    // We accept that players who appear on the map
    // will see the monster directly at its new location,
    // while players already on the map will see the monster move to a new location
    if let Err(err) = handler.update_location(monster.entity_id, new_rpc_loc) {
        tracing::error!(
            "{LOG_PREFIX} Drop behaviour task for monster id: {monster_id}: Update location: {err}",
        );
        return false;
    }

    // send MonsterMove event through tx
    let move_event = WorldEvent::MonsterMove(MonsterMove {
        identifier: monster.identifier(),
        destination: new_rpc_loc,
        map: monster.map.into(),
    });

    if let Err(err) = world_tx.blocking_send(move_event) {
        tracing::error!("{LOG_PREFIX} Fail to send WorldEvent: MonsterMove: {err}");
    }
    // Return true, therefore this behaviour never stop
    // It will be dropped when the associated monster disappear from database
    return true;
}

pub struct BehaviourHandler {
    tasks: Vec<BehaviourTask>,
    timer: u128,
}

impl BehaviourHandler {
    pub fn new() -> Self {
        BehaviourHandler {
            tasks: Vec::new(),
            timer: 0,
        }
    }

    fn add(&mut self, new_task: BehaviourTask) {
        self.tasks.push(new_task);
    }

    fn task_exist_with(&self, monster_id: i32) -> bool {
        self.tasks
            .iter()
            .any(|task| monster_id.eq(&task.monster_id))
    }
}

impl WorldEngineComponent for BehaviourHandler {
    fn update(
        &mut self,
        delta_ts: u128,
        handler: &mut MonsterHandler,
        tx: &Sender<WorldEvent>,
        world_importer: &WorldImport,
    ) {
        // Periodically load Monsters from DB
        // Create behaviours (timer + callback) for each monster.id
        if self.timer > BEHAVIOUR_UPDATE_RATE {
            if let Ok(monsters) = Monster::read_all(&mut handler.connection) {
                // Drop all BehaviourTask associated to inexistant monster id in DB
                self.tasks.retain(|task| {
                    monsters
                        .iter()
                        .any(|monster| monster.id.eq(&task.monster_id))
                });

                // Create BehaviourTask for each monster id not already registered
                for monster in monsters.iter() {
                    if self.task_exist_with(monster.id) == false {
                        self.add(BehaviourTask::new(monster.id, Box::new(move_behaviour)));
                    }
                }
            }
            self.timer = 0;
        } else {
            self.timer += delta_ts;
        }

        for task in self.tasks.iter_mut() {
            if task.timer == 0 {
                task.execute(handler, tx, world_importer);
            } else {
                task.timer = task.timer.saturating_sub(delta_ts);
            }
        }
        // Drop all finished BehaviourTasks
        self.tasks.retain(|task| task.running);
    }
}
