use crate::{
    constants::{TILEMAP_HEIGHT, TILEMAP_WIDTH},
    world::Coord_tmp,
};

pub struct PathFinder {
    start: Coord_tmp,
    destination: Coord_tmp,
    map: Vec<Vec<bool>>,
}

struct Cell {
    cell: Coord_tmp,
    priority: u32,
}

impl PathFinder {
    pub fn new(start: Coord_tmp, destination: Coord_tmp, map: Vec<Vec<bool>>) -> Self {
        PathFinder {
            start,
            destination,
            map,
        }
    }

    fn manhattan_distance_from(&self, cell: &Coord_tmp) -> u32 {
        ((cell.x - self.destination.x).abs() + (cell.y - self.destination.y).abs()).max(0) as u32
    }

    pub fn compute(&self) -> Option<Vec<Coord_tmp>> {
        let map_size = (TILEMAP_HEIGHT * TILEMAP_WIDTH) as usize;
        let mut queue: Vec<Cell> = Vec::new();
        let mut cost: Vec<Option<u32>> = vec![None; map_size];
        let mut predecessors: Vec<Option<Coord_tmp>> = vec![None; map_size];

        cost[self.start.flat_position()] = Some(0);
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

                let current_cost = cost[current.cell.flat_position()].unwrap();

                let neighbours = vec![
                    (current.cell + Coord_tmp { x: 1, y: 0 })
                        .bounds_x_with(TILEMAP_WIDTH as i32 - 1, 0)
                        .bounds_y_with(TILEMAP_HEIGHT as i32 - 1, 0), // Right neighb'
                    (current.cell + Coord_tmp { x: 0, y: 1 })
                        .bounds_x_with(TILEMAP_WIDTH as i32 - 1, 0)
                        .bounds_y_with(TILEMAP_HEIGHT as i32 - 1, 0), // Below neighb'
                    (current.cell + Coord_tmp { x: -1, y: 0 })
                        .bounds_x_with(TILEMAP_WIDTH as i32 - 1, 0)
                        .bounds_y_with(TILEMAP_HEIGHT as i32 - 1, 0), // Above neighb'
                    (current.cell + Coord_tmp { x: 0, y: -1 })
                        .bounds_x_with(TILEMAP_WIDTH as i32 - 1, 0)
                        .bounds_y_with(TILEMAP_HEIGHT as i32 - 1, 0), // Left neighb'
                ];

                let _: Vec<_> = neighbours
                    .iter()
                    .filter(|nb| self.map[nb.y as usize][nb.x as usize] == false) // Filter out collider cell
                    .map(|nb| {
                        let nb_cost = current_cost + 1;
                        // If we already encounter this cell
                        if let Some(last_nb_cost) = cost[nb.flat_position()] {
                            // Skip this new cost if the last one is better
                            if nb_cost >= last_nb_cost {
                                return;
                            }
                        }

                        cost[nb.flat_position()] = Some(nb_cost);
                        predecessors[nb.flat_position()] = Some(current.cell);
                        queue.push(Cell {
                            priority: nb_cost + self.manhattan_distance_from(&nb),
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
            if let Some(parent) = predecessors[cursor.flat_position()] {
                path.push(parent.clone());
                cursor = parent;
            } else {
                break;
            }
        }

        Some(path)
    }
}
