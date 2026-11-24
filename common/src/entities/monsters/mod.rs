use crate::entities::{
    model::EntityModel,
    monsters::{bouftou::Bouftou, crabedoeuf::Crabedoeuf},
    traits::{GameEntity, MonsterSpecies},
};

mod bouftou;
mod crabedoeuf;

#[derive(Debug, Clone)]
pub struct Monster<T: MonsterSpecies> {
    pub entity: EntityModel,
    pub species: T,
}

impl<T: MonsterSpecies> Monster<T> {
    pub fn new(entity: EntityModel) -> Self {
        Self {
            entity,
            species: T::new(),
        }
    }
}

impl<T: MonsterSpecies> GameEntity for Monster<T> {
    fn model(&self) -> &EntityModel {
        &self.entity
    }

    fn model_mut(&mut self) -> &mut EntityModel {
        &mut self.entity
    }
}

#[derive(Debug, Clone)]
pub enum MonsterType {
    Bouftou(Monster<Bouftou>),
    Crabedoeuf(Monster<Crabedoeuf>),
}

impl GameEntity for MonsterType {
    fn model(&self) -> &EntityModel {
        match self {
            MonsterType::Bouftou(b) => b.model(),
            MonsterType::Crabedoeuf(c) => c.model(),
        }
    }

    fn model_mut(&mut self) -> &mut EntityModel {
        match self {
            MonsterType::Bouftou(b) => b.model_mut(),
            MonsterType::Crabedoeuf(c) => c.model_mut(),
        }
    }
}
