use std::{cell::RefCell, collections::HashSet};

use egui::Pos2;
use pathfinding::prelude::*;

use crate::grid::GRID_SIZE;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Edges {
    pub east: Option<i16>,
    pub south: Option<i16>,
}

impl Edges {
    pub fn full_no_weight() -> Self {
        Self {
            east: Some(0),
            south: Some(0),
        }
    }
    pub fn add_east_cost(&mut self, cost: i16) {
        if let Some(east) = self.east.as_mut() {
            *east += cost;
        }
    }
    pub fn add_south_cost(&mut self, cost: i16) {
        if let Some(south) = self.south.as_mut() {
            *south += cost;
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Node {
    pub row: usize,
    pub col: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

const TURN_COST: i16 = 10;

fn turn_cost(from: Option<Direction>, to: Direction) -> i16 {
    if let Some(from_dir) = from {
        if from_dir == to { 0 } else { TURN_COST }
    } else {
        0
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct SearchState {
    node: Node,
    dir: Option<Direction>,
}

impl From<(usize, usize)> for Node {
    fn from(value: (usize, usize)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    nodes: Vec<Vec<Edges>>,
    nrows: usize,
    ncols: usize,
    origin: Pos2,
}

impl Graph {
    pub fn node(&self, pos: Pos2) -> Node {
        let col = ((pos.x - self.origin.x) / GRID_SIZE).floor() as usize;
        let row = ((pos.y - self.origin.y) / GRID_SIZE).floor() as usize;
        Node { row, col }
    }
    pub fn pos(&self, node: Node) -> Pos2 {
        let x = self.origin.x + node.col as f32 * GRID_SIZE;
        let y = self.origin.y + node.row as f32 * GRID_SIZE;
        Pos2 { x, y }
    }
    pub fn iter(&self) -> impl Iterator<Item = (Node, &Edges)> {
        self.nodes.iter().enumerate().flat_map(|(row_idx, row)| {
            row.iter().enumerate().map(move |(col_idx, edges)| {
                (
                    Node {
                        row: row_idx,
                        col: col_idx,
                    },
                    edges,
                )
            })
        })
    }
    pub fn new(nrows: usize, ncols: usize, origin: Pos2) -> Self {
        let mut nodes = vec![vec![Edges::full_no_weight(); ncols]; nrows];
        // All internal nodes have full connectivity, but anything on the edges
        // needs to have the corresponding edges removed
        nodes[nrows - 1]
            .iter_mut()
            .for_each(|cell| cell.south = None);
        nodes.iter_mut().for_each(|row| {
            row[ncols - 1].east = None;
        });
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
    ) -> impl Iterator<Item = &Edges> + '_ {
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
    ) -> impl Iterator<Item = &mut Edges> + '_ {
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
    ) -> impl Iterator<Item = &Edges> + '_ {
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
    ) -> impl Iterator<Item = &mut Edges> + '_ {
        let col = col.clamp(0, self.ncols - 1);
        let start_row = start_row.clamp(0, self.nrows - 1);
        let end_row = end_row.clamp(0, self.nrows - 1);
        self.nodes[start_row..=end_row]
            .iter_mut()
            .map(move |row_slice| &mut row_slice[col])
    }
    pub fn block_rect(&mut self, pos1: Node, pos2: Node) {
        let start_row = pos1.row.min(pos2.row);
        let end_row = pos1.row.max(pos2.row);
        let start_col = pos1.col.min(pos2.col);
        let end_col = pos1.col.max(pos2.col);
        (start_row..=end_row).for_each(|row| {
            self.row_segment_mut(row, start_col, end_col)
                .for_each(|e| *e = Edges::default());
        });
        if start_row != 0 {
            self.row_segment_mut(start_row - 1, start_col, end_col)
                .for_each(|e| e.south = None);
        }
        if start_col != 0 {
            self.col_segment_mut(start_col - 1, start_row, end_row)
                .for_each(|e| e.east = None);
        }
    }
    pub fn north_south_bumpers(&mut self, pos1: Node, pos2: Node, distance: usize, cost: i16) {
        let start_row = pos1.row.min(pos2.row);
        let end_row = pos1.row.max(pos2.row);
        let start_col = pos1.col.min(pos2.col);
        let end_col = pos1.col.max(pos2.col);
        if start_col >= distance {
            self.col_segment_mut(
                start_col - distance,
                start_row.saturating_sub(distance),
                end_row + distance,
            )
            .for_each(|e| {
                e.add_south_cost(cost);
            });
        }
        if end_col + distance < self.ncols {
            self.col_segment_mut(
                end_col + distance,
                start_row.saturating_sub(distance),
                end_row + distance,
            )
            .for_each(|e| {
                e.add_south_cost(cost);
            });
        }
    }
    pub fn east_west_bumpers(&mut self, pos1: Node, pos2: Node, distance: usize, cost: i16) {
        let start_row = pos1.row.min(pos2.row);
        let end_row = pos1.row.max(pos2.row);
        let start_col = pos1.col.min(pos2.col);
        let end_col = pos1.col.max(pos2.col);
        if start_row >= distance {
            self.row_segment_mut(
                start_row - distance,
                start_col.saturating_sub(distance),
                end_col + distance,
            )
            .for_each(|e| {
                e.add_east_cost(cost);
            });
        }
        if end_row + distance < self.nrows {
            self.row_segment_mut(
                end_row + distance,
                start_col.saturating_sub(distance),
                end_col + distance,
            )
            .for_each(|e| {
                e.add_east_cost(cost);
            });
        }
    }
    pub fn path_find(&mut self, start: Node, end: Node) -> Option<(Vec<Node>, HashSet<Node>, i16)> {
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
        let mut seen: RefCell<HashSet<Node>> = RefCell::new(HashSet::new());
        let result = astar(
            &start,
            |state| {
                seen.borrow_mut().insert(state.node);
                self.successors(state)
            },
            |state| {
                let row_diff = (state.node.row as isize - end.row as isize).abs() as i16;
                let col_diff = (state.node.col as isize - end.col as isize).abs() as i16;
                // Manhattan distance - include a bend cost if the target is not
                // on the same path.
                if let Some(dir) = state.dir {
                    if (dir == Direction::North || dir == Direction::South) && col_diff > 0 {
                        return row_diff + col_diff + TURN_COST;
                    }
                    if (dir == Direction::East || dir == Direction::West) && row_diff > 0 {
                        return row_diff + col_diff + TURN_COST;
                    }
                }

                ((row_diff + col_diff) as f32 * 1.2) as i16
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
    fn successors(&self, state: &SearchState) -> Vec<(SearchState, i16)> {
        let Node { row, col } = state.node;
        let prev_dir = state.dir;
        // Get the edges outgoing for this node
        let mut successors = Vec::new();
        if let Some(east_cost) = self.nodes[row][col].east {
            successors.push((
                SearchState {
                    node: Node { row, col: col + 1 },
                    dir: Some(Direction::East),
                },
                east_cost + 1 + turn_cost(prev_dir, Direction::East),
            ));
        }
        if let Some(south_cost) = self.nodes[row][col].south {
            successors.push((
                SearchState {
                    node: Node { row: row + 1, col },
                    dir: Some(Direction::South),
                },
                south_cost + 1 + turn_cost(prev_dir, Direction::South),
            ));
        }
        // Check the incoming edges for this node as well.
        if col > 0
            && let Some(east_cost) = self.nodes[row][col - 1].east
        {
            successors.push((
                SearchState {
                    node: Node { row, col: col - 1 },
                    dir: Some(Direction::West),
                },
                east_cost + 1 + turn_cost(prev_dir, Direction::West),
            ));
        }
        if row > 0
            && let Some(south_cost) = self.nodes[row - 1][col].south
        {
            successors.push((
                SearchState {
                    node: Node { row: row - 1, col },
                    dir: Some(Direction::North),
                },
                south_cost + 1 + turn_cost(prev_dir, Direction::North),
            ));
        }
        successors
    }
}
