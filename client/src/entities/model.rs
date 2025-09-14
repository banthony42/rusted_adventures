use common::grpc_codegen::{
    entity::Family, Classes as RpcClasses, LocationType, Species as RpcSpecies,
};
use common::{constants::*, MapCoord, WorldCoord};

use crate::{
    import::assets::{Animations, EntityAssets, Species},
    world::World,
};

const ENTITY_RUN_SPEED: f64 = 225.0;
const ENTITY_RUN_STEP_DURATION: f64 = 1.0 / ENTITY_RUN_SPEED;

#[derive(Debug, Clone)]
pub enum Orientation {
    Est,
    West,
    North,
    South,
}

#[derive(Debug, Clone)]
pub struct EntityModel {
    name: String,
    uuid: String,
    species: Species,
    offset: MapCoord,
    step: f64,
    map: MapCoord,
    world: WorldCoord,
    state: Animations,
    destination: Option<MapCoord>,
    path: Vec<MapCoord>,
    orientation: Orientation,
    next_map: Option<Orientation>,
    frame: u8,
    timer: u128,
}

// Needed to manipulate EntityModel attributes using IEntity abstraction
pub trait IEntity: Send {
    fn get_name(&self) -> &String;

    fn get_species(&self) -> &Species;

    fn get_world(&self) -> WorldCoord;
    fn set_world(&mut self, world: WorldCoord);

    fn get_uuid(&self) -> &String;

    fn get_map(&self) -> MapCoord;
    fn set_map(&mut self, map: MapCoord);

    fn get_timer(&self) -> u128;
    fn set_timer(&mut self, timer: u128);

    fn get_frame(&self) -> usize;
    fn set_frame(&mut self, frame: u8);

    fn get_assets(&self) -> &EntityAssets;

    fn set_path(&mut self, path: Vec<MapCoord>, next_map: Option<Orientation>);

    fn consume_destination(&mut self) -> Option<MapCoord>;
    fn set_destination(&mut self, destination: MapCoord);

    fn update(&mut self, delta_ts: u128, world: &World) -> Option<LocationType>;
    fn get_real_pos(&self) -> MapCoord;

    fn get_orientation(&self) -> &Orientation;
}

impl EntityModel {
    pub fn new(name: String, uuid: String, race: Species) -> Self {
        EntityModel {
            name,
            uuid,
            species: race,
            offset: MapCoord::default(),
            world: WorldCoord::default(),
            map: MapCoord::default(),
            state: Animations::default(),
            destination: None,
            path: Vec::new(),
            orientation: Orientation::Est,
            next_map: None,
            timer: 0,
            frame: 0,
            step: 0.0,
        }
    }

    fn get_assets(&self) -> &EntityAssets {
        match self.species {
            Species::Crabedoeuf => match self.state {
                Animations::Idle => &EntityAssets::Crabedoeuf(Animations::Idle),
                Animations::Run => &EntityAssets::Crabedoeuf(Animations::Run),
            },
            Species::Bouftou => match self.state {
                Animations::Idle => &EntityAssets::Bouftou(Animations::Idle),
                Animations::Run => &EntityAssets::Bouftou(Animations::Run),
            },
            Species::Warrior => match self.state {
                Animations::Idle => &EntityAssets::Warrior(Animations::Idle),
                Animations::Run => &EntityAssets::Warrior(Animations::Run),
            },
            Species::Mage => match self.state {
                Animations::Idle => &EntityAssets::Mage(Animations::Idle),
                Animations::Run => &EntityAssets::Mage(Animations::Run),
            },
        }
    }

    fn set_state(&mut self, state: Animations) {
        if self.state != state {
            self.state = state;
            self.frame = 0;
            self.timer = 0;
        }
    }
}

