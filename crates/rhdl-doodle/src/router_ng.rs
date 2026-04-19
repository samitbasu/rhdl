use std::collections::{BTreeMap, BTreeSet};

use pathfinding::directed::dijkstra::dijkstra;
use petgraph::{
    graph::{NodeIndex, UnGraph},
    visit::EdgeRef,
};

use crate::{
    state::{RouteDirection, RouteEdge, Waypoint, WaypointId},
    turtle::{Mark, Turtle},
};

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
        value.0 as f64 / UNIT_SCALE
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

// This is a lattice point on the grid.  The grid is double ended
// and the coordinates can be negative, so we use a signed integer.
// We use two newtype wrappers to handle the two axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordX(i32);

impl CoordX {
    fn min(self, other: Self) -> Self {
        CoordX(self.0.min(other.0))
    }

    fn max(self, other: Self) -> Self {
        CoordX(self.0.max(other.0))
    }
}

impl From<f32> for CoordX {
    fn from(value: f32) -> Self {
        CoordX((value / crate::grid::GRID_SIZE).round() as i32)
    }
}

impl From<i32> for CoordX {
    fn from(value: i32) -> Self {
        CoordX(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordY(i32);

impl CoordY {
    fn min(self, other: Self) -> Self {
        CoordY(self.0.min(other.0))
    }

    fn max(self, other: Self) -> Self {
        CoordY(self.0.max(other.0))
    }
}

impl From<f32> for CoordY {
    fn from(value: f32) -> Self {
        CoordY((value / crate::grid::GRID_SIZE).round() as i32)
    }
}

impl From<i32> for CoordY {
    fn from(value: i32) -> Self {
        CoordY(value)
    }
}

impl std::ops::Add for CoordX {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        CoordX(self.0 + rhs.0)
    }
}

impl std::ops::Sub for CoordX {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        CoordX(self.0 - rhs.0)
    }
}

impl std::ops::Add for CoordY {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        CoordY(self.0 + rhs.0)
    }
}

impl std::ops::Sub for CoordY {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        CoordY(self.0 - rhs.0)
    }
}

// A point on the grid is a pair of coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: CoordX,
    pub y: CoordY,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x.0, self.y.0)
    }
}

impl Point {
    pub fn manhattan_distance(self, other: Point) -> i32 {
        (self.x.0 - other.x.0).abs() + (self.y.0 - other.y.0).abs()
    }
}

fn point(x: impl Into<CoordX>, y: impl Into<CoordY>) -> Point {
    Point {
        x: x.into(),
        y: y.into(),
    }
}

impl std::ops::Add<CoordX> for Point {
    type Output = Self;
    fn add(self, rhs: CoordX) -> Self {
        Point {
            x: self.x + rhs,
            y: self.y,
        }
    }
}

