use common::{world::ColliderMap, CellCoord};

pub mod astar;

pub trait PathFindingStrategy {
    fn new() -> Self;
    fn compute(
        &mut self,
        start: CellCoord,
        destination: CellCoord,
        colllider_map: &ColliderMap,
    ) -> bool;
    fn get_path(&self) -> Vec<CellCoord>;
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
        start: CellCoord,
        destination: CellCoord,
        collider_map: &ColliderMap,
    ) -> bool {
        self.strategy.compute(start, destination, collider_map)
    }

    pub fn get_path(&self) -> Vec<CellCoord> {
        self.strategy.get_path()
    }
}
