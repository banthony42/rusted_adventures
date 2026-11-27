use crate::entities::model::EntityModel;

pub trait GameEntity: Send {
    fn model(&self) -> &EntityModel;
    fn model_mut(&mut self) -> &mut EntityModel;
}

pub trait PlayerClasses: Send {
    fn new() -> Self;
}

pub trait MonsterSpecies: Send {
    fn new() -> Self;
}