impl std::ops::Add<CoordY> for Point {
    type Output = Self;
    fn add(self, rhs: CoordY) -> Self {
        Point {
            x: self.x,
            y: self.y + rhs,
        }
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Self;
    fn add(self, rhs: Point) -> Self {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

const INFINITY_X: CoordX = CoordX(i32::MAX >> 4);
const INFINITY_Y: CoordY = CoordY(i32::MAX >> 4);
const NEG_INFINITY_X: CoordX = CoordX(i32::MIN >> 4);
const NEG_INFINITY_Y: CoordY = CoordY(i32::MIN >> 4);

// Conversion from a Pos2 to a point cannot fail unless there is an
// overflow/underflow situation, which we do not handle.
impl From<egui::Pos2> for Point {
    fn from(pos: egui::Pos2) -> Self {
        Point {
            x: CoordX::from(pos.x),
            y: CoordY::from(pos.y),
        }
    }
}

// Round tripping will move any Pos2 to the nearest lattice point.
impl From<Point> for egui::Pos2 {
    fn from(point: Point) -> Self {
        egui::Pos2::new(
            (point.x.0 as f32) * crate::grid::GRID_SIZE,
            (point.y.0 as f32) * crate::grid::GRID_SIZE,
        )
    }
}

// A blocked rectangle - inclusive of the edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl Block {
    fn expand_x(&self, delta_x: CoordX) -> Self {
        Block {
            top_left: point(self.top_left.x - delta_x, self.top_left.y),
            bottom_right: point(self.bottom_right.x + delta_x, self.bottom_right.y),
        }
    }
    pub fn spans_y(&self, y: CoordY) -> bool {
        self.top_left.y <= y && self.bottom_right.y >= y
    }
    pub fn spans_x(&self, x: CoordX) -> bool {
        self.top_left.x <= x && self.bottom_right.x >= x
    }
    pub fn is_left_of(&self, x: CoordX) -> bool {
        self.bottom_right.x < x
    }
    pub fn is_right_of(&self, x: CoordX) -> bool {
        self.top_left.x > x
    }
    pub fn is_above(&self, y: CoordY) -> bool {
        self.bottom_right.y < y
    }
    pub fn is_below(&self, y: CoordY) -> bool {
        self.top_left.y > y
    }
    pub fn contains(&self, point: Point) -> bool {
        self.spans_x(point.x) && self.spans_y(point.y)
    }
    fn intersects_edge(&self, edge: &RouteEdge) -> bool {
        // Check for intersection between the edge and the block.  The edge is
        // either horizontal or vertical, so we can check for intersection by comparing the coordinates.
        match edge.direction() {
            RouteDirection::Horizontal => {
                let end_point: Point = edge.end.into();
                let start_point: Point = edge.start.into();
                let min_x = start_point.x.min(end_point.x);
                let max_x = start_point.x.max(end_point.x);
                // The edge goes from [min_x,max_x], and we have the interval
                // [self.top_left.x, self.bottom_right.x] - the edge intersects the block if the intervals overlap.
                self.spans_y(start_point.y)
                    && interval_overlap(min_x, max_x, self.top_left.x, self.bottom_right.x)
            }
            RouteDirection::Vertical => {
                let end_point: Point = edge.end.into();
                let start_point: Point = edge.start.into();
                let min_y = start_point.y.min(end_point.y);
                let max_y = start_point.y.max(end_point.y);
                self.spans_x(start_point.x)
                    && interval_overlap(min_y, max_y, self.top_left.y, self.bottom_right.y)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelOrientation {
    Horizontal,
    Vertical,
}

// A routing channel has a seed coordinate and a cost.
// the seed coordinate is the point that anchors the
// channel, which can then be vertical or horizontal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel {
    pub seed: Point,
    pub cost: Cost,
    pub orientation: ChannelOrientation,
}

fn h_channel(seed: impl Into<Point>, cost: impl Into<Cost>) -> Channel {
    Channel {
        seed: seed.into(),
        cost: cost.into(),
        orientation: ChannelOrientation::Horizontal,
    }
}

fn v_channel(seed: impl Into<Point>, cost: impl Into<Cost>) -> Channel {
    Channel {
        seed: seed.into(),
        cost: cost.into(),
        orientation: ChannelOrientation::Vertical,
    }
}

// A linear segment is a start and end coordinate and a cost.
// A segment is either horizontal or vertical, and the cost is
// the cost of routing through that segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Segment<P> {
    pub start: P,
    pub end: P,
    pub cost: Cost,
}

type HSegment = Segment<CoordX>;
type VSegment = Segment<CoordY>;

fn hseg(start: impl Into<CoordX>, end: impl Into<CoordX>, cost: impl Into<Cost>) -> HSegment {
    HSegment {
        start: start.into(),
        end: end.into(),
        cost: cost.into(),
    }
}

fn vseg(start: impl Into<CoordY>, end: impl Into<CoordY>, cost: impl Into<Cost>) -> VSegment {
    VSegment {
        start: start.into(),
        end: end.into(),
        cost: cost.into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EventSense {
    Enter,
    Scan,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Event<T, C> {
    time: T,
    sense: EventSense,
    payload: C,
}

impl<T: Copy, C: Copy> Event<T, C> {
    fn t(&self) -> T {
        self.time
    }
    fn cost(&self) -> C {
        self.payload
    }
    fn count(&self) -> i32 {
        match self.sense {
            EventSense::Enter => 1,
            EventSense::Exit => -1,
            EventSense::Scan => 0,
        }
    }
    fn is_enter(&self) -> bool {
        matches!(self.sense, EventSense::Enter)
    }
    fn enter(time: T, payload: C) -> Self {
        Event {
            time,
            sense: EventSense::Enter,
            payload,
        }
    }
    fn exit(time: T, payload: C) -> Self {
        Event {
            time,
            sense: EventSense::Exit,
            payload,
        }
    }
    fn scan(time: T, payload: C) -> Self {
        Event {
            time,
            sense: EventSense::Scan,
            payload,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

const TURN_COST: Cost = Cost::new(25.0);
const MOVE_COST: Cost = Cost::new(1.0);
pub const WIRE_COST: Cost = Cost::new(10.0);

fn cross_cost(
    from: Option<Direction>,
    to: Direction,
    cost_to_cross_east_west: Cost,
    cost_to_cross_north_south: Cost,
) -> Cost {
    if let Some(from_dir) = from {
        match (from_dir, to) {
            (Direction::North, Direction::South) | (Direction::South, Direction::North) => {
                cost_to_cross_east_west
            }
            (Direction::East, Direction::West) | (Direction::West, Direction::East) => {
                cost_to_cross_north_south
            }
            _ => COST_ZERO,
        }
    } else {
        COST_ZERO
    }
}

fn turn_cost(from: Option<Direction>, to: Direction) -> Cost {
    if let Some(from_dir) = from {
        if from_dir == to {
            COST_ZERO
        } else {
            if to == from_dir.opposite() {
                TURN_COST * 100.0
            } else {
                TURN_COST
            }
        }
    } else {
        COST_ZERO
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct SearchState {
    node: NodeIndex,
    dir: Option<Direction>,
}

#[derive(Debug, Clone, Default)]
pub struct RouterNGBuilder {
    /// The blocking rectangles
    blocks: Vec<Block>,
    /// The routing channels
    channels: Vec<Channel>,
}

impl RouterNGBuilder {
    pub fn add_h_channel(&mut self, seed: impl Into<Point>, cost: impl Into<Cost>) {
        self.channels.push(h_channel(seed, cost));
    }
    pub fn add_v_channel(&mut self, seed: impl Into<Point>, cost: impl Into<Cost>) {
        self.channels.push(v_channel(seed, cost));
    }
    fn add_routing_moat(
        &mut self,
        top_left: Point,
        bottom_right: Point,
        distance: i32,
        cost: Cost,
    ) {
        let min_x = top_left.x.0.min(bottom_right.x.0);
        let max_x = top_left.x.0.max(bottom_right.x.0);
        let min_y = top_left.y.0.min(bottom_right.y.0);
        let max_y = top_left.y.0.max(bottom_right.y.0);
        self.add_v_channel(point(min_x - distance - 2, min_y), cost);
        self.add_v_channel(point(min_x - distance - 2, max_y), cost);
        self.add_v_channel(point(max_x + distance + 2, min_y), cost);
        self.add_v_channel(point(max_x + distance + 2, max_y), cost);
        self.add_h_channel(point(min_x, min_y - distance - 2), cost);
        self.add_h_channel(point(max_x, min_y - distance - 2), cost);
        self.add_h_channel(point(min_x, max_y + distance + 2), cost);
        self.add_h_channel(point(max_x, max_y + distance + 2), cost);
    }
    pub fn add_routable_point(&mut self, point: impl Into<Point>) {
        let point: Point = point.into();
        self.add_h_channel(point, COST_ZERO);
        self.add_v_channel(point, COST_ZERO);
    }
    pub fn add_block(&mut self, top_left: impl Into<Point>, bottom_right: impl Into<Point>) {
        let top_left: Point = top_left.into();
        let bottom_right: Point = bottom_right.into();
        let min_x = top_left.x.0.min(bottom_right.x.0);
        let max_x = top_left.x.0.max(bottom_right.x.0);
        let min_y = top_left.y.0.min(bottom_right.y.0);
        let max_y = top_left.y.0.max(bottom_right.y.0);
        let block = Block {
            top_left: point(min_x, min_y),
            bottom_right: point(max_x, max_y),
        };
        self.blocks.push(block);
        // Add the routing channels around the blocked rectangle.
        for moat_lane in 0..5 {
            let cost = if moat_lane == 0 {
                Cost::new(0.2)
            } else {
                Cost::new(0.1)
            };
            self.add_routing_moat(top_left, bottom_right, moat_lane, cost);
        }
    }
    pub fn build(self) -> RouterNG {
        let mut router = RouterNG {
            blocks: self.blocks,
            h_segments: BTreeMap::new(),
            v_segments: BTreeMap::new(),
            nodes: BTreeSet::new(),
            graph: UnGraph::default(),
            node_to_index: BTreeMap::new(),
            dirty: true,
        };
        for channel in self.channels {
            match channel.orientation {
                ChannelOrientation::Horizontal => {
                    router.seed_horiz_channel(channel.seed, channel.cost);
                }
                ChannelOrientation::Vertical => {
                    router.seed_vert_channel(channel.seed, channel.cost);
                }
            }
        }
        router.update();
        router
    }
}

#[derive(Debug, Clone)]
pub struct RouterNG {
    /// The blocking rectangles - not mutable
    blocks: Vec<Block>,
    /// Horizontal segments, keyed by their vertical coordinate
    h_segments: BTreeMap<CoordY, Vec<HSegment>>,
    /// Vertical segments, keyed by their horizontal coordinate
    v_segments: BTreeMap<CoordX, Vec<VSegment>>,
    /// Nodes: the intersection points of the segments
    nodes: BTreeSet<Point>,
    /// The graph to be used for pathfinding, built from the segments and nodes
    graph: UnGraph<Point, Cost>,
    /// A map from node to index in the graph, for quick lookup
    node_to_index: BTreeMap<Point, petgraph::graph::NodeIndex>,
    /// Dirty flag that indicates h_segments or v_segments have been modified and the graph needs to be rebuilt.
    dirty: bool,
}

impl RouterNG {
    pub fn debug_marks(&self) -> Vec<Mark> {
        assert!(
            !self.dirty,
            "Cannot generate debug marks when the graph is dirty"
        );
        let mut turtle = Turtle::default();
        for node in self.graph.node_indices() {
            let &pos = self.graph.node_weight(node).unwrap();
            turtle.move_to(pos.into());
            turtle.circle(3.0, egui::Color32::RED.gamma_multiply(0.2));
            // To make the edges more visible, we add a gap at the beginning and
            // end of the edge line, so that it looks like this * ---- * rather than this *------------------*
            for edge in self.graph.edges(node) {
                let target = edge.target();
                let &target_pos = self.graph.node_weight(target).unwrap();

                // Calculate direction and distance
                let start_pos: egui::Pos2 = pos.into();
                let end_pos: egui::Pos2 = target_pos.into();
                let dx = end_pos.x - start_pos.x;
                let dy = end_pos.y - start_pos.y;
                let distance = (dx * dx + dy * dy).sqrt();

                // Skip very short edges
                if distance < 8.0 {
                    continue;
                }

                // Create 4-pixel gap at each end
                let gap = 4.0;
                let gap_ratio = gap / distance;

                // Start point with gap from the node
                let line_start =
                    egui::Pos2::new(start_pos.x + dx * gap_ratio, start_pos.y + dy * gap_ratio);

                // End point with gap before the target
                let line_end =
                    egui::Pos2::new(end_pos.x - dx * gap_ratio, end_pos.y - dy * gap_ratio);

                turtle.move_to(line_start);
                turtle.line_to(
                    line_end,
                    (0.5, egui::Color32::RED.gamma_multiply(0.2)).into(),
                );
            }
        }
        turtle.compile()
    }
    pub fn is_route_blocked(&mut self, edges: &[RouteEdge]) -> bool {
        self.update();
        edges
            .iter()
            .any(|edge| self.blocks.iter().any(|block| block.intersects_edge(edge)))
    }
    pub fn is_accessible(&self, test: impl Into<Point>) -> bool {
        let test: Point = test.into();
        !self.blocks.iter().any(|block| block.contains(test))
    }
    pub fn add_existing_route(&mut self, edges: &[RouteEdge], cost: impl Into<Cost>) {
        let cost: Cost = cost.into();
        for edge in edges {
            let start: Point = edge.start.into();
            let end: Point = edge.end.into();
            match edge.direction() {
                RouteDirection::Horizontal => {
                    let left = start.x.min(end.x);
                    let right = start.x.max(end.x);
                    self.add_horiz_segment(start.y, left, right, cost);
                }
                RouteDirection::Vertical => {
                    let top = start.y.min(end.y);
                    let bottom = start.y.max(end.y);
                    self.add_vert_segment(start.x, top, bottom, cost);
                }
            }
        }
        self.update();
    }
    fn seed_horiz_channel(&mut self, center: impl Into<Point>, cost: impl Into<Cost>) {
        let center: Point = center.into();
        let cost: Cost = cost.into();
        let mut left_endpoint = NEG_INFINITY_X;
        let mut right_endpoint = INFINITY_X;
        for block in &self.blocks {
            let block = block.expand_x(CoordX(1));
            // Loop over the blocks.  For each block, if it intersects the horizontal
            // channel, then we update the left and right endpoints of the channel.
            // First, we test that the y-coordinate
            if block.spans_y(center.y) {
                if block.spans_x(center.x) {
                    // The span is blocked since the seed point of the
                    // span is in the middle of a block - reject it.
                    return;
                }
                if block.is_left_of(center.x) {
                    // The block is to the left of the center, so it can only affect the left endpoint.
                    left_endpoint = left_endpoint.max(block.bottom_right.x);
                }
                if block.is_right_of(center.x) {
                    // The block is to the right of the center, so it can only affect the right endpoint.
                    right_endpoint = right_endpoint.min(block.top_left.x);
                }
            }
        }
        // Add a horizontal segment for the channel if it is valid.
        if left_endpoint < right_endpoint {
            self.add_horiz_segment(center.y, left_endpoint, right_endpoint, cost);
        }
    }
    fn seed_vert_channel(&mut self, center: impl Into<Point>, cost: impl Into<Cost>) {
        let center: Point = center.into();
        let cost: Cost = cost.into();
        let mut top_endpoint = NEG_INFINITY_Y;
        let mut bottom_endpoint = INFINITY_Y;
        for block in &self.blocks {
            // Loop over the blocks.  For each block, if it intersects the vertical
            // channel, then we update the top and bottom endpoints of the channel.
            // First, we test that the x-coordinate - allow for a gutter on either
            // side of the block
            if block.expand_x(CoordX(1)).spans_x(center.x) {
                if block.spans_y(center.y) {
                    // The span is blocked since the seed point of the
                    // span is in the middle of a block - reject it.
                    return;
                }
                if block.is_above(center.y) {
                    // The block is above the center, so it can only affect the top endpoint.
                    top_endpoint = top_endpoint.max(block.bottom_right.y);
                }
                if block.is_below(center.y) {
                    // The block is below the center, so it can only affect the bottom endpoint.
                    bottom_endpoint = bottom_endpoint.min(block.top_left.y);
                }
            }
        }
        // Add a vertical segment for the channel if it is valid.
        if top_endpoint < bottom_endpoint {
            self.add_vert_segment(center.x, top_endpoint, bottom_endpoint, cost);
        }
    }
    fn seed_channels(&mut self, center: impl Into<Point>, cost: impl Into<Cost>) {
        let center: Point = center.into();
        let cost: Cost = cost.into();
        self.seed_horiz_channel(center, cost);
        self.seed_vert_channel(center, cost);
    }
    fn add_horiz_segment(
        &mut self,
        vert: impl Into<CoordY>,
        left: impl Into<CoordX>,
        right: impl Into<CoordX>,
        cost: impl Into<Cost>,
    ) {
        let vert: CoordY = vert.into();
        let left: CoordX = left.into();
        let right: CoordX = right.into();
        let cost: Cost = cost.into();
        if right > left {
            self.h_segments
                .entry(vert)
                .or_default()
                .push(hseg(left, right, cost));
            self.dirty = true;
        }
    }
    fn add_vert_segment(
        &mut self,
        horiz: impl Into<CoordX>,
        top: impl Into<CoordY>,
        bottom: impl Into<CoordY>,
        cost: impl Into<Cost>,
    ) {
        let horiz: CoordX = horiz.into();
        let top: CoordY = top.into();
        let bottom: CoordY = bottom.into();
        let cost: Cost = cost.into();
        if bottom > top {
            self.v_segments
                .entry(horiz)
                .or_default()
                .push(vseg(top, bottom, cost));
            self.dirty = true;
        }
    }
    fn update(&mut self) {
        if !self.dirty {
            return;
        }
        let h_segments = std::mem::take(&mut self.h_segments);
        for (vert, segments) in h_segments {
            normalize_collinear_segments(segments, |left, right, cost| {
                self.add_horiz_segment(vert, left, right, cost);
            });
        }
        let v_segments = std::mem::take(&mut self.v_segments);
        for (horiz, segments) in v_segments {
            normalize_collinear_segments(segments, |top, bottom, cost| {
                self.add_vert_segment(horiz, top, bottom, cost);
            });
        }
        self.nodes = collect_intersections(self.iter_hsegs(), self.iter_vsegs());
        // Re-segment, but now add segments for each node.
        let mut h_segments = std::mem::take(&mut self.h_segments);
        self.nodes.iter().for_each(|&node| {
            h_segments
                .entry(node.y)
                .or_default()
                .push(hseg(node.x, node.x, COST_ZERO));
        });
        for (vert, segments) in h_segments {
            normalize_collinear_segments(segments, |left, right, cost| {
                self.add_horiz_segment(vert, left, right, cost);
            });
        }
        let mut v_segments = std::mem::take(&mut self.v_segments);
        self.nodes.iter().for_each(|&node| {
            v_segments
                .entry(node.x)
                .or_default()
                .push(vseg(node.y, node.y, COST_ZERO));
        });
        for (horiz, segments) in v_segments {
            normalize_collinear_segments(segments, |top, bottom, cost| {
                self.add_vert_segment(horiz, top, bottom, cost);
            });
        }
        let mut nodes = collect_intersections(self.iter_hsegs(), self.iter_vsegs());
        nodes.extend(
            self.iter_hsegs()
                .flat_map(|(y, h_seg)| [point(h_seg.start, y), point(h_seg.end, y)])
                .chain(
                    self.iter_vsegs()
                        .flat_map(|(x, v_seg)| [point(x, v_seg.start), point(x, v_seg.end)]),
                ),
        );
        self.nodes = nodes;
        self.rebuild_graph();
        self.dirty = false;
    }
    fn iter_hsegs(&self) -> impl Iterator<Item = (CoordY, HSegment)> + '_ {
        self.h_segments
            .iter()
            .flat_map(|(&y, h_segs)| h_segs.iter().map(move |h_seg| (y, *h_seg)))
    }
    fn iter_vsegs(&self) -> impl Iterator<Item = (CoordX, VSegment)> + '_ {
        self.v_segments
            .iter()
            .flat_map(|(&x, v_segs)| v_segs.iter().map(move |v_seg| (x, *v_seg)))
    }
    fn rebuild_graph(&mut self) {
        let mut node_to_index: BTreeMap<Point, petgraph::graph::NodeIndex> = BTreeMap::new();
        let mut graph = UnGraph::default();
        for &node in &self.nodes {
            let index = graph.add_node(node);
            node_to_index.insert(node, index);
        }
        for hseg in self.iter_hsegs() {
            let y = hseg.0;
            let h_seg = hseg.1;
            let start_node = point(h_seg.start, y);
            let end_node = point(h_seg.end, y);
            graph.add_edge(
                node_to_index[&start_node],
                node_to_index[&end_node],
                h_seg.cost,
            );
        }
        for vseg in self.iter_vsegs() {
            let x = vseg.0;
            let v_seg = vseg.1;
            let start_node = point(x, v_seg.start);
            let end_node = point(x, v_seg.end);
            graph.add_edge(
                node_to_index[&start_node],
                node_to_index[&end_node],
                v_seg.cost,
            );
        }
        self.graph = graph;
        self.node_to_index = node_to_index;
    }
    pub fn reachable_neighbors(&mut self, point: impl Into<Point>) -> Vec<Point> {
        self.update();
        let point: Point = point.into();
        let Some(&node) = self.node_to_index.get(&point) else {
            return vec![];
        };
        self.graph
            .edges(node)
            .map(|edge| *self.graph.node_weight(edge.target()).unwrap())
            .collect()
    }
    fn successors(&self, state: &SearchState) -> Vec<(SearchState, Cost)> {
        let prev_dir = state.dir;
        let prev_point = *self.graph.node_weight(state.node).unwrap();
        let mut north_cost: Option<Cost> = None;
        let mut south_cost: Option<Cost> = None;
        let mut east_cost: Option<Cost> = None;
        let mut west_cost: Option<Cost> = None;
        // Get the costs to move in the 4 cardinal directions from the current node.
        for edge in self.graph.edges(state.node) {
            let neighbor = edge.target();
            let cost = *edge.weight();
            let neighbor_point = *self.graph.node_weight(neighbor).unwrap();
            if neighbor_point.x > prev_point.x {
                east_cost = Some(cost);
            } else if neighbor_point.x < prev_point.x {
                west_cost = Some(cost);
            } else if neighbor_point.y > prev_point.y {
                south_cost = Some(cost);
            } else {
                north_cost = Some(cost);
            };
        }
        // Calculate the east and west cost as a single cost,
        // since if we are north/south bound, we should consider
        // this a crossing.
        let east_west_crossing_cost = match (east_cost, west_cost) {
            (Some(east), Some(west)) => east.max(west),
            _ => COST_ZERO,
        };
        let north_south_crossing_cost = match (north_cost, south_cost) {
            (Some(north), Some(south)) => north.max(south),
            _ => COST_ZERO,
        };
        // Rescan the edges to generate the successors with the correct costs.
        self.graph
            .edges(state.node)
            .map(|edge| {
                let neighbor = edge.target();
                let cost = *edge.weight();
                let neighbor_point = *self.graph.node_weight(neighbor).unwrap();
                let dir = if neighbor_point.x > prev_point.x {
                    Direction::East
                } else if neighbor_point.x < prev_point.x {
                    Direction::West
                } else if neighbor_point.y > prev_point.y {
                    Direction::South
                } else {
                    Direction::North
                };
                let step_length = neighbor_point.manhattan_distance(prev_point) as f64;
                let step_cost = turn_cost(prev_dir, dir)
                    + MOVE_COST * step_length
                    + cost * step_length
                    + cross_cost(
                        prev_dir,
                        dir,
                        east_west_crossing_cost,
                        north_south_crossing_cost,
                    );
                (
                    SearchState {
                        node: neighbor,
                        dir: Some(dir),
                    },
                    step_cost,
                )
            })
            .collect()
    }
    fn path_find(&mut self, start: impl Into<Point>, end: impl Into<Point>) -> Option<Vec<Point>> {
        self.update();
        let start: Point = start.into();
        let end: Point = end.into();
        let &start_node = self.node_to_index.get(&start)?;
        let end_node = self.node_to_index.get(&end)?;
        let start = SearchState {
            node: start_node,
            dir: None,
        };
        let result = dijkstra(
            &start,
            |state| self.successors(state),
            |state| state.node == *end_node,
        );
        result.map(|(path, _cost)| {
            path.into_iter()
                .map(|state| *self.graph.node_weight(state.node).unwrap())
                .collect()
        })
    }
    fn path_find_with_fallback(
        &mut self,
        start: impl Into<Point>,
        end: impl Into<Point>,
    ) -> Vec<Point> {
        let start: Point = start.into();
        let end: Point = end.into();
        if let Some(path) = self.path_find(start, end) {
            return path;
        }
        // Couldn't find a path. so just connect the two points with a horizontal and vertical segment.
        vec![start, point(end.x, start.y), end]
    }
    pub fn waypoint_path<T>(
        &mut self,
        start: T,
        waypoints: &[Waypoint],
        head: T,
    ) -> Vec<TaggedPoint>
    where
        T: Into<Point> + Copy,
    {
        let mut path = Vec::new();
        self.seed_channels(start, COST_ZERO);
        if let Some(first_wp) = waypoints.first() {
            self.seed_channels(first_wp.pos, COST_ZERO);
            let subpath = self.path_find_with_fallback(start, first_wp.pos);
            path.extend(subpath.into_iter().map(|point| TaggedPoint {
                pos: point,
                segment: SegmentKind::StartToWaypoint(first_wp.id),
            }));
            for windows in waypoints.windows(2) {
                let wp_start = windows[0];
                let wp_end = windows[1];
                self.seed_channels(wp_end.pos, COST_ZERO);
                let subpath = self.path_find_with_fallback(wp_start, wp_end);
                path.extend(subpath.into_iter().map(|point| TaggedPoint {
                    segment: SegmentKind::WaypointToWaypoint(wp_start.id, wp_end.id),
                    pos: point,
                }));
            }
            let last_wp = waypoints.last().unwrap_or(first_wp);
            self.seed_channels(head, COST_ZERO);
            let subpath = self.path_find_with_fallback(last_wp.pos, head);
            path.extend(subpath.into_iter().map(|point| TaggedPoint {
                pos: point,
                segment: SegmentKind::WaypointToEnd(last_wp.id),
            }));
            path
        } else {
            self.seed_channels(head, COST_ZERO);
            let subpath = self.path_find_with_fallback(start, head);
            subpath
                .into_iter()
                .map(|point| TaggedPoint {
                    pos: point,
                    segment: SegmentKind::StartToEnd,
                })
                .collect()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SegmentKind {
    StartToWaypoint(WaypointId),
    WaypointToWaypoint(WaypointId, WaypointId),
    WaypointToEnd(WaypointId),
    StartToEnd,
}

impl SegmentKind {
    pub fn is_wp_to_wp(&self) -> bool {
        matches!(self, SegmentKind::WaypointToWaypoint(_, _))
    }
}

#[derive(Copy, Clone)]
pub struct TaggedPoint {
    pub segment: SegmentKind,
    pub pos: Point,
}

impl std::fmt::Debug for TaggedPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.segment {
            SegmentKind::StartToEnd => write!(f, "s->e {}", self.pos),
            SegmentKind::StartToWaypoint(wp) => write!(f, "s->wp[{}] {}", wp, self.pos),
            SegmentKind::WaypointToWaypoint(wp0, wp1) => {
                write!(f, "wp[{}]->wp[{}] {}", wp0, wp1, self.pos)
            }
            SegmentKind::WaypointToEnd(wp) => write!(f, "wp[{}] -> e {}", wp, self.pos),
        }
    }
}

// Run a line-sweep stype algorithm to collect the intersections.
// The algorithm works by creating a list of events sorted in x.  Each event
// is either the start or end of a horizontal segment (using the Enter/Exit events)
// or a vertical segment (using the Scan event).  The events are sorted by their X-coordinate,
// and then processed in order.  We maintain a list of active horizontal segments at any given
// time, and then when we encounter a scan event, we list out all intersections of that vertical
// segment with the active horizontal segments.
fn collect_intersections(
    h_segments: impl IntoIterator<Item = (CoordY, HSegment)>,
    v_segments: impl IntoIterator<Item = (CoordX, VSegment)>,
) -> BTreeSet<Point> {
    let mut events: Vec<Event<CoordX, (CoordY, CoordY)>> = h_segments
        .into_iter()
        .flat_map(|(y, h_seg)| {
            [
                Event::enter(h_seg.start, (y, y)),
                Event::exit(h_seg.end, (y, y)),
            ]
        })
        .chain(
            v_segments
                .into_iter()
                .map(|(x, v_seg)| Event::scan(x, (v_seg.start, v_seg.end))),
        )
        .collect::<Vec<_>>();
    events.sort();
    let mut intersections = BTreeSet::new();
    // Use a map to count active segments at each y-coordinate
    // This handles segments that touch at boundaries (e.g., one ends at x=10, another starts at x=10)
    let mut active_h_segments: BTreeMap<CoordY, usize> = BTreeMap::new();
    for event in events {
        match event.sense {
            EventSense::Enter => {
                let y = event.cost().0;
                *active_h_segments.entry(y).or_insert(0) += 1;
            }
            EventSense::Exit => {
                let y = event.cost().0;
                if let Some(count) = active_h_segments.get_mut(&y) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        active_h_segments.remove(&y);
                    }
                }
            }
            EventSense::Scan => {
                let (start, end) = event.cost();
                // Check all y-coordinates with non-zero count (active segments)
                for (&y, &count) in &active_h_segments {
                    if count > 0 && y >= start && y <= end {
                        intersections.insert(point(event.t(), y));
                    }
                }
            }
        }
    }
    intersections
}

fn normalize_collinear_segments<T: Ord + Copy>(
    segments: impl IntoIterator<Item = Segment<T>>,
    mut maker: impl FnMut(T, T, Cost),
) {
    let mut events = segments
        .into_iter()
        .flat_map(|seg| {
            [
                Event::enter(seg.start, seg.cost),
                Event::exit(seg.end, seg.cost),
            ]
        })
        .collect::<Vec<_>>();
    // Sort the events by their coordinate, with Enter events before Exit events in case of ties.
    events.sort();
    scan_disjoint_segments(events, |start, end, cost| {
        maker(start, end, cost);
    });
}

fn scan_disjoint_segments<T: Ord + Copy>(
    events: impl IntoIterator<Item = Event<T, Cost>>,
    mut maker: impl FnMut(T, T, Cost),
) {
    let mut events_iter = events.into_iter();

    // Handle the first event to initialize state
    let Some(first_event) = events_iter.next() else {
        return;
    };

    let mut last_t = first_event.t();
    let mut line_count = first_event.count();
    let mut current_cost = if first_event.is_enter() {
        first_event.cost()
    } else {
        COST_ZERO - first_event.cost()
    };

    // Process remaining events
    for event in events_iter {
        let t = event.t();
        // Invariant: line_count > 0 means last_t was assigned in a previous iteration
        if line_count != 0 {
            maker(last_t, t, current_cost);
        }
        last_t = t;
        line_count += event.count();
        current_cost = if event.is_enter() {
            current_cost + event.cost()
        } else {
            current_cost - event.cost()
        };
    }
}

fn interval_overlap<T: Ord>(a_start: T, a_end: T, b_start: T, b_end: T) -> bool {
    a_start < b_end && b_start < a_end
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! hseg {
        (y=$y:expr, [$(($start:expr => $end:expr, $cost:expr)),* $(,)?]) => {
            BTreeMap::from([(
                CoordY($y),
                vec![
                    $(HSegment {
                        start: CoordX($start),
                        end: CoordX($end),
                        cost: $cost.into(),
                    }),*
                ]
            )])
        };
    }

    macro_rules! vseg {
        (x=$x:expr, [$(($start:expr => $end:expr, $cost:expr)),* $(,)?]) => {
            BTreeMap::from([(
                CoordX($x),
                vec![
                    $(VSegment {
                        start: CoordY($start),
                        end: CoordY($end),
                        cost: $cost.into(),
                    }),*
                ]
            )])
        };
    }

    // Brute force algorithm
    fn collect_intersections_brute_force(
        h_segments: impl IntoIterator<Item = (CoordY, HSegment)>,
        v_segments: impl IntoIterator<Item = (CoordX, VSegment)>,
    ) -> Vec<Point> {
        let mut points = vec![];
        let v_segments = v_segments.into_iter().collect::<Vec<_>>();
        for (y, hseg) in h_segments.into_iter() {
            for (x, vseg) in &v_segments {
                if hseg.start <= *x && hseg.end >= *x && vseg.start <= y && vseg.end >= y {
                    points.push(point(*x, y));
                }
            }
        }
        points
    }

    #[test]
    fn test_vseed() {
        let mut router = RouterNGBuilder::default().build();
        router.seed_vert_channel(point(0, 0), 1.0);
        router.update();
        assert_eq!(
            router.v_segments,
            BTreeMap::from([(CoordX(0), vec![vseg(NEG_INFINITY_Y, INFINITY_Y, 1.0)])])
        );
    }

    #[test]
    fn test_normalize() {
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 10, 1.0);
        router.add_horiz_segment(0, 5, 15, 2.0);
        router.update();
        assert_eq!(
            router.h_segments,
            hseg!(y=0, [(0=>5, 1.0), (5=>10, 3.0), (10=>15, 2.0)])
        );
    }

    #[test]
    fn test_normalize_complete_overlap() {
        // One segment completely contains another
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 20, 1.0);
        router.add_horiz_segment(0, 5, 15, 2.0);
        router.update();
        assert_eq!(
            router.h_segments,
            hseg!(y=0, [(0=>5, 1.0), (5=>15, 3.0), (15=>20, 1.0)])
        );
    }

    #[test]
    fn test_normalize_no_overlap() {
        // Segments don't overlap at all
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 10, 1.0);
        router.add_horiz_segment(0, 20, 30, 2.0);
        router.update();
        assert_eq!(router.h_segments, hseg!(y=0, [(0=>10, 1.0), (20=>30, 2.0)]));
    }

    #[test]
    fn test_normalize_adjacent_segments() {
        // Segments touch at endpoints but don't overlap
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 10, 1.0);
        router.add_horiz_segment(0, 10, 20, 2.0);
        router.update();
        assert_eq!(router.h_segments, hseg!(y=0, [(0=>10, 1.0), (10=>20, 2.0)]));
    }

    #[test]
    fn test_normalize_triple_overlap() {
        // Three segments with various overlaps
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 15, 1.0);
        router.add_horiz_segment(0, 5, 20, 2.0);
        router.add_horiz_segment(0, 10, 25, 3.0);
        router.update();
        assert_eq!(
            router.h_segments,
            hseg!(y=0, [(0=>5, 1.0), (5=>10, 3.0), (10=>15, 6.0), (15=>20, 5.0), (20=>25, 3.0)])
        );
    }

    #[test]
    fn test_normalize_multiple_rows() {
        // Segments on different rows should be handled independently
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 10, 1.0);
        router.add_horiz_segment(0, 5, 15, 2.0);
        router.add_horiz_segment(5, 0, 10, 3.0);
        router.add_horiz_segment(5, 5, 15, 4.0);
        router.update();

        let mut expected = BTreeMap::new();
        expected.extend(hseg!(y=0, [(0=>5, 1.0), (5=>10, 3.0), (10=>15, 2.0)]));
        expected.extend(hseg!(y=5, [(0=>5, 3.0), (5=>10, 7.0), (10=>15, 4.0)]));
        assert_eq!(router.h_segments, expected);
    }

    #[test]
    fn test_normalize_negative_coords() {
        // Segments in negative coordinate space
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(-5, -20, -10, 1.0);
        router.add_horiz_segment(-5, -15, -5, 2.0);
        router.update();
        assert_eq!(
            router.h_segments,
            hseg!(y=-5, [(-20 => -15, 1.0), (-15 => -10, 3.0), (-10 => -5, 2.0)])
        );
    }

    #[test]
    fn test_normalize_vertical_segments() {
        // Test vertical segment normalization
        let mut router = RouterNGBuilder::default().build();
        router.add_vert_segment(0, 0, 10, 1.0);
        router.add_vert_segment(0, 5, 15, 2.0);
        router.update();
        assert_eq!(
            router.v_segments,
            vseg!(x=0, [(0=>5, 1.0), (5=>10, 3.0), (10=>15, 2.0)])
        );
    }

    #[test]
    fn test_normalize_identical_segments() {
        // Same segment added twice
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 0, 10, 1.0);
        router.add_horiz_segment(0, 0, 10, 1.0);
        router.update();
        assert_eq!(router.h_segments, hseg!(y=0, [(0=>10, 2.0)]));
    }

    #[test]
    fn test_normalize_reverse_order() {
        // Segments added in decreasing coordinate order
        let mut router = RouterNGBuilder::default().build();
        router.add_horiz_segment(0, 20, 30, 1.0);
        router.add_horiz_segment(0, 10, 25, 2.0);
        router.add_horiz_segment(0, 0, 15, 3.0);
        router.update();
        assert_eq!(
            router.h_segments,
            hseg!(y=0, [(0=>10, 3.0), (10=>15, 5.0), (15=>20, 2.0), (20=>25, 3.0), (25=>30, 1.0)])
        );
    }

    // Tests for collect_intersections function

    #[test]
    fn test_collect_intersections_no_intersections() {
        // Horizontal segments with no vertical intersections
        let h_segs = vec![
            (CoordY(0), hseg(0, 10, 1.0)),
            (CoordY(5), hseg(15, 25, 1.0)),
        ];
        // Vertical segment outside the y-range of all horizontal segments
        let v_segs = vec![(CoordX(20), vseg(10, 15, 1.0))];
        let intersections = collect_intersections(h_segs, v_segs);
        assert_eq!(intersections, BTreeSet::new());
    }

    #[test]
    fn test_collect_intersections_at_boundaries() {
        // Intersections at segment boundaries (start/end)
        let h_segs = vec![(CoordY(5), hseg(0, 10, 1.0))];
        // Vertical segment at the start of horizontal segment
        let v_segs_start = vec![(CoordX(0), vseg(0, 10, 1.0))];
        let intersections = collect_intersections(h_segs.clone(), v_segs_start);
        assert_eq!(intersections, BTreeSet::from([point(0, 5)]));

        // Vertical segment at the end of horizontal segment
        let v_segs_end = vec![(CoordX(10), vseg(0, 10, 1.0))];
        let intersections = collect_intersections(h_segs.clone(), v_segs_end);
        assert_eq!(intersections, BTreeSet::from([point(10, 5)]));

        // Vertical segment spanning from horizontal's y-coordinate exactly
        let v_segs_y_start = vec![(CoordX(5), vseg(5, 15, 1.0))];
        let intersections = collect_intersections(h_segs.clone(), v_segs_y_start);
        assert_eq!(intersections, BTreeSet::from([point(5, 5)]));

        // Vertical segment ending at horizontal's y-coordinate exactly
        let v_segs_y_end = vec![(CoordX(5), vseg(0, 5, 1.0))];
        let intersections = collect_intersections(h_segs, v_segs_y_end);
        assert_eq!(intersections, BTreeSet::from([point(5, 5)]));
    }

    #[test]
    fn test_collect_intersections_corners() {
        // Corner case: vertical start == horizontal start
        let h_segs = vec![(CoordY(5), hseg(10, 20, 1.0))];
        let v_segs = vec![(CoordX(10), vseg(5, 15, 1.0))];
        let intersections = collect_intersections(h_segs, v_segs);
        assert_eq!(intersections, BTreeSet::from([point(10, 5)]));
    }

    #[test]
    fn test_collect_intersections_corner_all_endpoints() {
        // All four corners: (h.start, v.start), (h.start, v.end), (h.end, v.start), (h.end, v.end)
        let h_seg_y = CoordY(10);
        let h_segs = vec![(h_seg_y, hseg(5, 15, 1.0))];

        // Test (h.start, v.start) corner
        let v_segs = vec![(CoordX(5), vseg(10, 20, 1.0))];
        let intersections = collect_intersections(h_segs.clone(), v_segs);
        assert_eq!(intersections, BTreeSet::from([point(5, 10)]));

        // Test (h.start, v.end) corner
        let v_segs = vec![(CoordX(5), vseg(0, 10, 1.0))];
        let intersections = collect_intersections(h_segs.clone(), v_segs);
        assert_eq!(intersections, BTreeSet::from([point(5, 10)]));

        // Test (h.end, v.start) corner
        let v_segs = vec![(CoordX(15), vseg(10, 20, 1.0))];
        let intersections = collect_intersections(h_segs.clone(), v_segs);
        assert_eq!(intersections, BTreeSet::from([point(15, 10)]));

        // Test (h.end, v.end) corner
        let v_segs = vec![(CoordX(15), vseg(0, 10, 1.0))];
        let intersections = collect_intersections(h_segs, v_segs);
        assert_eq!(intersections, BTreeSet::from([point(15, 10)]));
    }

    #[test]
    fn test_collect_intersections_multiple() {
        // Multiple intersections from a single vertical segment crossing multiple horizontal segments
        let h_segs = vec![
            (CoordY(5), hseg(0, 20, 1.0)),
            (CoordY(10), hseg(0, 20, 1.0)),
            (CoordY(15), hseg(0, 20, 1.0)),
        ];
        let v_segs = vec![(CoordX(10), vseg(0, 20, 1.0))];
        let intersections = collect_intersections(h_segs, v_segs);
        assert_eq!(
            intersections,
            BTreeSet::from([point(10, 5), point(10, 10), point(10, 15)])
        );
    }

    #[test]
    fn test_collect_intersections_multiple_verticals() {
        // Single horizontal segment crossing multiple vertical segments
        let h_segs = vec![(CoordY(10), hseg(0, 30, 1.0))];
        let v_segs = vec![
            (CoordX(5), vseg(5, 15, 1.0)),
            (CoordX(15), vseg(5, 15, 1.0)),
            (CoordX(25), vseg(5, 15, 1.0)),
        ];
        let intersections = collect_intersections(h_segs, v_segs);
        assert_eq!(
            intersections,
            BTreeSet::from([point(5, 10), point(15, 10), point(25, 10)])
        );
    }

    #[test]
    fn test_collect_intersections_vertical_outside_horizontal_y_range() {
        // Vertical segment exists in x-range of horizontal but y is outside
        let h_segs = vec![(CoordY(10), hseg(0, 20, 1.0))];
        let v_segs = vec![(CoordX(10), vseg(15, 25, 1.0))];
        let intersections = collect_intersections(h_segs, v_segs);
        assert_eq!(intersections, BTreeSet::new());
    }

    #[test]
    fn test_collect_intersections_empty_inputs() {
        // Empty horizontal segments
        let intersections = collect_intersections(vec![], vec![(CoordX(5), vseg(0, 10, 1.0))]);
        assert_eq!(intersections, BTreeSet::new());

        // Empty vertical segments
        let intersections = collect_intersections(vec![(CoordY(5), hseg(0, 10, 1.0))], vec![]);
        assert_eq!(intersections, BTreeSet::new());

        // Both empty
        let intersections: BTreeSet<Point> = collect_intersections(
            Vec::<(CoordY, HSegment)>::new(),
            Vec::<(CoordX, VSegment)>::new(),
        );
        assert_eq!(intersections, BTreeSet::new());
    }

    #[test]
    fn test_random_segments_line_sweep_matches_brute_force() {
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};

        // Use a fixed seed for reproducibility
        let mut rng = StdRng::seed_from_u64(42);

        const NUM_H_SEGMENTS: usize = 1000;
        const NUM_V_SEGMENTS: usize = 1000;
        const FIELD_SIZE: i32 = 200;

        let mut router = RouterNGBuilder::default().build();

        // Generate random horizontal segments
        for _ in 0..NUM_H_SEGMENTS {
            let y = rng.random_range(0..FIELD_SIZE);
            let x1 = rng.random_range(0..FIELD_SIZE);
            let x2 = rng.random_range(0..FIELD_SIZE);
            let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
            // Ensure non-zero length segments
            if start < end {
                router.add_horiz_segment(y, start, end, rng.random_range(0.1..10.0));
            }
        }

        // Generate random vertical segments
        for _ in 0..NUM_V_SEGMENTS {
            let x = rng.random_range(0..FIELD_SIZE);
            let y1 = rng.random_range(0..FIELD_SIZE);
            let y2 = rng.random_range(0..FIELD_SIZE);
            let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
            // Ensure non-zero length segments
            if start < end {
                router.add_vert_segment(x, start, end, rng.random_range(0.1..10.0));
            }
        }

        // Normalize segments (merge overlapping segments)
        let normalize_start = std::time::Instant::now();
        router.update();
        let normalize_time = normalize_start.elapsed();

        println!("Normalization took: {:?}", normalize_time);
        println!(
            "Normalized to {} horizontal segments and {} vertical segments",
            router.h_segments.values().map(|v| v.len()).sum::<usize>(),
            router.v_segments.values().map(|v| v.len()).sum::<usize>()
        );

        // Collect intersections using line-sweep algorithm
        let line_sweep_start = std::time::Instant::now();
        let line_sweep_intersections =
            collect_intersections(router.iter_hsegs(), router.iter_vsegs());
        let line_sweep_time = line_sweep_start.elapsed();

        println!("Line-sweep algorithm took: {:?}", line_sweep_time);
        println!("Found {} intersections", line_sweep_intersections.len());

        // Collect intersections using brute force algorithm
        let brute_force_start = std::time::Instant::now();
        let brute_force_intersections: BTreeSet<Point> =
            collect_intersections_brute_force(router.iter_hsegs(), router.iter_vsegs())
                .into_iter()
                .collect();
        let brute_force_time = brute_force_start.elapsed();

        println!("Brute-force algorithm took: {:?}", brute_force_time);

        let speedup = brute_force_time.as_secs_f64() / line_sweep_time.as_secs_f64();
        println!("Line-sweep is {:.2}x faster than brute-force", speedup);

        // Compare results
        assert_eq!(
            line_sweep_intersections.len(),
            brute_force_intersections.len(),
            "Number of intersections differs: line-sweep found {}, brute-force found {}",
            line_sweep_intersections.len(),
            brute_force_intersections.len()
        );

        assert_eq!(
            line_sweep_intersections, brute_force_intersections,
            "Intersection sets differ between line-sweep and brute-force algorithms"
        );
    }
}
