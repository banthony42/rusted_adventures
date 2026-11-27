use crate::entities::traits::PlayerClasses;

#[derive(Debug, Clone)]
pub struct Mage {}

impl Mage {
    pub fn fireball(&self) {
        println!("Cast fireball");
    }

    pub fn ice_trap(&self) {
        println!("Cast ice trap");
    }
}

impl PlayerClasses for Mage {
    fn new() -> Self {
        Self {}
    }
}
