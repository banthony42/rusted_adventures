use crate::{
    import::assets::{Animations, EntityAssets},
    world::{MapCoord, WorldCoord},
};

const ENTITY_RUN_SPEED: f64 = 350.0;

pub enum Orientation {
    Est,
    West,
    North,
    South,
}

pub struct EntityModel {
    name: String,
    race: Bestiary,
    offset: MapCoord,
    step: f64,
    map: MapCoord,
    world: WorldCoord,
    state: Animations,
    path: Vec<MapCoord>,
    frame: u8,
    timer: u128,
}

// Needed to manipulate EntityModel attributes using IEntity abstraction
pub trait IEntity {
    fn get_world(&self) -> WorldCoord;
    fn set_world(&mut self, world: WorldCoord);

    fn get_map(&self) -> MapCoord;
    fn set_map(&mut self, map: MapCoord);

    fn get_timer(&self) -> u128;
    fn set_timer(&mut self, timer: u128);

    fn get_frame(&self) -> usize;
    fn set_frame(&mut self, frame: u8);

    fn get_assets(&self) -> &EntityAssets;

    fn set_path(&mut self, path: Vec<MapCoord>);

    fn update(&mut self, delta_ts: u128);
    fn get_real_pos(&self) -> MapCoord;

    fn get_orientation(&self) -> Orientation;
}

impl EntityModel {
    pub fn new(name: String, race: Bestiary) -> Self {
        EntityModel {
            name,
            race,
            offset: MapCoord::default(),
            world: WorldCoord::default(),
            map: MapCoord::default(),
            state: Animations::default(),
            path: Vec::new(),
            timer: 0,
            frame: 0,
            step: 0.0,
        }
    }

    fn get_assets(&self) -> &EntityAssets {
        match self.race {
            Bestiary::Human => match self.state {
                Animations::Idle => &EntityAssets::Character(Animations::Idle),
                Animations::Run => &EntityAssets::Character(Animations::Run),
            },
            Bestiary::Bouftou => match self.state {
                Animations::Idle => &EntityAssets::Bouftou(Animations::Idle),
                Animations::Run => &EntityAssets::Bouftou(Animations::Run),
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

pub enum Bestiary {
    Human,
    Bouftou,
}

impl IEntity for EntityModel {
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

    fn set_path(&mut self, path: Vec<MapCoord>) {
        self.path = path;
    }

    fn update(&mut self, delta_ts: u128) {
        if self.path.is_empty() {
            self.set_state(Animations::Idle);
        } else {
            self.set_state(Animations::Run);
            let step_duration = 1.0 / 150.0; // TODO: constant or self.var
            let target = *self.path.last().unwrap(); //TODO: remove unwrap

            // Compute unit vector (x and y could be 1, -1 or 0)
            let mut direction = target - self.map;
            if self.step > 1.0 {
                self.map = self.path.pop().unwrap();
                self.offset.x = 0;
                self.offset.y = 0;
                self.step = 0.0;
            } else {
                direction *= 64.0;
                self.offset = MapCoord {
                    x: lerp(0.0, direction.x as f64, self.step) as i64,
                    y: lerp(0.0, direction.y as f64, self.step) as i64,
                };
                self.step += delta_ts as f64 * step_duration;
            }
        }
    }

    fn get_real_pos(&self) -> MapCoord {
        MapCoord {
            x: self.map.x * 64 + self.offset.x,
            y: self.map.y * 64 + self.offset.y,
        }
    }

    fn get_orientation(&self) -> Orientation {
        // TODO: The orientation seems to switch between Est / West
        // On horizontal path
        if self.offset.x.is_negative() {
            return Orientation::West;
        }
        Orientation::Est
    }
}

fn lerp(v0: f64, v1: f64, t: f64) -> f64 {
    (1.0 - t) * v0 + t * v1
}
