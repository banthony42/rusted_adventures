pub mod astar;

use crate::world::MapCoord;

pub trait PathFindingStrategy {
    fn new() -> Self;
    fn compute(&mut self, start: MapCoord, destination: MapCoord, map: &Vec<Vec<bool>>) -> bool;
    fn get_path(&self) -> Vec<MapCoord>;
}

pub struct PathFinder<T>
where
    T: PathFindingStrategy,
{
    strategy: T,
}

impl<T> PathFinder<T>
where
    T: PathFindingStrategy,
{
    pub fn new(strategy: T) -> Self {
        PathFinder { strategy }
    }

    pub fn compute(
        &mut self,
        start: MapCoord,
        destination: MapCoord,
        map: &Vec<Vec<bool>>,
    ) -> bool {
        self.strategy.compute(start, destination, map)
    }

    pub fn get_path(&self) -> Vec<MapCoord> {
        self.strategy.get_path()
    }
}
