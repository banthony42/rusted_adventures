use crate::entities::traits::PlayerClasses;

#[derive(Debug, Clone)]
pub struct Warrior {}

impl Warrior {
    pub fn absorption(&self) {
        println!("Cast absorption");
    }

    pub fn charge(&self) {
        println!("Cast charge");
    }
}

impl PlayerClasses for Warrior {
    fn new() -> Self {
        Self {}
    }
}
