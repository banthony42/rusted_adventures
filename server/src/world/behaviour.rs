use common::{
    database::model::{monster::Monster, EntityIdentifiable},
    grpc_codegen::{Coord as RpcCoord, Location as RpcLocation},
    monster::MonsterHandler,
    CellCoord,
};
use tokio::sync::mpsc::Sender;

use crate::world::engine::{MonsterMove, WorldEngineComponent, WorldEvent};

type BehaviourTaskCallback =
    Box<dyn Fn(i32, &mut MonsterHandler, &Sender<WorldEvent>) -> bool + Send + 'static>;

const LOG_PREFIX: &str = "Server: WorldEngine: Behaviour: ";
const BEHAVIOUR_UPDATE_RATE: u128 = 10000;
const MOVE_BEHAVIOUR_RATE: u128 = 8000;
const MONSTER_PM: i64 = 3;

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

    fn execute(&mut self, handler: &mut MonsterHandler, world_tx: &Sender<WorldEvent>) {
        self.running = (self.callback)(self.monster_id, handler, world_tx);
        if self.running {
            self.reset_timer();
        }
    }
}

fn move_behaviour(
    monster_id: i32,
    handler: &mut MonsterHandler,
    world_tx: &Sender<WorldEvent>,
) -> bool {
    let Ok(monster) = Monster::read_info(&mut handler.connection, &monster_id) else {
        println!(
            "{}Drop behaviour task for monster id: {}",
            LOG_PREFIX, monster_id
        );
        return false;
    };

    let move_x: i64 = rand::random_range(-MONSTER_PM..MONSTER_PM);
    let remaining_pm = MONSTER_PM - move_x.abs();
    let move_y: i64 = remaining_pm
        .gt(&0)
        .then(|| rand::random_range(-remaining_pm..remaining_pm))
        .unwrap_or(0);

    let new_destination = CellCoord {
        x: monster.cell.0 as i64 + move_x,
        y: monster.cell.1 as i64 + move_y,
    }
    .limit();

    let new_rpc_loc = RpcLocation {
        map: Some(monster.map.into()),
        cell: Some(RpcCoord {
            x: new_destination.x,
            y: new_destination.y,
        }),
    };

    // It's assume that player spawning on map
    // will see the monster directly at its new location
    // whereas player already on the map will see the monster move to new location
    if let Err(err) = handler.update_location(monster.entity_id, new_rpc_loc) {
        println!(
            "{}Error while updating location for monster id:{} {}. Behaviour has been dropped.",
            LOG_PREFIX, monster_id, err
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
        println!(
            "{}World transmitter send has failed sending: MonsterMove: {:?}",
            LOG_PREFIX, err
        );
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
    fn update(&mut self, delta_ts: u128, handler: &mut MonsterHandler, tx: &Sender<WorldEvent>) {
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
                task.execute(handler, tx);
            } else {
                task.timer = task.timer.saturating_sub(delta_ts);
            }
        }
        // Drop all finished BehaviourTasks
        self.tasks.retain(|task| task.running);
    }
}
