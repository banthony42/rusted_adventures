use crate::entities::traits::MonsterSpecies;

#[derive(Debug, Clone)]
pub struct Crabedoeuf {}

impl MonsterSpecies for Crabedoeuf {
    fn new() -> Self {
        Self {}
    }
}
