use crate::entities::{
    classes::{mage::Mage, warrior::Warrior},
    model::EntityModel,
    traits::{GameEntity, PlayerClasses},
};

#[derive(Debug, Clone)]
pub struct Player<T: PlayerClasses> {
    pub entity: EntityModel,
    pub classes: T,
}

impl<T: PlayerClasses> Player<T> {
    pub fn new(entity: EntityModel) -> Self {
        Self {
            entity,
            classes: T::new(),
        }
    }
}

impl<T: PlayerClasses> GameEntity for Player<T> {
    fn model(&self) -> &EntityModel {
        &self.entity
    }

    fn model_mut(&mut self) -> &mut EntityModel {
        &mut self.entity
    }
}

#[derive(Debug, Clone)]
pub enum PlayerType {
    Warrior(Player<Warrior>),
    Mage(Player<Mage>),
}

impl GameEntity for PlayerType {
    // We can add a get_classes here
    // but it mean add a get_species for monster
    // think to unify this

    fn model(&self) -> &EntityModel {
        match self {
            PlayerType::Warrior(w) => w.model(),
            PlayerType::Mage(m) => m.model(),
        }
    }

    fn model_mut(&mut self) -> &mut EntityModel {
        match self {
            PlayerType::Warrior(w) => w.model_mut(),
            PlayerType::Mage(m) => m.model_mut(),
        }
    }
}
