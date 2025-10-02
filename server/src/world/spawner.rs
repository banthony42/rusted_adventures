use common::{
    database::model::{bestiary::PgSpecies, monster::Monster},
    monster::MonsterHandler,
    WorldCoord,
};
use rand::seq::IndexedRandom;
use tokio::sync::mpsc::Sender;

use crate::world::engine::{WorldEngineComponent, WorldEvent, WorldEventType};

const LOG_PREFIX: &str = "Server: WorldEngine: Spawner: ";

struct SpawnOrder {
    species: PgSpecies,
    running: bool,
    timer: u128,
}

impl SpawnOrder {
    fn spawn_time(&self) -> u128 {
        match self.species {
            PgSpecies::Bouftou => 60000,
            PgSpecies::Crabedoeuf => 30000,
        }
    }

    fn spawn(
        &mut self,
        handler: &mut MonsterHandler,
        world: WorldCoord,
        tx: &Sender<WorldEvent>,
    ) -> bool {
        match handler.create(&self.species, world) {
            Ok(monster) => {
                println!(
                    "{}Monster creation succeed: {:?} {:?}",
                    LOG_PREFIX, self.species, world
                );
                self.running = false;
                let mob_spawn_event = WorldEvent {
                    event: WorldEventType::MonsterSpawn,
                    world,
                    monster_id: monster.id,
                };
                if let Err(err) = tx.blocking_send(mob_spawn_event) {
                    println!(
                        "{}World transmitter send has failed sending: MonsterSpawn: {:?}",
                        LOG_PREFIX, err
                    );
                }
                return true;
            }
            Err(err) => println!(
                "{}Fail to create monster: {:?} at world: {:?} err: {:?}",
                LOG_PREFIX, self.species, world, err
            ),
        }
        false
    }
}

const SPAWNER_UPDATE_RATE: u128 = 10000;

pub struct Spawner {
    species: Vec<PgSpecies>,
    world: WorldCoord,
    number: usize,
    update_timer: u128,
    spawn_orders: Vec<SpawnOrder>,
}

impl Spawner {
    pub fn new(species: Vec<PgSpecies>, world: WorldCoord) -> Self {
        assert!(species.len() > 0);
        Self {
            species,
            world,
            number: 3,
            update_timer: 0,
            spawn_orders: Vec::new(),
        }
    }

    fn update_spawn_orders(&mut self, handler: &mut MonsterHandler) {
        // Foreach missing monsters on map, that is not already tracked in spawn_orders list,
        // choose randomly a monster and add it to the spawn_orders
        if let Ok(monsters) = Monster::read_all_by_world(&mut handler.connection, self.world.into())
        {
            let missing = self.number - monsters.len() - self.spawn_orders.len();
            // Even if several monsters are missing, only spawn one
            // to create delay between each spawn
            if missing > 0 {
                println!(
                    "{}update orders: missing: {} monsters: {}",
                    LOG_PREFIX,
                    missing,
                    monsters.len()
                );
                if let Some(species) = self.species.choose(&mut rand::rng()) {
                    self.spawn_orders.push(SpawnOrder {
                        species: species.clone(),
                        timer: 0,
                        running: true,
                    });
                } else {
                    println!("{}update: Failed to randomly choose species.", LOG_PREFIX);
                }
                println!(
                    "{}update orders: orders: {}",
                    LOG_PREFIX,
                    self.spawn_orders.len()
                )
            }
        }
    }
}

impl WorldEngineComponent for Spawner {
    fn update(&mut self, delta_ts: u128, handler: &mut MonsterHandler, tx: &Sender<WorldEvent>) {
        if self.update_timer > SPAWNER_UPDATE_RATE {
            self.update_timer = 0;
            self.update_spawn_orders(handler);
            for order in self.spawn_orders.iter() {
                if order.running {
                    println!(
                        "{}{:?} will spawn in {}s",
                        LOG_PREFIX,
                        order.species,
                        (order.spawn_time() - order.timer) / 1000
                    );
                }
            }
        } else {
            self.update_timer += delta_ts;
        }
        // Update all spawn orders timer, and, when timer is reached
        // spawn the associated monster on the world
        for order in self.spawn_orders.iter_mut() {
            if order.timer > order.spawn_time() {
                order.spawn(handler, self.world, tx);
            } else {
                order.timer += delta_ts;
            }
        }
        // Drop all outdated orders
        self.spawn_orders.retain(|order| order.running);
    }
}
