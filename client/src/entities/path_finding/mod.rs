use crate::{constants::TILEMAP_LINEAR_SIZE, world::MapCoord};

pub struct PathFinder {
    start: MapCoord,
    destination: MapCoord,
    map: Vec<Vec<bool>>,
}

struct Cell {
    cell: MapCoord,
    priority: u32,
}

impl PathFinder {
    pub fn new(start: MapCoord, destination: MapCoord, map: Vec<Vec<bool>>) -> Self {
        PathFinder {
            start,
            destination,
            map,
        }
    }

    fn manhattan_distance_from(&self, cell: &MapCoord) -> u32 {
        ((cell.x - self.destination.x).abs() + (cell.y - self.destination.y).abs()).max(0) as u32
    }

    pub fn compute(&self) -> Option<Vec<MapCoord>> {
        let mut queue: Vec<Cell> = Vec::new();
        let mut cost: Vec<Option<u32>> = vec![None; TILEMAP_LINEAR_SIZE];
        let mut predecessors: Vec<Option<MapCoord>> = vec![None; TILEMAP_LINEAR_SIZE];

        cost[self.start.linear_index()] = Some(0);
        queue.push(Cell {
            cell: self.start,
            priority: 0,
        });

        loop {
            if let Some(current) = queue.pop() {
                if current.cell.eq(&self.destination) {
                    println!("PathFinding: path found!");
                    break;
                }

                let current_cost = cost[current.cell.linear_index()].unwrap();

                let neighbours = vec![
                    (current.cell + MapCoord { x: 1, y: 0 }).limit(), // Right neighb'
                    (current.cell + MapCoord { x: 0, y: 1 }).limit(), // Below neighb'
                    (current.cell + MapCoord { x: -1, y: 0 }).limit(), // Above neighb'
                    (current.cell + MapCoord { x: 0, y: -1 }).limit(), // Left neighb'
                ];

                let _: Vec<_> = neighbours
                    .iter()
                    .filter(|nb| self.map[nb.y as usize][nb.x as usize] == false) // Filter out collider cell
                    .map(|nb| {
                        let neighbour_index = nb.linear_index();
                        let current_neighbour_cost = current_cost + 1;

                        // If we already encounter this cell
                        if let Some(last_cost) = cost[neighbour_index] {
                            // Skip this new cost if the last one is better
                            if current_neighbour_cost >= last_cost {
                                return;
                            }
                        }

                        cost[neighbour_index] = Some(current_neighbour_cost);
                        predecessors[neighbour_index] = Some(current.cell);
                        queue.push(Cell {
                            priority: current_neighbour_cost + self.manhattan_distance_from(&nb),
                            cell: *nb,
                        });
                        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
                    })
                    .collect();
            } else {
                println!("PathFinding: no solutions found!");
                return None;
            }
        }

        // Path found, backtrack on predecessor and build the path
        let mut path = Vec::new();
        let mut cursor = self.destination;
        path.push(self.destination);

        loop {
            if let Some(parent) = predecessors[cursor.linear_index()] {
                path.push(parent.clone());
                cursor = parent;
            } else {
                break;
            }
        }

        Some(path)
    }
}
