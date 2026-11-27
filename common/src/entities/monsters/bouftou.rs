use crate::entities::traits::MonsterSpecies;

#[derive(Debug, Clone)]
pub struct Bouftou {}

impl MonsterSpecies for Bouftou {
    fn new() -> Self {
        Self {}
    }
}