impl From<Family> for Species {
    fn from(value: Family) -> Self {
        match value {
            Family::Species(species) => match species {
                val if val == RpcSpecies::Bouftou as i32 => Species::Bouftou,
                val if val == RpcSpecies::Crabedoeuf as i32 => Species::Crabedoeuf,
                _ => todo!(),
            },
            Family::Class(class) => match class {
                val if val == RpcClasses::Warrior as i32 => Species::Warrior,
                val if val == RpcClasses::Mage as i32 => Species::Mage,
                _ => todo!(),
            },
        }
    }
}

impl IEntity for EntityModel {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn set_world(&mut self, world: WorldCoord) {
        self.world = world;
    }

    fn get_world(&self) -> WorldCoord {
        self.world
    }

    fn set_map(&mut self, map: MapCoord) {
        self.map = map.limit();
    }

    fn get_map(&self) -> MapCoord {
        self.map
    }

    fn get_timer(&self) -> u128 {
        self.timer
    }

    fn set_timer(&mut self, timer: u128) {
        self.timer = timer;
    }

    fn get_frame(&self) -> usize {
        self.frame as usize
    }

    fn set_frame(&mut self, frame: u8) {
        self.frame = frame;
    }

    fn get_assets(&self) -> &EntityAssets {
        self.get_assets()
    }

    fn set_path(&mut self, path: Vec<MapCoord>, next_map: Option<Orientation>) {
        self.path = path;
        self.next_map = next_map;
    }

    fn update(&mut self, delta_ts: u128, world: &World) -> Option<LocationType> {
        let mut new_pos: Option<LocationType> = None;
        if self.path.is_empty() {
            self.set_state(Animations::Idle);
            self.world = match self.next_map.take() {
                Some(Orientation::Est) => {
                    if let Some(new_map) = world.get_east_map(&self.world) {
                        self.map.x = 0;
                        new_pos = Some(LocationType::NewWorld);
                        new_map.0
                    } else {
                        self.world
                    }
                }
                Some(Orientation::West) => {
                    if let Some(new_map) = world.get_west_map(&self.world) {
                        self.map.x = TILEMAP_WIDTH as i64 - 1;
                        new_pos = Some(LocationType::NewWorld);
                        new_map.0
                    } else {
                        self.world
                    }
                }
                _ => self.world,
            };
        } else {
            self.set_state(Animations::Run);
            let target = *self.path.last().unwrap(); //TODO: remove unwrap

            // Compute unit vector (x and y could be 1, -1 or 0)
            let mut direction = target - self.map;
            if self.step > 1.0 {
                self.map = self.path.pop().unwrap();
                self.offset.x = 0;
                self.offset.y = 0;
                self.step = 0.0;
                new_pos = Some(LocationType::Update);
            } else {
                direction *= 64.0;
                self.offset = MapCoord {
                    x: lerp(0.0, direction.x as f64, self.step) as i64,
                    y: lerp(0.0, direction.y as f64, self.step) as i64,
                };
                self.step += delta_ts as f64 * ENTITY_RUN_STEP_DURATION;
            }

            if let Some(new_orientation) = match direction {
                dir if dir.x > 0 => Some(Orientation::Est),
                dir if dir.x < 0 => Some(Orientation::West),
                dir if dir.y > 0 => Some(Orientation::South),
                dir if dir.y < 0 => Some(Orientation::North),
                _ => None,
            } {
                self.orientation = new_orientation;
            }
        }
        new_pos
    }

    fn get_real_pos(&self) -> MapCoord {
        MapCoord {
            x: self.map.x * 64 + self.offset.x,
            y: self.map.y * 64 + self.offset.y,
        }
    }

    fn get_orientation(&self) -> &Orientation {
        &self.orientation
    }

    fn get_uuid(&self) -> &String {
        &self.uuid
    }

    fn consume_destination(&mut self) -> Option<MapCoord> {
        self.destination.take()
    }

    fn set_destination(&mut self, destination: MapCoord) {
        self.destination = Some(destination);
    }

    fn get_species(&self) -> &Species {
        &self.species
    }
}

fn lerp(v0: f64, v1: f64, t: f64) -> f64 {
    (1.0 - t) * v0 + t * v1
}
