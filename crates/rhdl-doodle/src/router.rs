use std::{cell::RefCell, collections::HashSet};

use egui::{Pos2, Rect, pos2};
use pathfinding::{num_traits::Zero, prelude::*};

use crate::{grid::GRID_SIZE, router_ng::RouterNG, state::RouteEdge, turtle::Mark};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct Cost(i64);

impl pathfinding::num_traits::Zero for Cost {
    fn zero() -> Self {
        COST_ZERO
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

const UNIT_SCALE: f64 = 16_777_216.0; // 2^24

impl From<Cost> for f64 {
    fn from(value: Cost) -> Self {
        value.0 as f64 / UNIT_SCALE as f64
    }
}

impl From<f64> for Cost {
    fn from(value: f64) -> Self {
        Self((value * UNIT_SCALE) as i64)
    }
}

impl From<f32> for Cost {
    fn from(value: f32) -> Self {
        Self((value as f64 * UNIT_SCALE) as i64)
    }
}

impl Cost {
    pub const fn new(cost: f64) -> Self {
        Self((cost * UNIT_SCALE) as i64)
    }
}

impl std::ops::AddAssign<Cost> for Cost {
    fn add_assign(&mut self, rhs: Cost) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign<Cost> for Cost {
    fn sub_assign(&mut self, rhs: Cost) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Add<Cost> for Cost {
    type Output = Self;

    fn add(self, rhs: Cost) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Cost> for Cost {
    type Output = Self;

    fn sub(self, rhs: Cost) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul<Cost> for i64 {
    type Output = Cost;

    fn mul(self, rhs: Cost) -> Self::Output {
        Cost(self * rhs.0)
    }
}

impl std::ops::Mul<f64> for Cost {
    type Output = Cost;

    fn mul(self, rhs: f64) -> Self::Output {
        Cost((self.0 as f64 * rhs) as i64)
    }
}

pub const COST_ZERO: Cost = Cost(0);

impl std::fmt::Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0 as f64 / UNIT_SCALE)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cell {
    Vertical {
        south: Cost,
    }, // Vertical and available for routing
    Horizontal {
        east: Cost,
    }, // Horizontal and available for routing
    HorizontalOnly {
        east: Cost,
    }, // Horizontal only and available for routing - cannot be converted into a corner
    Corner {
        east: Cost,
        south: Cost,
    }, // Corner and available for routing
    Blocked, // Blocked and unavailable for routing
    #[default]
    Empty, // Empty and unavailable for routing.
}

impl Cell {
    pub fn east_cost(&self) -> Option<Cost> {
        match self {
            Cell::Horizontal { east }
            | Cell::Corner { east, .. }
            | Cell::HorizontalOnly { east } => Some(*east),
            _ => None,
        }
    }
    pub fn south_cost(&self) -> Option<Cost> {
        match self {
            Cell::Vertical { south } | Cell::Corner { south, .. } => Some(*south),
            _ => None,
        }
    }
    pub fn enable_horizontal(&mut self, cost: Cost) {
        match self {
            Cell::Vertical { south } => {
                *self = Cell::Corner {
                    east: cost,
                    south: *south,
                };
            }
            Cell::Empty => {
                *self = Cell::Horizontal { east: cost };
            }
            _ => {}
        }
    }
    pub fn enable_vertical(&mut self, cost: Cost) {
        match self {
            Cell::Horizontal { east } => {
                *self = Cell::Corner {
                    east: *east,
                    south: cost,
                };
            }
            Cell::Empty => {
                *self = Cell::Vertical { south: cost };
            }
            _ => {}
        }
    }
    pub fn add_east_cost(&mut self, cost: Cost) {
        match self {
            Cell::Horizontal { east }
            | Cell::Corner { east, .. }
            | Cell::HorizontalOnly { east } => {
                *east += cost;
            }
            _ => {}
        }
    }
    pub fn add_south_cost(&mut self, cost: Cost) {
        match self {
            Cell::Vertical { south } | Cell::Corner { south, .. } => {
                *south += cost;
            }
            _ => {}
        }
    }
    pub fn is_vertical(&self) -> bool {
        matches!(self, Cell::Vertical { .. })
    }
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Cell::Horizontal { .. } | Cell::HorizontalOnly { .. })
    }
    pub fn is_corner(&self) -> bool {
        matches!(self, Cell::Corner { .. })
    }
    pub fn is_blocked(&self) -> bool {
        matches!(self, Cell::Blocked)
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

fn point(row: usize, col: usize) -> Point {
    Point { row, col }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

const TURN_COST: Cost = Cost::new(25.0);
const MOVE_COST: Cost = Cost::new(1.0);
pub const WIRE_COST: Cost = Cost::new(10.0);

fn turn_cost(from: Option<Direction>, to: Direction) -> Cost {
    if let Some(from_dir) = from {
        if from_dir == to { COST_ZERO } else { TURN_COST }
    } else {
        COST_ZERO
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct SearchState {
    node: Point,
    dir: Option<Direction>,
}

impl From<(usize, usize)> for Point {
    fn from(value: (usize, usize)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Graph {
    nodes: Vec<Vec<Cell>>,
    nrows: usize,
    ncols: usize,
    origin: Pos2,
}

// Render the graph to the console for debug purposes.
// The cells display as follows:
//   Vertical:
//    '.  '
//    ':  '
//   Horizontal:
//    '.--'
//    '   '
//   Corner:
//    '+--'
//    ':  '
//   Blocked:
//    '.xx'
//    'xxx'
//   Empty:
//    '.  '
//    '   '
// So each cell takes up 6 characters in a 2x3 block of the output.
impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.nodes {
            for line in 0..2 {
                for cell in row {
                    let cell_str = match cell {
                        Cell::Vertical { .. } => {
                            if line == 0 {
                                ".  "
                            } else {
                                ":  "
                            }
                        }
                        Cell::Horizontal { .. } => {
                            if line == 0 {
                                ".--"
                            } else {
                                "   "
                            }
                        }
                        Cell::HorizontalOnly { .. } => {
                            if line == 0 {
                                ".=="
                            } else {
                                "   "
                            }
                        }
                        Cell::Corner { .. } => {
                            if line == 0 {
                                "+--"
                            } else {
                                ":  "
                            }
                        }
                        Cell::Blocked => {
                            if line == 0 {
                                ".xx"
                            } else {
                                "xxx"
                            }
                        }
                        Cell::Empty => {
                            if line == 0 {
                                ".  "
                            } else {
                                "   "
                            }
                        }
                    };
                    write!(f, "{}", cell_str)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Graph {
    pub fn point(&self, pos: Pos2) -> Option<Point> {
        if pos.x < self.origin.x
            || pos.y < self.origin.y
            || pos.x >= self.origin.x + self.ncols as f32 * GRID_SIZE
            || pos.y >= self.origin.y + self.nrows as f32 * GRID_SIZE
        {
            return None;
        }
        let col = ((pos.x - self.origin.x) / GRID_SIZE).floor() as usize;
        let row = ((pos.y - self.origin.y) / GRID_SIZE).floor() as usize;
        Some(Point { row, col })
    }
    pub fn pos(&self, node: Point) -> Pos2 {
        let x = self.origin.x + node.col as f32 * GRID_SIZE;
        let y = self.origin.y + node.row as f32 * GRID_SIZE;
        Pos2 { x, y }
    }
    pub fn iter(&self) -> impl Iterator<Item = (Point, &Cell)> {
        self.nodes.iter().enumerate().flat_map(|(row_idx, row)| {
            row.iter().enumerate().map(move |(col_idx, edges)| {
                (
                    Point {
                        row: row_idx,
                        col: col_idx,
                    },
                    edges,
                )
            })
        })
    }
    pub fn new(nrows: usize, ncols: usize, origin: Pos2) -> Self {
        // Start out with full connectivity
        let mut nodes = vec![vec![Cell::default(); ncols]; nrows];
        Self {
            nodes,
            nrows,
            ncols,
            origin,
        }
    }
    fn row_segment(
        &self,
        row: usize,
        start_col: usize,
        end_col: usize,
    ) -> impl Iterator<Item = &Cell> + '_ {
        let row = row.clamp(0, self.nrows - 1);
        let start_col = start_col.clamp(0, self.ncols - 1);
        let end_col = end_col.clamp(0, self.ncols - 1);
        self.nodes[row][start_col..=end_col].iter()
    }
    fn row_segment_mut(
        &mut self,
        row: usize,
        start_col: usize,
        end_col: usize,
    ) -> impl Iterator<Item = &mut Cell> + '_ {
        let row = row.clamp(0, self.nrows - 1);
        let start_col = start_col.clamp(0, self.ncols - 1);
        let end_col = end_col.clamp(0, self.ncols - 1);
        self.nodes[row][start_col..=end_col].iter_mut()
    }
    fn col_segment(
        &self,
        col: usize,
        start_row: usize,
        end_row: usize,
    ) -> impl Iterator<Item = &Cell> + '_ {
        let col = col.clamp(0, self.ncols - 1);
        let start_row = start_row.clamp(0, self.nrows - 1);
        let end_row = end_row.clamp(0, self.nrows - 1);
        self.nodes[start_row..=end_row]
            .iter()
            .map(move |row_slice| &row_slice[col])
    }
    fn col_segment_mut(
        &mut self,
        col: usize,
        start_row: usize,
        end_row: usize,
    ) -> impl Iterator<Item = &mut Cell> + '_ {
        let col = col.clamp(0, self.ncols - 1);
        let start_row = start_row.clamp(0, self.nrows - 1);
        let end_row = end_row.clamp(0, self.nrows - 1);
        self.nodes[start_row..=end_row]
            .iter_mut()
            .map(move |row_slice| &mut row_slice[col])
    }
    pub fn seed_vert_channel(&mut self, seed: Pos2, cost: Cost) {
        //self.ng.seed_vert_channel(seed, cost);
        let Some(point) = self.point(seed) else {
            return;
        };
        let row = point.row;
        let col = point.col;
        let row = row.clamp(0, self.nrows - 1);
        let col = col.clamp(0, self.ncols - 1);
        // Try to add a vertical edge starting from this seed point,
        // extending downwards until we hit a blocked cell or the end of the grid.
        for r in row..self.nrows - 1 {
            if self.nodes[r][col].is_blocked() {
                break;
            }
            self.nodes[r][col].enable_vertical(cost);
        }
        // Try also to extend upwards from the seed point.
        for r in (0..row).rev() {
            if self.nodes[r][col].is_blocked() {
                break;
            }
            self.nodes[r][col].enable_vertical(cost);
        }
    }
    pub fn seed_horiz_channel(&mut self, seed: Pos2, cost: Cost) {
        //self.ng.seed_horiz_channel(seed, cost);
        let Some(point) = self.point(seed) else {
            return;
        };
        let row = point.row;
        let col = point.col;
        let row = row.clamp(0, self.nrows - 1);
        let col = col.clamp(0, self.ncols - 1);
        // Try to add a horizontal edge starting from this seed point,
        // extending to the right until we hit a blocked cell or the end of the grid.
        for c in col..self.ncols - 1 {
            if self.nodes[row][c].is_blocked() {
                break;
            }
            self.nodes[row][c].enable_horizontal(cost);
        }
        // Try also to extend to the left from the seed point.
        for c in (0..col).rev() {
            if self.nodes[row][c].is_blocked() {
                break;
            }
            self.nodes[row][c].enable_horizontal(cost);
        }
    }
    pub fn block_rectangle(&mut self, rect: Rect) {
        //self.ng.add_block(rect.left_top(), rect.right_bottom());
        let Some(pos_lt) = self.point(rect.left_top()) else {
            return;
        };
        let Some(pos_rb) = self.point(rect.right_bottom()) else {
            return;
        };
        self.block_rect(pos_lt, pos_rb);
        // Add the routing channels around the blocked rectangle.
        for vert_channel in 0..5 {
            let vert_channel = vert_channel as f32;
            let cost = if vert_channel == 0.0 {
                Cost::new(0.2)
            } else {
                COST_ZERO
            };
            self.seed_vert_channel(
                pos2(rect.left() - (vert_channel + 2.0) * GRID_SIZE, rect.top()),
                cost,
            );
            self.seed_vert_channel(
                pos2(
                    rect.left() - (vert_channel + 2.0) * GRID_SIZE,
                    rect.bottom(),
                ),
                cost,
            );
            self.seed_vert_channel(
                pos2(rect.right() + (vert_channel + 3.0) * GRID_SIZE, rect.top()),
                cost,
            );
            self.seed_vert_channel(
                pos2(
                    rect.right() + (vert_channel + 3.0) * GRID_SIZE,
                    rect.bottom(),
                ),
                cost,
            );
        }
        for horz_channel in 0..5 {
            let horz_channel = horz_channel as f32;
            self.seed_horiz_channel(
                pos2(rect.left(), rect.top() - (horz_channel + 2.0) * GRID_SIZE),
                COST_ZERO,
            );
            self.seed_horiz_channel(
                pos2(rect.right(), rect.top() - (horz_channel + 2.0) * GRID_SIZE),
                COST_ZERO,
            );
            self.seed_horiz_channel(
                pos2(
                    rect.left(),
                    rect.bottom() + (horz_channel + 1.0) * GRID_SIZE,
                ),
                COST_ZERO,
            );
            self.seed_horiz_channel(
                pos2(
                    rect.right(),
                    rect.bottom() + (horz_channel + 1.0) * GRID_SIZE,
                ),
                COST_ZERO,
            );
        }
        // Mark the cells immediately to the right of the block as horizontal-only
        self.col_segment_mut(pos_lt.col - 2, pos_lt.row, pos_rb.row)
            .for_each(|e| {
                *e = Cell::HorizontalOnly {
                    east: Cost::new(0.5),
                };
            });
        self.col_segment_mut(pos_rb.col + 1, pos_lt.row, pos_rb.row)
            .for_each(|e| {
                *e = Cell::HorizontalOnly {
                    east: Cost::new(0.5),
                };
            });
    }
    fn block_rect(&mut self, pos1: Point, pos2: Point) {
        let start_row = pos1.row.min(pos2.row).saturating_sub(1);
        let end_row = pos1.row.max(pos2.row);
        let start_col = pos1.col.min(pos2.col).saturating_sub(1);
        let end_col = pos1.col.max(pos2.col);
        (start_row..=end_row).for_each(|row| {
            self.row_segment_mut(row, start_col, end_col)
                .for_each(|e| *e = Cell::Blocked);
        });
    }
    pub fn mark_route(&mut self, start: Point, end: Point) {
        // The two points should be on a vertical or horizontal edge.
        // It is possible they are not.  So we cover both cases here.
        self.row_segment_mut(start.row, start.col.min(end.col), start.col.max(end.col))
            .for_each(|e| {
                if e.east_cost().is_none() {
                    e.enable_horizontal(WIRE_COST);
                } else {
                    e.add_east_cost(WIRE_COST);
                }
            });
        self.col_segment_mut(start.col, start.row.min(end.row), start.row.max(end.row))
            .for_each(|e| {
                if e.south_cost().is_none() {
                    e.enable_vertical(WIRE_COST);
                } else {
                    e.add_south_cost(WIRE_COST);
                }
            });
    }
    pub fn is_route_blocked(&self, start: Pos2, edges: &[RouteEdge]) -> bool {
        let mut turtle = start;
        // Scan through the route and check for any blocked cells
        for edge in edges {
            let segment_start = self.point(turtle);
            if segment_start.is_none() {
                return false;
            }
            match edge {
                RouteEdge::Horizontal(len) => {
                    turtle.x += *len;
                }
                RouteEdge::Vertical(len) => {
                    turtle.y += *len;
                }
            }
            let segment_stop = self.point(turtle);
            if segment_stop.is_none() {
                return false;
            }
            let segment_start = segment_start.unwrap();
            let segment_stop = segment_stop.unwrap();
            if segment_start.row == segment_stop.row {
                if self
                    .row_segment(
                        segment_start.row,
                        segment_start.col.min(segment_stop.col),
                        segment_start.col.max(segment_stop.col),
                    )
                    .any(|cell| cell.is_blocked())
                {
                    return true;
                }
            } else if segment_start.col == segment_stop.col
                && self
                    .col_segment(
                        segment_start.col,
                        segment_start.row.min(segment_stop.row),
                        segment_start.row.max(segment_stop.row),
                    )
                    .any(|cell| cell.is_blocked())
            {
                return true;
            }
        }
        false
    }
    pub fn add_route(&mut self, start: Pos2, edges: &[RouteEdge]) {
        //self.ng.add_route(start, edges, WIRE_COST);
        let tic = std::time::Instant::now();
        //        self.ng.normalize_segments();
        eprintln!(
            "Adding route starting at {:?} with edges {:?} - elapsed time to normalize segments: {:?}",
            start,
            edges,
            tic.elapsed()
        );
        let mut turtle = start;
        for edge in edges {
            let segment_start = self.point(turtle).expect("Route edge starts out of bounds");
            match edge {
                RouteEdge::Horizontal(len) => {
                    turtle.x += *len;
                }
                RouteEdge::Vertical(len) => {
                    turtle.y += *len;
                }
            }
            let segment_stop = self.point(turtle).expect("Route edge ends out of bounds");
            self.mark_route(segment_start, segment_stop);
        }
    }
    pub fn waypoint_path(
        &mut self,
        start: Pos2,
        waypoints: &[Pos2],
        head: Pos2,
    ) -> Option<(Vec<Point>, HashSet<Point>, Cost)> {
        let mut path = Vec::new();
        let mut seen = HashSet::new();
        let mut total_cost = COST_ZERO;
        self.seed_horiz_channel(start, COST_ZERO);
        self.seed_vert_channel(start, COST_ZERO);
        let mut current_start = self.point(start)?;
        for &waypoint in waypoints.iter().chain(std::iter::once(&head)) {
            self.seed_horiz_channel(waypoint, COST_ZERO);
            self.seed_vert_channel(waypoint, COST_ZERO);
            let waypoint = self.point(waypoint)?;
            if let Some((segment_path, segment_seen, segment_cost)) =
                self.path_find(current_start, waypoint)
            {
                path.extend(segment_path);
                seen.extend(segment_seen);
                total_cost += segment_cost;
                current_start = waypoint;
            } else {
                return None;
            }
        }
        Some((path, seen, total_cost))
    }
    pub fn path_find(
        &mut self,
        start: Point,
        end: Point,
    ) -> Option<(Vec<Point>, HashSet<Point>, Cost)> {
        let start = SearchState {
            node: start,
            dir: None,
        };
        /*        let result = dijkstra(
            &start,
            |state| self.successors(state),
            |state| state.node == end,
        );
        */
        let seen: RefCell<HashSet<Point>> = RefCell::new(HashSet::new());
        let result = astar(
            &start,
            |state| {
                seen.borrow_mut().insert(state.node);
                self.successors(state)
            },
            |state| {
                let row_diff =
                    (state.node.row as isize - end.row as isize).abs() as i64 * MOVE_COST;
                let col_diff =
                    (state.node.col as isize - end.col as isize).abs() as i64 * MOVE_COST;
                // Manhattan distance - include a bend cost if the target is not
                // on the same path.
                if let Some(dir) = state.dir {
                    if (dir == Direction::North || dir == Direction::South) && !col_diff.is_zero() {
                        return row_diff + col_diff + TURN_COST;
                    }
                    if (dir == Direction::East || dir == Direction::West) && !row_diff.is_zero() {
                        return row_diff + col_diff + TURN_COST;
                    }
                }

                row_diff + col_diff
            },
            |state| state.node == end,
        );
        result.map(|(path, min_cost)| {
            (
                path.into_iter().map(|s| s.node).collect(),
                seen.into_inner(),
                min_cost,
            )
        })
    }
    fn successors(&self, state: &SearchState) -> Vec<(SearchState, Cost)> {
        let Point { row, col } = state.node;
        let prev_dir = state.dir;
        // Get the edges outgoing for this node
        let mut successors = Vec::new();
        if let Some(east) = self.nodes[row][col].east_cost() {
            successors.push((
                SearchState {
                    node: Point { row, col: col + 1 },
                    dir: Some(Direction::East),
                },
                east + MOVE_COST + turn_cost(prev_dir, Direction::East),
            ));
        }
        if col > 0
            && let Some(east) = self.nodes[row][col - 1].east_cost()
        {
            successors.push((
                SearchState {
                    node: Point { row, col: col - 1 },
                    dir: Some(Direction::West),
                },
                east + MOVE_COST + turn_cost(prev_dir, Direction::West),
            ));
        }
        if let Some(south) = self.nodes[row][col].south_cost() {
            successors.push((
                SearchState {
                    node: Point { row: row + 1, col },
                    dir: Some(Direction::South),
                },
                south + MOVE_COST + turn_cost(prev_dir, Direction::South),
            ));
        }
        if row > 0
            && let Some(south) = self.nodes[row - 1][col].south_cost()
        {
            successors.push((
                SearchState {
                    node: Point { row: row - 1, col },
                    dir: Some(Direction::North),
                },
                south + MOVE_COST + turn_cost(prev_dir, Direction::North),
            ));
        }
        successors
    }
}
