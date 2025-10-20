use common::{
    database::model::{bestiary::PgSpecies, monster::Monster},
    monster::MonsterHandler,
    world::WorldImport,
    CellCoord, MapCoord,
};
use rand::seq::IndexedRandom;
use tokio::sync::mpsc::Sender;

use crate::world::engine::{MonsterSpawn, WorldEngineComponent, WorldEvent};

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
        map: MapCoord,
        tx: &Sender<WorldEvent>,
        world_importer: &WorldImport,
    ) -> bool {
        let Some(map_data) = world_importer.atlas.get(&map) else {
            println!(
                "{}MonsterSpawn: MapCoord not found in atlas for map: {:?}",
                LOG_PREFIX, map
            );
            return false;
        };

        let cell = CellCoord::random_not_collider(&map_data.collider_map);
        match handler.create(&self.species, map, cell) {
            Ok(monster) => {
                self.running = false;
                let mob_spawn_event = WorldEvent::MonsterSpawn(MonsterSpawn {
                    map,
                    monster_id: monster.id,
                });
                if let Err(err) = tx.blocking_send(mob_spawn_event) {
                    println!(
                        "{}World transmitter send has failed sending: MonsterSpawn: {:?}",
                        LOG_PREFIX, err
                    );
                }
                return true;
            }
            Err(err) => println!(
                "{}Fail to create monster: {:?} at map: {:?} err: {:?}",
                LOG_PREFIX, self.species, map, err
            ),
        }
        false
    }
}

const SPAWNER_UPDATE_RATE: u128 = 10000;

pub struct Spawner {
    species: Vec<PgSpecies>,
    map: MapCoord,
    number: usize,
    update_timer: u128,
    spawn_orders: Vec<SpawnOrder>,
}

impl Spawner {
    pub fn new(species: Vec<PgSpecies>, map: MapCoord) -> Self {
        assert!(species.len() > 0);
        Self {
            species,
            map,
            number: 4,
            update_timer: 0,
            spawn_orders: Vec::new(),
        }
    }

    fn update_spawn_orders(&mut self, handler: &mut MonsterHandler) {
        // Foreach missing monsters on map, that is not already tracked in spawn_orders list,
        // choose randomly a monster and add it to the spawn_orders
        if let Ok(monsters) = Monster::read_all_by_map(&mut handler.connection, self.map.into()) {
            let missing = self
                .number
                .saturating_sub(monsters.len())
                .saturating_sub(self.spawn_orders.len());
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
                    let order = SpawnOrder {
                        species: species.clone(),
                        timer: 0,
                        running: true,
                    };
                    println!(
                        "{}{:?} will spawn in {}s",
                        LOG_PREFIX,
                        order.species,
                        order.spawn_time() / 1000
                    );
                    self.spawn_orders.push(order);
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
    fn update(
        &mut self,
        delta_ts: u128,
        handler: &mut MonsterHandler,
        tx: &Sender<WorldEvent>,
        world_importer: &WorldImport,
    ) {
        if self.update_timer > SPAWNER_UPDATE_RATE {
            self.update_timer = 0;
            self.update_spawn_orders(handler);
        } else {
            self.update_timer += delta_ts;
        }
        // Update all spawn orders timer, and, when timer is reached
        // spawn the associated monster on the map
        for order in self.spawn_orders.iter_mut() {
            if order.timer > order.spawn_time() {
                order.spawn(handler, self.map, tx, world_importer);
            } else {
                order.timer += delta_ts;
            }
        }
        // Drop all outdated orders
        self.spawn_orders.retain(|order| order.running);
    }
}
