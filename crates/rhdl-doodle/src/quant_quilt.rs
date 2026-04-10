use std::{cell, collections::BTreeMap, ops::Range};

use crate::grid::GRID_SIZE;

// We use a bit mask here to get the maximum performance.
const HSEG_EDGE: u8 = 0b0000_0001;
const HSEG_VERTEX: u8 = 0b0000_0010;
const HSEG_MASK: u8 = HSEG_EDGE | HSEG_VERTEX;
const VSEG_EDGE: u8 = 0b0000_0100;
const VSEG_VERTEX: u8 = 0b0000_1000;
const VSEG_VIRTUAL: u8 = 0b0001_0000;
const VSEG_MASK: u8 = VSEG_EDGE | VSEG_VERTEX | VSEG_VIRTUAL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HSegKind {
    Edge,
    Vertex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum VSegKind {
    Edge,
    Vertex,
    Virtual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Cell(u8);

impl Cell {
    fn hseg_kind(&self) -> Option<HSegKind> {
        if self.0 & HSEG_EDGE != 0 {
            Some(HSegKind::Edge)
        } else if self.0 & HSEG_VERTEX != 0 {
            Some(HSegKind::Vertex)
        } else {
            None
        }
    }

    fn vseg_kind(&self) -> Option<VSegKind> {
        if self.0 & VSEG_EDGE != 0 {
            Some(VSegKind::Edge)
        } else if self.0 & VSEG_VERTEX != 0 {
            Some(VSegKind::Vertex)
        } else if self.0 & VSEG_VIRTUAL != 0 {
            Some(VSegKind::Virtual)
        } else {
            None
        }
    }

    fn set_hseg_kind(&mut self, kind: HSegKind) {
        self.0 &= !HSEG_MASK;
        match kind {
            HSegKind::Edge => self.0 |= HSEG_EDGE,
            HSegKind::Vertex => self.0 |= HSEG_VERTEX,
        }
    }

    fn set_vseg_kind(&mut self, kind: VSegKind) {
        self.0 &= !VSEG_MASK;
        match kind {
            VSegKind::Edge => self.0 |= VSEG_EDGE,
            VSegKind::Vertex => self.0 |= VSEG_VERTEX,
            VSegKind::Virtual => self.0 |= VSEG_VIRTUAL,
        }
    }
}

/// Prompt: Render the quilt to the screen as ASCII art
/// for debugging.  Use a single line to represent edge segments
/// and double lines to represent vertex segments.  Use a
/// dotted line to represent (vertical) virtual segments.
/// For cells that are empty, render a single dot.
impl std::fmt::Display for Quilt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.nrows {
            // First line of the row: dot + horizontal segments
            for col in 0..self.ncols {
                let cell = &self.cells[col][row];
                write!(f, ".")?;
                match cell.hseg_kind() {
                    Some(HSegKind::Edge) => write!(f, "--")?,
                    Some(HSegKind::Vertex) => write!(f, "==")?,
                    None => write!(f, "  ")?,
                }
            }
            writeln!(f)?;

            // Second line of the row: vertical segments
            for col in 0..self.ncols {
                let cell = &self.cells[col][row];
                match cell.vseg_kind() {
                    Some(VSegKind::Edge) => write!(f, "|")?,
                    Some(VSegKind::Vertex) => write!(f, "║")?,
                    Some(VSegKind::Virtual) => write!(f, ":")?,
                    None => write!(f, " ")?,
                }
                write!(f, "  ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CoordX(i32);

fn x(val: f32) -> CoordX {
    CoordX((val / GRID_SIZE).round() as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CoordY(i32);

fn y(val: f32) -> CoordY {
    CoordY((val / GRID_SIZE).round() as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: CoordX,
    y: CoordY,
}

enum Edge {
    Horizontal(CoordY, Range<CoordX>),
    Vertical(CoordX, Range<CoordY>),
}

struct Quilt {
    cells: Vec<Vec<Cell>>,
    edges: Vec<Edge>,
    nrows: usize,
    ncols: usize,
    origin: Point,
    nodes: BTreeMap<NodeId, Rectangle>,
    node_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Rectangle {
    x_range: Range<CoordX>,
    y_range: Range<CoordY>,
}

impl Rectangle {
    fn intersects(&self, other: &Rectangle) -> bool {
        self.x_range.start < other.x_range.end
            && self.x_range.end > other.x_range.start
            && self.y_range.start < other.y_range.end
            && self.y_range.end > other.y_range.start
    }
    fn contains_y(&self, y: CoordY) -> bool {
        self.y_range.contains(&y)
    }
    fn contains_x(&self, x: CoordX) -> bool {
        self.x_range.contains(&x)
    }
    fn contains(&self, point: Point) -> bool {
        self.contains_x(point.x) && self.contains_y(point.y)
    }
    fn start_x(&self) -> CoordX {
        self.x_range.start
    }
    fn end_x(&self) -> CoordX {
        self.x_range.end
    }
    fn start_y(&self) -> CoordY {
        self.y_range.start
    }
    fn end_y(&self) -> CoordY {
        self.y_range.end
    }
    fn x_range(&self) -> Range<CoordX> {
        self.x_range.clone()
    }
    fn y_range(&self) -> Range<CoordY> {
        self.y_range.clone()
    }
}

const fn rectangle(x_range: Range<CoordX>, y_range: Range<CoordY>) -> Rectangle {
    Rectangle { x_range, y_range }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct NodeId(usize);

impl NodeId {
    fn next(self) -> Self {
        NodeId(self.0 + 1)
    }
}

enum Direction {
    North,
    South,
    East,
    West,
}

struct Segment {
    direction: Direction,
    length: usize,
    kind: VSegKind,
}

impl Quilt {
    fn new(nrows: usize, ncols: usize, origin: Point) -> Self {
        let cells = vec![vec![Cell(0); nrows]; ncols];
        Self {
            cells,
            edges: Vec::new(),
            nrows,
            ncols,
            origin,
            nodes: BTreeMap::new(),
            node_id: NodeId(0),
        }
    }
    fn corners(&self) -> Vec<Point> {
        let mut points: Vec<Point> = self
            .nodes
            .values()
            .flat_map(|rect| {
                let x_start = rect.start_x();
                let x_end = rect.end_x();
                let y_start = rect.start_y();
                let y_end = rect.end_y();
                [
                    Point {
                        x: x_start,
                        y: y_start,
                    },
                    Point {
                        x: x_start,
                        y: y_end,
                    },
                    Point {
                        x: x_end,
                        y: y_start,
                    },
                    Point { x: x_end, y: y_end },
                ]
            })
            .collect();
        points.extend(self.edges.iter().flat_map(|edge| match edge {
            Edge::Horizontal(y, x_range) => {
                let y = *y;
                let x_start = x_range.start;
                let x_end = x_range.end;
                [Point { x: x_start, y }, Point { x: x_end, y }]
            }
            Edge::Vertical(x, y_range) => {
                let x = *x;
                let y_start = y_range.start;
                let y_end = y_range.end;
                [Point { x, y: y_start }, Point { x, y: y_end }]
            }
        }));
        points
    }
    fn hline(&mut self, row: usize, cols: Range<usize>, kind: HSegKind) {
        for col in cols {
            self.cells[col][row].set_hseg_kind(kind);
        }
    }
    fn vline(&mut self, col: usize, rows: Range<usize>, kind: VSegKind) {
        for row in rows {
            self.cells[col][row].set_vseg_kind(kind);
        }
    }
    fn x_coord_to_index(&self, x: CoordX) -> usize {
        (x.0 - self.origin.x.0) as usize
    }
    fn y_coord_to_index(&self, y: CoordY) -> usize {
        (y.0 - self.origin.y.0) as usize
    }
    fn add_node(&mut self, rect: Rectangle) -> Option<NodeId> {
        if self.nodes.values().any(|r| r.intersects(&rect)) {
            return None;
        }
        let node_id = self.node_id;
        self.nodes.insert(node_id, rect);
        self.node_id = self.node_id.next();
        Some(node_id)
    }
    fn add_h_edge(&mut self, y: CoordY, x_start: CoordX, x_end: CoordX) {
        self.edges.push(Edge::Horizontal(y, x_start..x_end));
    }
    fn add_v_edge(&mut self, x: CoordX, y_start: CoordY, y_end: CoordY) {
        self.edges.push(Edge::Vertical(x, y_start..y_end));
    }
    fn cell_is_horizontally_blocked(&self, col: usize, row: usize) -> bool {
        // A cell is horizontally blocked if the horizontal segment
        // is an edge, and if the adjacent cell (on the left) is
        // also an edge.
        if col == 0 {
            return false;
        }
        let cell = &self.cells[col][row];
        let left_cell = &self.cells[col - 1][row];
        if cell.hseg_kind().is_some() && left_cell.hseg_kind().is_some() {
            return true;
        }
        false
    }
    // We want to run a boundary scan while travelling east,
    // We use a counter clockwise direction convention
    fn scan_east(&self, start: Point) -> Vec<Segment> {
        let mut direction = Direction::South;
        let col = self.x_coord_to_index(start.x);
        let row = self.y_coord_to_index(start.y);
        // Scan south (down) until we hit a cell with a 
        // horizontal edge segment, then turn east (left)
        // Start a new segment each time the vertical segment
        // kind changes (edge, vertex, virtual, none)
        let mut segments = Vec::new();
        let mut length = 0;
        for r in row..self.nrows {
            let cell = &self.cells[col][r];
            match cell.vseg_kind() {
                Some(kind) => {
                    if length > 0 {
                        segments.push(Segment {
                            direction,
                            length,
                            kind: VSegKind::Edge, // Placeholder, will be updated later
                        });
                    }
                    length = 1;
                }
                None => {
                    if length > 0 {
                        segments.push(Segment {
                            direction,
                            length,
                            kind: VSegKind::Edge, // Placeholder, will be updated later
                        });
                        length = 0;
                    }
                }
            }
        }

    }
    fn scan(&self, start: Point, direction: Direction) -> Vec<Segment> {
        match direction {
            Direction::East => self.scan_east(start),
            _ => todo!(),
        }
    }
    fn virtual_extend_from_corner(&mut self, corner: Point) {
        let col = self.x_coord_to_index(corner.x);
        let row = self.y_coord_to_index(corner.y);

        // Don't extend if the corner is horizontally blocked
        if self.cell_is_horizontally_blocked(col, row) {
            return;
        }

        // Extend downward from corner (including the corner itself)
        if self.cells[col][row].vseg_kind().is_none() {
            self.cells[col][row].set_vseg_kind(VSegKind::Virtual);
        }
        for r in row + 1..self.nrows {
            if self.cell_is_horizontally_blocked(col, r) || self.cells[col][r].vseg_kind().is_some()
            {
                break;
            }
            self.cells[col][r].set_vseg_kind(VSegKind::Virtual);
        }

        // Extend upward from corner
        for r in (0..row).rev() {
            if self.cells[col][r].vseg_kind().is_some() {
                break;
            }
            self.cells[col][r].set_vseg_kind(VSegKind::Virtual);
            if self.cell_is_horizontally_blocked(col, r) {
                break;
            }
        }
    }
    fn rectangularize(&mut self) {
        let corners = self.corners();
        let nodes = std::mem::take(&mut self.nodes);
        for rect in nodes.values() {
            // Mark the edges of the rectangle as vertex segments
            let x_start = self.x_coord_to_index(rect.start_x());
            let x_end = self.x_coord_to_index(rect.end_x());
            let y_start = self.y_coord_to_index(rect.start_y());
            let y_end = self.y_coord_to_index(rect.end_y());
            self.hline(y_start, x_start..x_end, HSegKind::Vertex);
            self.hline(y_end, x_start..x_end, HSegKind::Vertex);
            self.vline(x_start, y_start..y_end, VSegKind::Vertex);
            self.vline(x_end, y_start..y_end, VSegKind::Vertex);
        }
        let edges = std::mem::take(&mut self.edges);
        for edge in &edges {
            match edge {
                Edge::Horizontal(y, x_range) => {
                    let row = self.y_coord_to_index(*y);
                    let col_start = self.x_coord_to_index(x_range.start);
                    let col_end = self.x_coord_to_index(x_range.end);
                    self.hline(row, col_start..col_end, HSegKind::Edge);
                }
                Edge::Vertical(x, y_range) => {
                    let col = self.x_coord_to_index(*x);
                    let row_start = self.y_coord_to_index(y_range.start);
                    let row_end = self.y_coord_to_index(y_range.end);
                    self.vline(col, row_start..row_end, VSegKind::Edge);
                }
            }
        }
        self.nodes = nodes;
        self.edges = edges;
        for corner in corners {
            self.virtual_extend_from_corner(corner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quilt() {
        let mut quilt = Quilt::new(
            11,
            20,
            Point {
                x: CoordX(0),
                y: CoordY(0),
            },
        );
        let rect1 = rectangle(x(10.0)..x(50.0), y(10.0)..y(30.0));
        let rect2 = rectangle(x(30.0)..x(70.0), y(40.0)..y(70.0));
        let rect3 = rectangle(x(120.0)..x(160.0), y(50.0)..y(100.0));
        quilt.add_node(rect1).unwrap();
        quilt.add_node(rect2).unwrap();
        quilt.add_node(rect3).unwrap();
        quilt.add_h_edge(y(20.0), x(50.0), x(80.0));
        quilt.add_v_edge(x(80.0), y(20.0), y(50.0));
        quilt.add_h_edge(y(50.0), x(70.0), x(80.0));
        quilt.add_h_edge(y(30.0), x(80.0), x(150.0));
        quilt.add_v_edge(x(150.0), y(30.0), y(50.0));
        quilt.add_v_edge(x(50.0), y(70.0), y(90.0));
        quilt.add_h_edge(y(90.0), x(50.0), x(120.0));
        quilt.rectangularize();
        println!("{}", quilt);
    }
}
