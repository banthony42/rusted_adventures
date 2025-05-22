use crate::world::{MapCoord, WorldCoord};

use super::model::{Bestiary, EntityModel, IEntity};

pub struct EntityClient {}

impl EntityClient {
    pub fn new(login: String, token: String) -> Self {
        EntityClient {}
    }

    pub fn fetch_player(&self) -> Box<dyn IEntity> {
        // Call gRPC to get player data ...
        let mut player = Box::new(EntityModel::new("New-Sulfurel".into(), Bestiary::Human));
        player.set_world(WorldCoord { x: 1, y: 0 });
        player.set_map(MapCoord { x: 8, y: 8 });

        player
    }

    pub fn fetch_entities(&self, world: WorldCoord) -> Vec<Box<dyn IEntity>> {
        // Call gRPC to get entities data of the given world map

        let mut entity = Box::new(EntityModel::new("New-Bouftou".into(), Bestiary::Bouftou));
        entity.set_world(world);
        entity.set_map(MapCoord { x: 7, y: 5 });

        vec![entity]
    }
}
