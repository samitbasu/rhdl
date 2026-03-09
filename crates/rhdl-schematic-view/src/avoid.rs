use std::collections::{BTreeSet, VecDeque};

use egui::ahash::HashMap;
use rhdl_core::circuit::schematic::{Coordinate, Schematic};

fn port_positions(schematic: &Schematic) -> impl Iterator<Item = (Coordinate, Coordinate)> {
    schematic
        .inputs
        .iter()
        .flatten()
        .map(|port| schematic.port_position_by_id(port.id))
        .chain(
            schematic
                .outputs
                .iter()
                .map(|port| schematic.port_position_by_id(port.id)),
        )
}

fn x_points_of_interest(schematic: &[Schematic]) -> impl Iterator<Item = Coordinate> {
    schematic.iter().flat_map(|x| {
        port_positions(x)
            .map(|(x, _)| x)
            .chain([x.origin.0, x.origin.0 + x.size.0])
    })
}

fn y_points_of_interest(schematic: &[Schematic]) -> impl Iterator<Item = Coordinate> {
    schematic.iter().flat_map(|x| {
        port_positions(x)
            .map(|(_, y)| y)
            .chain([x.origin.1, x.origin.1 + x.size.1])
    })
}

pub struct Grid {
    pub x: Vec<Coordinate>,
    pub y: Vec<Coordinate>,
    pub rev_x: HashMap<Coordinate, usize>,
    pub rev_y: HashMap<Coordinate, usize>,
    pub occupied: Vec<Vec<bool>>,
}

pub fn grid(schematic: &Schematic) -> Grid {
    let x = x_points_of_interest(&schematic.inner).collect::<BTreeSet<_>>();
    let y = y_points_of_interest(&schematic.inner).collect::<BTreeSet<_>>();
    let numx = x.len();
    let numy = y.len();
    let rev_x: HashMap<Coordinate, usize> = x.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let rev_y: HashMap<Coordinate, usize> = y.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let mut occupied = vec![vec![false; numx]; numy];
    for child in &schematic.inner {
        let x1 = rev_x[&child.origin.0];
        let x2 = rev_x[&(child.origin.0 + child.size.0)];
        let y1 = rev_y[&child.origin.1];
        let y2 = rev_y[&(child.origin.1 + child.size.1)];
        (y1..=y2).for_each(|y| {
            occupied[y][x1..=x2].fill(true);
        });
    }

    Grid {
        x: x.into_iter().collect(),
        y: y.into_iter().collect(),
        rev_x,
        rev_y,
        occupied,
    }
}

impl Grid {
    pub fn route(
        &self,
        from: (Coordinate, Coordinate),
        to: (Coordinate, Coordinate),
    ) -> Vec<(Coordinate, Coordinate)> {
        // We are going to use Hadlock's algorithm to route from the start to the end.
        let mut queue = VecDeque::new();
        let mut cost_map = vec![vec![None; self.x.len()]; self.y.len()];
        let start = (self.rev_x[&from.0], self.rev_y[&from.1]);
        let end = (self.rev_x[&to.0], self.rev_y[&to.1]);
        queue.push_back((start, 0));
        cost_map[start.1][start.0] = Some(0);
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        while let Some((current, cost)) = queue.pop_front() {
            if current == end {
                break;
            }
            let current_distance = (current.0 as isize - end.0 as isize).abs()
                + (current.1 as isize - end.1 as isize).abs();
            for (dx, dy) in directions {
                let next = (current.0 as isize + dx, current.1 as isize + dy);
                if next.0 < 0
                    || next.0 >= self.x.len() as isize
                    || next.1 < 0
                    || next.1 >= self.y.len() as isize
                {
                    continue;
                }
                let next = (next.0 as usize, next.1 as usize);
                // If the next position is occupied, then continue
                if self.occupied[next.1][next.0] {
                    continue;
                }
                // Calculate the new cost for this move
                let next_distance = (next.0 as isize - end.0 as isize).abs()
                    + (next.1 as isize - end.1 as isize).abs();
                let new_cost = if next_distance > current_distance {
                    cost + 1
                } else {
                    cost
                };
                // If the next position is already labelled with a better or equal cost, skip it
                if let Some(existing_cost) = cost_map[next.1][next.0]
                    && existing_cost <= new_cost
                {
                    continue;
                }
                // Update the cost and add to the queue
                cost_map[next.1][next.0] = Some(new_cost);
                if next_distance > current_distance {
                    queue.push_back((next, new_cost));
                } else {
                    queue.push_front((next, new_cost));
                }
            }
        }
        // Print out the cost map
        for (row_ndx, row) in cost_map.iter().enumerate() {
            for (col_ndx, cell) in row.iter().enumerate() {
                if (col_ndx, row_ndx) == start {
                    print!("  S ");
                    continue;
                }
                if (col_ndx, row_ndx) == end {
                    print!("  T ");
                    continue;
                }
                if self.occupied[row_ndx][col_ndx] {
                    print!("  # ");
                    continue;
                }
                match cell {
                    Some(cost) => print!("{:3} ", cost),
                    None => print!("  . "),
                }
            }
            println!();
        }
        todo!("Reconstruct the path from the cost map");
        let mut path = Vec::new();
        let mut current = end;
        while current != start {
            path.push((self.x[current.0], self.y[current.1]));
            let mut next = current;
            for (dx, dy) in directions {
                let candidate = (current.0 as isize + dx, current.1 as isize + dy);
                if candidate.0 < 0
                    || candidate.0 >= self.x.len() as isize
                    || candidate.1 < 0
                    || candidate.1 >= self.y.len() as isize
                {
                    continue;
                }
                let candidate = (candidate.0 as usize, candidate.1 as usize);
                if let Some(candidate_cost) = cost_map[candidate.1][candidate.0]
                    && candidate_cost < cost_map[current.1][current.0].unwrap()
                {
                    next = candidate;
                }
            }
            current = next;
        }
        path.push((self.x[start.0], self.y[start.1]));
        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid() {
        const GRID: [[u8; 14]; 15] = [
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        let x_coordinates = (0..14)
            .map(|n| Coordinate::from(n as f32))
            .collect::<Vec<_>>();
        let y_coordinates = (0..15)
            .map(|n| Coordinate::from(n as f32))
            .collect::<Vec<_>>();
        let rev_x = x_coordinates
            .iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect::<HashMap<_, _>>();
        let rev_y = y_coordinates
            .iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect::<HashMap<_, _>>();
        let occupied = GRID
            .iter()
            .map(|row| row.iter().map(|&cell| cell == 1).collect())
            .collect::<Vec<_>>();
        let grid = Grid {
            x: x_coordinates,
            y: y_coordinates,
            rev_x,
            rev_y,
            occupied,
        }; // col 6, row 10
        let from = (Coordinate::from(6.0), Coordinate::from(9.0));
        // col 10, row 3
        let to = (Coordinate::from(10.0), Coordinate::from(3.0));
        let path = grid.route(from, to);
    }
}
