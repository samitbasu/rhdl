use std::collections::{BTreeSet, VecDeque};

use egui::ahash::HashMap;
use rhdl_core::circuit::schematic::{Coordinate, Schematic};

fn port_positions(schematic: &Schematic) -> impl Iterator<Item = (Coordinate, Coordinate)> {
    schematic
        .inputs
        .iter()
        .flatten()
        .flat_map(|port| schematic.port_position_by_id(port.id))
        .chain(
            schematic
                .outputs
                .iter()
                .flat_map(|port| schematic.port_position_by_id(port.id)),
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

pub fn grid(schematic: &Schematic) -> Option<Grid> {
    let x = x_points_of_interest(&schematic.inner)
        .map(|x| x + schematic.origin.0)
        .collect::<BTreeSet<_>>();
    let y = y_points_of_interest(&schematic.inner)
        .map(|y| y + schematic.origin.1)
        .collect::<BTreeSet<_>>();
    eprintln!("x points of interest: {x:?}");
    eprintln!("y points of interest: {y:?}");
    /*     // Get the raw coordinates for min and max with bounds
       const GRID_SPACING: i32 = 100;
       let x_min = (x.first()?.raw() - 2000) / GRID_SPACING * GRID_SPACING;
       let x_max = (x.last()?.raw() + 2000 + GRID_SPACING - 1) / GRID_SPACING * GRID_SPACING;
       let y_min = (y.first()?.raw() - 2000) / GRID_SPACING * GRID_SPACING;
       let y_max = (y.last()?.raw() + 2000 + GRID_SPACING - 1) / GRID_SPACING * GRID_SPACING;
       // We want to inject a grid of points that are 50 units apart (which is a spacing of 500 in the x_min and x_max)
       x.extend(
           (x_min..=x_max)
               .step_by(GRID_SPACING as usize)
               .map(Coordinate::from_raw),
       );
       y.extend(
           (y_min..=y_max)
               .step_by(GRID_SPACING as usize)
               .map(Coordinate::from_raw),
       );
    */
    let numx = x.len();
    let numy = y.len();
    let rev_x: HashMap<Coordinate, usize> = x.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let rev_y: HashMap<Coordinate, usize> = y.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let mut occupied = vec![vec![false; numx]; numy];
    for child in &schematic.inner {
        let x1 = rev_x[&(child.origin.0 + schematic.origin.0)];
        let x2 = rev_x[&(child.origin.0 + child.size.0 + schematic.origin.0)];
        let y1 = rev_y[&(child.origin.1 + schematic.origin.1)];
        let y2 = rev_y[&(child.origin.1 + child.size.1 + schematic.origin.1)];
        (y1..=y2).for_each(|y| {
            occupied[y][x1..=x2].fill(true);
        });
    }
    Some(Grid {
        x: x.into_iter().collect(),
        y: y.into_iter().collect(),
        rev_x,
        rev_y,
        occupied,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

impl Direction {
    fn delta(&self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

impl Grid {
    pub fn print_grid(
        &self,
        cost_map: &[Vec<Option<i32>>],
        from: (Coordinate, Coordinate),
        to: (Coordinate, Coordinate),
    ) {
        let start = (self.rev_x[&from.0], self.rev_y[&from.1]);
        let end = (self.rev_x[&to.0], self.rev_y[&to.1]);
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
    }
    pub fn route(
        &self,
        from: (Coordinate, Coordinate),
        to: (Coordinate, Coordinate),
    ) -> Option<Vec<(Coordinate, Coordinate)>> {
        // We are going to use Lee's algorithm to route from the start to the end.
        let mut queue = VecDeque::new();
        let mut cost_map: Vec<Vec<Option<i32>>> = vec![vec![None; self.x.len()]; self.y.len()];
        println!("Routing from {:?} to {:?}", from, to);
        let start = (*self.rev_x.get(&from.0)?, *self.rev_y.get(&from.1)?);
        println!("Start: {start:?}");
        let end = (*self.rev_x.get(&to.0)?, *self.rev_y.get(&to.1)?);
        println!("End: {end:?}");
        queue.push_back((start, 0));
        cost_map[start.1][start.0] = Some(0);
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut final_cost = 0;
        while let Some((current, cost)) = queue.pop_front() {
            if current == end {
                final_cost = cost;
                break;
            }
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
                // If the next position has been visited already then continue
                if self.occupied[next.1][next.0] || cost_map[next.1][next.0].is_some() {
                    continue;
                }
                cost_map[next.1][next.0] = Some(cost + 1);
                queue.push_back((next, cost + 1));
            }
        }
        let mut path = Vec::new();
        let mut current = end;
        // To do the traceback, we first need to know the min cost
        let mut head_cost = final_cost;
        let mut tracking_direction = Direction::Right;
        while current != start {
            path.push((self.x[current.0], self.y[current.1]));
            // Check to see if by moving the tracking direction, we can
            // get to a cell with a lower cost than the head_cost.  If so,
            // then we move the tracking direction and update the head_cost.
            let (dx, dy) = tracking_direction.delta();
            let next = (current.0 as isize + dx, current.1 as isize + dy);
            if next.0 >= 0
                && next.0 < self.x.len() as isize
                && next.1 >= 0
                && next.1 < self.y.len() as isize
            {
                let next = (next.0 as usize, next.1 as usize);
                if let Some(next_cost) = cost_map[next.1][next.0]
                    && next_cost < head_cost
                {
                    // We can move in the tracking direction, so we do that and update the head_cost
                    head_cost = next_cost;
                    current = next;
                    continue;
                }
            }
            // We cannot move in the tracking direction, so look for the best direction to move in
            let mut found_direction = false;
            for direction in DIRECTIONS {
                let (dx, dy) = direction.delta();
                let next = (current.0 as isize + dx, current.1 as isize + dy);
                if next.0 >= 0
                    && next.0 < self.x.len() as isize
                    && next.1 >= 0
                    && next.1 < self.y.len() as isize
                {
                    let next = (next.0 as usize, next.1 as usize);
                    if let Some(next_cost) = cost_map[next.1][next.0]
                        && next_cost < head_cost
                    {
                        tracking_direction = direction;
                        head_cost = next_cost;
                        current = next;
                        found_direction = true;
                        break;
                    }
                }
            }
            if !found_direction {
                return None;
            }
        }
        path.push((self.x[start.0], self.y[start.1]));
        path.reverse();
        eprintln!("Routed path {path:?}");
        Some(path)
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
        let mut grid = Grid {
            x: x_coordinates,
            y: y_coordinates,
            rev_x,
            rev_y,
            occupied,
        }; // col 6, row 10
        let from = (Coordinate::from(6.0), Coordinate::from(9.0));
        // col 10, row 3
        let to = (Coordinate::from(10.0), Coordinate::from(3.0));
        let path = grid.route(from, to).unwrap();
        expect_test::expect![[r#"
            [
                (
                    Coordinate(
                        60,
                    ),
                    Coordinate(
                        90,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        90,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        80,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        70,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        60,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        50,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        40,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        30,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        20,
                    ),
                ),
                (
                    Coordinate(
                        50,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        60,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        70,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        80,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        90,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        100,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        100,
                    ),
                    Coordinate(
                        20,
                    ),
                ),
                (
                    Coordinate(
                        100,
                    ),
                    Coordinate(
                        30,
                    ),
                ),
            ]
        "#]]
        .assert_debug_eq(&path);
    }

    #[test]
    fn test_congested() {
        const GRID: [[u8; 5]; 5] = [
            [0, 0, 0, 0, 1],
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 1, 1, 0],
            [0, 0, 0, 0, 0],
        ];
        let x_coordinates = (0..5)
            .map(|n| Coordinate::from(n as f32))
            .collect::<Vec<_>>();
        let y_coordinates = (0..5)
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
        let mut grid = Grid {
            x: x_coordinates,
            y: y_coordinates,
            rev_x,
            rev_y,
            occupied,
        };
        let from = (Coordinate::from(2.0), Coordinate::from(2.0));
        let to = (Coordinate::from(4.0), Coordinate::from(4.0));
        let path = grid.route(from, to).unwrap();
        expect_test::expect![[r#"
            [
                (
                    Coordinate(
                        20,
                    ),
                    Coordinate(
                        20,
                    ),
                ),
                (
                    Coordinate(
                        20,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        20,
                    ),
                    Coordinate(
                        0,
                    ),
                ),
                (
                    Coordinate(
                        10,
                    ),
                    Coordinate(
                        0,
                    ),
                ),
                (
                    Coordinate(
                        0,
                    ),
                    Coordinate(
                        0,
                    ),
                ),
                (
                    Coordinate(
                        0,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
                (
                    Coordinate(
                        0,
                    ),
                    Coordinate(
                        20,
                    ),
                ),
                (
                    Coordinate(
                        0,
                    ),
                    Coordinate(
                        30,
                    ),
                ),
                (
                    Coordinate(
                        0,
                    ),
                    Coordinate(
                        40,
                    ),
                ),
                (
                    Coordinate(
                        10,
                    ),
                    Coordinate(
                        40,
                    ),
                ),
                (
                    Coordinate(
                        20,
                    ),
                    Coordinate(
                        40,
                    ),
                ),
                (
                    Coordinate(
                        30,
                    ),
                    Coordinate(
                        40,
                    ),
                ),
                (
                    Coordinate(
                        40,
                    ),
                    Coordinate(
                        40,
                    ),
                ),
            ]
        "#]]
        .assert_debug_eq(&path);
    }

    #[test]
    fn test_neighbors() {
        const GRID: [[u8; 5]; 5] = [
            [0, 0, 0, 0, 1],
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 1, 1, 0],
            [0, 0, 0, 0, 0],
        ];
        let x_coordinates = (0..5)
            .map(|n| Coordinate::from(n as f32))
            .collect::<Vec<_>>();
        let y_coordinates = (0..5)
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
        let mut grid = Grid {
            x: x_coordinates,
            y: y_coordinates,
            rev_x,
            rev_y,
            occupied,
        };
        let from = (Coordinate::from(2.0), Coordinate::from(2.0));
        let to = (Coordinate::from(2.0), Coordinate::from(1.0));
        let path = grid.route(from, to).unwrap();
        expect_test::expect![[r#"
            [
                (
                    Coordinate(
                        20,
                    ),
                    Coordinate(
                        20,
                    ),
                ),
                (
                    Coordinate(
                        20,
                    ),
                    Coordinate(
                        10,
                    ),
                ),
            ]
        "#]]
        .assert_debug_eq(&path);
    }

    #[test]
    fn test_fail() {
        const GRID: [[u8; 5]; 5] = [
            [0, 0, 0, 0, 1],
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 1, 1, 0],
            [0, 0, 1, 0, 0],
        ];
        let x_coordinates = (0..5)
            .map(|n| Coordinate::from(n as f32))
            .collect::<Vec<_>>();
        let y_coordinates = (0..5)
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
        let mut grid = Grid {
            x: x_coordinates,
            y: y_coordinates,
            rev_x,
            rev_y,
            occupied,
        };
        let from = (Coordinate::from(2.0), Coordinate::from(2.0));
        let to = (Coordinate::from(4.0), Coordinate::from(4.0));
        let path = grid.route(from, to);
        assert!(path.is_none());
    }
}
