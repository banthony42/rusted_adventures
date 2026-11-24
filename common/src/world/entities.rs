pub const ENTITY_RUN_SPEED: f64 = 225.0;
pub const ENTITY_RUN_STEP_DURATION: f64 = 1.0 / ENTITY_RUN_SPEED;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum Species {
    Warrior,
    Mage,
    Bouftou,
    Crabedoeuf,
}
