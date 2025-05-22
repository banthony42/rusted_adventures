use crate::{
    import::assets::{Animations, EntityAssets},
    world::Coord_tmp,
};

pub struct EntityModel {
    name: String,
    race: Bestiary,
    map: Coord_tmp,
    world: Coord_tmp,
    state: Animations,
    path: Option<Vec<Coord_tmp>>,
    frame: u8,
    timer: u128,
}

// Needed to manipulate EntityModel attributes using IEntity abstraction
pub trait IEntity {
    fn get_world(&self) -> Coord_tmp;
    fn set_world(&mut self, world: Coord_tmp);

    fn get_map(&self) -> Coord_tmp;
    fn set_map(&mut self, map: Coord_tmp);

    fn get_timer(&self) -> u128;
    fn set_timer(&mut self, timer: u128);

    fn get_frame(&self) -> usize;
    fn set_frame(&mut self, frame: u8);

    fn get_assets(&self) -> &EntityAssets;

    fn set_path(&mut self, path: Option<Vec<Coord_tmp>>);
    fn get_path(&self) -> Option<Vec<Coord_tmp>>;
}

impl EntityModel {
    pub fn new(name: String, race: Bestiary) -> Self {
        EntityModel {
            name,
            race,
            world: Coord_tmp::default(),
            map: Coord_tmp::default(),
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
    fn set_world(&mut self, world: Coord_tmp) {
        self.world = world;
    }

    fn set_map(&mut self, map: Coord_tmp) {
        self.map = map;
    }

    fn get_world(&self) -> Coord_tmp {
        self.world
    }

    fn get_map(&self) -> Coord_tmp {
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

    fn get_path(&self) -> Option<Vec<Coord_tmp>> {
        self.path.clone()
    }

    fn set_path(&mut self, path: Option<Vec<Coord_tmp>>) {
        self.path = path;
    }
}
