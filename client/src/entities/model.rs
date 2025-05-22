use crate::{
    import::assets::{Animations, EntityAssets},
    world::{MapCoord, WorldCoord},
};

pub struct EntityModel {
    name: String,
    race: Bestiary,
    map: MapCoord,
    world: WorldCoord,
    state: Animations,
    path: Option<Vec<MapCoord>>,
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

    fn set_path(&mut self, path: Option<Vec<MapCoord>>);
    fn get_path(&self) -> Option<Vec<MapCoord>>;
}

impl EntityModel {
    pub fn new(name: String, race: Bestiary) -> Self {
        EntityModel {
            name,
            race,
            world: WorldCoord::default(),
            map: MapCoord::default(),
            state: Animations::default(),
            path: None,
            timer: 0,
            frame: 0,
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

    fn get_path(&self) -> Option<Vec<MapCoord>> {
        self.path.clone()
    }

    fn set_path(&mut self, path: Option<Vec<MapCoord>>) {
        self.path = path;
    }
}
