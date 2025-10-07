use common::CellCoord;

use common::constants::TILEMAP_LINEAR_SIZE;

use super::PathFindingStrategy;

struct Cell {
    cell: CellCoord,
    priority: u32,
}

struct PriorityQueue(Vec<Cell>);

impl PriorityQueue {
    /// Store the cell according to its priority
    fn priority_push(&mut self, cell: Cell) {
        self.0.push(cell);
        // The last element is the cell with the lowest priority.
        self.0.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get the lowest priority Cell
    fn priority_pop(&mut self) -> Option<Cell> {
        self.0.pop()
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}
const RIGHT_NEIGHBOR: CellCoord = CellCoord { x: 1, y: 0 };
const BOTTOM_NEIGHBOR: CellCoord = CellCoord { x: 0, y: 1 };
const LEFT_NEIGHBOR: CellCoord = CellCoord { x: -1, y: 0 };
const TOP_NEIGHBOR: CellCoord = CellCoord { x: 0, y: -1 };

pub struct AStar {
    cells: PriorityQueue,
    cost: Vec<Option<u32>>,
    predecessors: Vec<Option<CellCoord>>,
    path_found: Vec<CellCoord>,
}

impl AStar {
    fn manhattan_distance_from(&self, cell: &CellCoord, destination: CellCoord) -> u32 {
        ((cell.x - destination.x).abs() + (cell.y - destination.y).abs()).max(0) as u32
    }

    fn clear(&mut self) {
        self.cells.clear();
        self.cost.fill(None);
        self.predecessors.fill(None);
        self.path_found.clear();
    }

    fn build_path(&mut self, destination: CellCoord) {
        let mut cursor = destination;
        self.path_found.push(destination);

        while let Some(parent) = self.predecessors[cursor.linear_index()] {
            self.path_found.push(parent.clone());
            cursor = parent;
        }
        self.path_found.pop();
    }

    fn valid_neighbors(&self, map_coord: CellCoord, map: &Vec<Vec<bool>>) -> Vec<CellCoord> {
        vec![
            (map_coord + RIGHT_NEIGHBOR).limit(),
            (map_coord + LEFT_NEIGHBOR).limit(),
            (map_coord + TOP_NEIGHBOR).limit(),
            (map_coord + BOTTOM_NEIGHBOR).limit(),
        ]
        .into_iter()
        .filter(|nb| map[nb.y as usize][nb.x as usize] == false)
        .collect()
    }
}

impl PathFindingStrategy for AStar {
    fn new() -> Self {
        AStar {
            cells: PriorityQueue(Vec::new()),
            cost: vec![None; TILEMAP_LINEAR_SIZE],
            predecessors: vec![None; TILEMAP_LINEAR_SIZE],
            path_found: Vec::new(),
        }
    }

    fn compute(
        &mut self,
        start: CellCoord,
        destination: CellCoord,
        collider_map: &Vec<Vec<bool>>,
    ) -> bool {
        self.clear();
        // Initialize AStar algorithm
        self.cost[start.linear_index()] = Some(0);
        self.cells.priority_push(Cell {
            cell: start,
            priority: 0,
        });

        while let Some(current) = self.cells.priority_pop() {
            if current.cell.eq(&destination) {
                self.build_path(destination);
                return true;
            }

            self.valid_neighbors(current.cell, &collider_map)
                .iter()
                .for_each(|nb| {
                    let current_cost = self.cost[current.cell.linear_index()]
                        .expect("cost / predecessors / queue should be updated at the same time.");
                    let neighbour_index = nb.linear_index();
                    let current_neighbour_cost = current_cost + 1;

                    // If we already encounter this cell
                    if let Some(last_cost) = self.cost[neighbour_index] {
                        // Skip this new cost if the last one is better
                        if current_neighbour_cost >= last_cost {
                            return;
                        }
                    }
                    self.cost[neighbour_index] = Some(current_neighbour_cost);
                    self.predecessors[neighbour_index] = Some(current.cell);
                    self.cells.priority_push(Cell {
                        priority: current_neighbour_cost
                            + self.manhattan_distance_from(&nb, destination),
                        cell: *nb,
                    });
                });
        }
        println!("PathFinding: no solutions found!");
        false
    }

    fn get_path(&self) -> Vec<CellCoord> {
        self.path_found.clone()
    }
}
