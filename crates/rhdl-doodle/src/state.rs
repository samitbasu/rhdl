use egui::{CursorIcon, Pos2, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, LINE_RADIUS, MIN_TEXT_EDGE_LENGTH, ROUTE_TEXT_SIZE, SHIM, snap_to_grid},
    label::LabelId,
    rectbox::{LineAnchor, RectId},
    router_ng::{Point, SegmentKind, TaggedPoint},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResizeMode {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
    CenterTop,
    CenterBottom,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub enum State {
    Idle,
    Panning,
    #[default]
    AddText,
    AddTextHoveredRoute(AddTextHoveredRoute),
    AddingRect(AddingRect),
    MovingRect(MovingRect),
    Selected(Selected),
    PotentialResize(PotentialResize),
    ResizingRect(ResizingRect),
    EditingName(EditingName),
    PortDragged(PortDragged),
    PortLabelHovered(PortLabelHovered),
    PortLabelGripHovered(PortLabelGripHovered),
    EditingLabelText(EditingLabelText),
    PortPinHovered(PortPinHovered),
    InProgressAutoRoute(InProgressAutoRoute),
    ProposedAutoRoute(ProposedAutoRoute),
    RouteHovered(RouteHovered),
    RouteSelected(RouteSelected),
    RouteEdgeHovered(RouteEdgeHovered),
    RouteEdgeDragged(RouteEdgeDragged),
    RouteCornerHovered(RouteCornerHovered),
    WaypointHovered(WaypointHovered),
    WaypointDragged(WaypointDragged),
    RouteLabelHovered(RouteLabelHovered),
    EditingRouteLabelText(EditingRouteLabelText),
    AddTextButtonHovered(AddTextButtonHovered),
}

impl State {
    pub fn idle() -> Self {
        State::Idle
    }
    pub fn panning() -> Self {
        State::Panning
    }
    pub fn adding_rect(start_pos: Pos2, end_pos: Pos2) -> Self {
        State::AddingRect(AddingRect { start_pos, end_pos })
    }
    pub fn moving_rect(rect: RectId, delta_pos: Vec2) -> Self {
        State::MovingRect(MovingRect { rect, delta_pos })
    }
    pub fn selected(rect: RectId) -> Self {
        State::Selected(Selected { rect })
    }
    pub fn potential_resize(rect: RectId, mode: ResizeMode) -> Self {
        State::PotentialResize(PotentialResize { rect, mode })
    }
    pub fn resizing_rect(rect: RectId, mode: ResizeMode, delta_pos: Vec2) -> Self {
        State::ResizingRect(ResizingRect {
            rect,
            mode,
            delta_pos,
        })
    }
    pub fn editing_name(rect: RectId) -> Self {
        State::EditingName(EditingName { rect })
    }
    pub fn port_dragged(rect: RectId, label: LabelId, delta_pos: Vec2) -> Self {
        State::PortDragged(PortDragged {
            rect,
            label,
            delta_pos,
        })
    }
    pub fn port_label_hovered(rect: RectId, label: LabelId) -> Self {
        State::PortLabelHovered(PortLabelHovered { rect, label })
    }
    pub fn port_label_grip_hovered(rect: RectId, label: LabelId) -> Self {
        State::PortLabelGripHovered(PortLabelGripHovered { rect, label })
    }
    pub fn editing_label_text(rect: RectId, label: LabelId) -> Self {
        State::EditingLabelText(EditingLabelText { rect, label })
    }
    pub fn port_pin_hovered(rect: RectId, label: LabelId) -> Self {
        State::PortPinHovered(PortPinHovered { rect, label })
    }
}

impl From<RouteHovered> for State {
    fn from(value: RouteHovered) -> Self {
        State::RouteHovered(value)
    }
}

impl From<RouteSelected> for State {
    fn from(value: RouteSelected) -> Self {
        State::RouteSelected(value)
    }
}
impl From<RouteEdgeHovered> for State {
    fn from(value: RouteEdgeHovered) -> Self {
        State::RouteEdgeHovered(value)
    }
}
impl From<RouteCornerHovered> for State {
    fn from(value: RouteCornerHovered) -> Self {
        State::RouteCornerHovered(value)
    }
}
impl From<InProgressAutoRoute> for State {
    fn from(value: InProgressAutoRoute) -> Self {
        State::InProgressAutoRoute(value)
    }
}

impl From<ProposedAutoRoute> for State {
    fn from(value: ProposedAutoRoute) -> Self {
        State::ProposedAutoRoute(value)
    }
}

impl From<WaypointHovered> for State {
    fn from(value: WaypointHovered) -> Self {
        State::WaypointHovered(value)
    }
}

impl From<RouteEdgeDragged> for State {
    fn from(value: RouteEdgeDragged) -> Self {
        State::RouteEdgeDragged(value)
    }
}

impl From<RouteLabelHovered> for State {
    fn from(value: RouteLabelHovered) -> Self {
        State::RouteLabelHovered(value)
    }
}

impl From<EditingRouteLabelText> for State {
    fn from(value: EditingRouteLabelText) -> Self {
        State::EditingRouteLabelText(value)
    }
}

impl From<WaypointDragged> for State {
    fn from(value: WaypointDragged) -> Self {
        State::WaypointDragged(value)
    }
}

impl From<AddTextButtonHovered> for State {
    fn from(value: AddTextButtonHovered) -> Self {
        State::AddTextButtonHovered(value)
    }
}

impl From<AddTextHoveredRoute> for State {
    fn from(value: AddTextHoveredRoute) -> Self {
        State::AddTextHoveredRoute(value)
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct AddingRect {
    pub start_pos: Pos2,
    pub end_pos: Pos2,
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct MovingRect {
    pub rect: RectId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Selected {
    pub rect: RectId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PotentialResize {
    pub rect: RectId,
    pub mode: ResizeMode,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingName {
    pub rect: RectId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteEdgeDragged {
    pub id: RouteId,
    pub direction: RouteDirection,
    pub start_waypoint: WaypointId,
    pub end_waypoint: WaypointId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortDragged {
    pub rect: RectId,
    pub label: LabelId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortLabelHovered {
    pub rect: RectId,
    pub label: LabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteControlPointHovered {
    pub id: RouteId,
    pub edge: EdgeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteControlPointDragged {
    pub id: RouteId,
    pub edge: EdgeId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteLabelHovered {
    pub id: RouteId,
    pub edge_index: EdgeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingRouteLabelText {
    pub id: RouteId,
    pub label_id: WireLabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortLabelGripHovered {
    pub rect: RectId,
    pub label: LabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingLabelText {
    pub rect: RectId,
    pub label: LabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortPinHovered {
    pub rect: RectId,
    pub label: LabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResizingRect {
    pub rect: RectId,
    pub mode: ResizeMode,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AddTextButtonHovered {
    pub route: RouteId,
    pub edge_id: EdgeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AddTextHoveredRoute {
    pub route: RouteId,
    pub edge_id: EdgeId,
    pub pos: Pos2,
}

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub struct EdgeId(usize);

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub struct WaypointId(usize);

impl From<usize> for WaypointId {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

impl std::fmt::Display for WaypointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub struct WireLabelId(usize);

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub enum RouteDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteEdge {
    pub id: EdgeId,
    pub start: Pos2,
    pub end: Pos2,
    pub kind: SegmentKind,
}

impl RouteEdge {
    pub fn distance(&self, pos: Pos2) -> f32 {
        let start = self.start;
        let end = self.end;
        let line_vec = end - start;
        let line_len = line_vec.length();
        if line_len == 0.0 {
            return (pos - start).length();
        }
        let t = ((pos - start).dot(line_vec) / line_len.powi(2)).clamp(0.0, 1.0);
        let projection = start + line_vec * t;
        (pos - projection).length()
    }
    pub fn direction(&self) -> RouteDirection {
        if (self.end.x - self.start.x).abs() > (self.end.y - self.start.y).abs() {
            RouteDirection::Horizontal
        } else {
            RouteDirection::Vertical
        }
    }
    pub fn length(&self) -> f32 {
        (self.end - self.start).length()
    }
    pub fn text_anchor(&self) -> Option<Pos2> {
        if self.direction() == RouteDirection::Horizontal && self.length() >= MIN_TEXT_EDGE_LENGTH {
            Some(pos2((self.start.x + self.end.x) / 2.0, self.start.y - SHIM))
        } else {
            None
        }
    }
    pub fn center(&self) -> Pos2 {
        self.start + (self.end - self.start) * 0.5
    }
    pub fn waypoint_position_start(&self) -> Pos2 {
        // Avoid the routing gutters.
        let dir = (self.end - self.start).normalized();
        match self.kind {
            SegmentKind::WaypointToWaypoint(_, _) => self.start,
            SegmentKind::StartToWaypoint(_) | SegmentKind::StartToEnd => {
                self.start + dir * GRID_SIZE
            }
            SegmentKind::WaypointToEnd(_) => self.start,
        }
    }
    pub fn waypoint_position_end(&self) -> Pos2 {
        // Avoid the routing gutters.
        let dir = (self.end - self.start).normalized();
        match self.kind {
            SegmentKind::WaypointToWaypoint(_, _) => self.end,
            SegmentKind::StartToWaypoint(_) => self.end,
            SegmentKind::WaypointToEnd(_) | SegmentKind::StartToEnd => self.end - dir * GRID_SIZE,
        }
    }
}

pub fn next_waypoint_id(waypoints: &[Waypoint]) -> WaypointId {
    waypoints
        .iter()
        .map(|wp| wp.id)
        .max()
        .map(|id| WaypointId(id.0 + 1))
        .unwrap_or(WaypointId(0))
}

#[derive(Clone, PartialEq, Debug)]
pub struct InProgressAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Vec<Waypoint>,
    pub head: Pos2,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProposedAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Vec<Waypoint>,
    pub finish: LineAnchor,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WaypointHovered {
    pub route: RouteId,
    pub waypoint: WaypointId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WaypointDragged {
    pub route: RouteId,
    pub waypoint: WaypointId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Hash, Copy, Debug, Eq, PartialOrd, Ord, Default)]
pub struct RouteId(usize);

impl From<usize> for RouteId {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

impl RouteId {
    pub fn next(&self) -> Self {
        RouteId(self.0 + 1)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteHovered {
    pub id: RouteId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteSelected {
    pub id: RouteId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteEdgeHovered {
    pub id: RouteId,
    pub edge_index: EdgeId,
    pub direction: RouteDirection,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteCornerHovered {
    pub id: RouteId,
    pub edge_1: EdgeId,
    pub edge_2: EdgeId,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Waypoint {
    pub pos: Pos2,
    pub id: WaypointId,
    pub locked: bool,
}

impl Waypoint {
    pub fn unlock(&mut self) {
        self.locked = false;
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl From<Waypoint> for Point {
    fn from(val: Waypoint) -> Self {
        val.pos.into()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WireLabel {
    pub id: WireLabelId,
    pub position: f32,
    pub text: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AutoRoute {
    pub start: LineAnchor,
    pub edges: Vec<RouteEdge>,
    pub finish: LineAnchor,
    pub start_pos: Pos2,
    pub end_pos: Pos2,
    pub waypoints: Vec<Waypoint>,
    pub labels: Vec<WireLabel>,
}

fn next_edge_id(edges: &[RouteEdge]) -> EdgeId {
    edges
        .iter()
        .map(|edge| edge.id)
        .max()
        .map(|id| EdgeId(id.0 + 1))
        .unwrap_or(EdgeId(0))
}

pub struct LocAndDirection {
    pub location: Pos2,
    pub direction: RouteDirection,
}

impl AutoRoute {
    // Convert a distance along the route to a point on the route.  This is the inverse
    // of distance_along_route.  If the distance is out of range, take the end anchor.
    pub fn map_position(&self, position: f32) -> LocAndDirection {
        let total_distance = self.edges.iter().map(|edge| edge.length()).sum::<f32>();
        let mut distance = position * total_distance;
        for edge in &self.edges {
            if edge.length() < distance {
                distance -= edge.length();
            } else {
                let frac = distance / edge.length();
                return LocAndDirection {
                    location: edge.start + frac * (edge.end - edge.start),
                    direction: edge.direction(),
                };
            }
        }
        LocAndDirection {
            location: self.end_pos,
            direction: RouteDirection::Horizontal,
        }
    }
    // Calculate the distance along the route to reach point closest to the provided
    // position.
    fn distance_along_route(&self, pos: Pos2) -> f32 {
        let total_distance = self.edges.iter().map(|edge| edge.length()).sum::<f32>();
        let mut distance = 0.0;
        for edge in &self.edges {
            let edge_distance = edge.distance(pos);
            if edge_distance <= LINE_RADIUS {
                // The point is close enough to this edge, so we calculate the distance along the route to this point.
                let start_to_pos = (pos - edge.start).length();
                let start_to_end = (edge.end - edge.start).length();
                if start_to_end > 0.0 {
                    distance += start_to_pos.min(start_to_end);
                }
                break;
            } else {
                // The point is not close to this edge, so we add the full length of this edge to the distance and continue.
                distance += edge.length();
            }
        }
        distance / total_distance
    }
    pub fn alloc_wp(&self, pos: Pos2) -> Waypoint {
        let waypoint_id = next_waypoint_id(&self.waypoints);
        Waypoint {
            id: waypoint_id,
            pos,
            locked: false,
        }
    }
    pub fn add_waypoint(&mut self, pos: Pos2) -> WaypointId {
        if let Some(wp) = self.hit_waypoint(pos, GRID_SIZE * 0.5) {
            return wp;
        }
        let wp = self.alloc_wp(pos);
        self.waypoints.push(wp);
        self.reorder_waypoints();
        wp.id
    }
    pub fn lock_waypoint(&mut self, id: WaypointId) {
        if let Some(wp) = self.waypoint_mut(id) {
            wp.locked = true;
        }
    }
    fn reorder_waypoints(&mut self) {
        let mut waypoints = self
            .waypoints
            .iter()
            .map(|wp| (self.distance_along_route(wp.pos), wp))
            .collect::<Vec<_>>();
        waypoints.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.waypoints = waypoints.into_iter().map(|(_, wp)| wp.clone()).collect();
    }
    pub fn allocate_label(&mut self, pos: Pos2) -> WireLabelId {
        let label_id = self
            .labels
            .iter()
            .map(|label| label.id)
            .max()
            .map(|id| WireLabelId(id.0 + 1))
            .unwrap_or(WireLabelId(0));
        self.labels.push(WireLabel {
            id: label_id,
            position: self.distance_along_route(pos),
            text: String::new(),
        });
        label_id
    }
    pub fn label(&self, label_id: WireLabelId) -> Option<&WireLabel> {
        self.labels.iter().find(|label| label.id == label_id)
    }
    pub fn label_mut(&mut self, label_id: WireLabelId) -> Option<&mut WireLabel> {
        self.labels.iter_mut().find(|label| label.id == label_id)
    }
    pub fn label_edit_details(&mut self, label_id: WireLabelId) -> Option<(Pos2, &mut WireLabel)> {
        let position = self.label(label_id)?.position;
        let pos = self.map_position(position).location;
        let label = self.label_mut(label_id)?;
        Some((pos, label))
    }
    pub fn waypoint(&self, waypoint_id: WaypointId) -> Option<&Waypoint> {
        self.waypoints.iter().find(|wp| wp.id == waypoint_id)
    }
    pub fn waypoint_mut(&mut self, waypoint_id: WaypointId) -> Option<&mut Waypoint> {
        self.waypoints.iter_mut().find(|wp| wp.id == waypoint_id)
    }
    pub fn hit_waypoint(&self, pos: Pos2, tolerance: f32) -> Option<WaypointId> {
        self.waypoints
            .iter()
            .find(|wp| wp.pos.distance(pos) <= tolerance)
            .map(|wp| wp.id)
    }
    pub fn points(&self) -> Vec<Pos2> {
        // We do not want the start and end points duplicated for internal edges.
        let mut points = Vec::new();
        points.push(self.start_pos);
        for edge in &self.edges {
            points.push(edge.end);
        }
        points
    }
    pub fn text_anchor(&self, edge_index: EdgeId) -> Option<Pos2> {
        self.edge(edge_index).and_then(|edge| edge.text_anchor())
    }
    pub fn text_anchors(&self) -> Vec<Pos2> {
        self.edges
            .iter()
            .filter_map(|edge| edge.text_anchor())
            .collect()
    }
    pub fn edge(&self, edge_index: EdgeId) -> Option<&RouteEdge> {
        self.edges.iter().find(|edge| edge.id == edge_index)
    }
    pub fn edge_mut(&mut self, edge_index: EdgeId) -> Option<&mut RouteEdge> {
        self.edges.iter_mut().find(|edge| edge.id == edge_index)
    }
    pub fn is_z_bend(&self, edge_index: EdgeId) -> bool {
        if let Some(ndx) = self.edges.iter().position(|edge| edge.id == edge_index)
            && ndx > 0
            && ndx < self.edges.len() - 1
        {
            return self.edges[ndx - 1].direction() == self.edges[ndx + 1].direction()
                && self.edges[ndx].direction() != self.edges[ndx - 1].direction();
        }
        false
    }
    pub fn move_edge(&mut self, edge_index: EdgeId, delta: Vec2) {
        if let Some(ndx) = self.edges.iter().position(|edge| edge.id == edge_index)
            && self.is_z_bend(edge_index)
        {
            // If this is the vertical segment of a horizontal z-bend, then
            // we calculate the new vertical position of the bend, and update
            // the start and end of the adjacent edges accordingly.
            let (start, end) = {
                let edge = &mut self.edges[ndx];
                let start = edge.start;
                match edge.direction() {
                    RouteDirection::Horizontal => {
                        let delta = vec2(0.0, delta.y);
                        let new_start: Pos2 = start + delta;
                        edge.start.y = new_start.y;
                        edge.end.y = new_start.y;
                    }
                    RouteDirection::Vertical => {
                        let delta = vec2(delta.x, 0.0);
                        let new_start: Pos2 = start + delta;
                        edge.start.x = new_start.x;
                        edge.end.x = new_start.x;
                    }
                }
                (edge.start, edge.end)
            };
            self.edges[ndx - 1].end = start;
            self.edges[ndx + 1].start = end;
        }
    }
    pub fn finish_drag(&mut self, edge_index: EdgeId) {
        if !self.is_z_bend(edge_index) {
            return;
        }
        self.edges.iter_mut().for_each(|edge| {
            edge.start = snap_to_grid(edge.start);
            edge.end = snap_to_grid(edge.end);
        });
        // Drop all waypoints that are no longer on the path.
        self.update_waypoints();
    }
    pub fn update_waypoints(&mut self) {}
    pub fn grid_points(&self) -> Vec<Point> {
        self.points().into_iter().map(|pos| pos.into()).collect()
    }
    pub fn hovered_corner(&self, hover_pos: Pos2) -> Option<(EdgeId, EdgeId)> {
        self.edges.windows(2).find_map(|edges| {
            let edge1 = &edges[0];
            let edge2 = &edges[1];
            if edge1.end.distance(hover_pos) <= LINE_RADIUS
                && edge1.direction() != edge2.direction()
            {
                Some((edge1.id, edge2.id))
            } else {
                None
            }
        })
    }
    pub fn hovered_edge(&self, hover_pos: Pos2) -> Option<EdgeId> {
        self.edges.iter().find_map(|edge| {
            if edge.distance(hover_pos) <= LINE_RADIUS
                && edge.start.distance(hover_pos) > LINE_RADIUS
                && edge.end.distance(hover_pos) > LINE_RADIUS
            {
                Some(edge.id)
            } else {
                None
            }
        })
    }
    pub fn build(
        start: LineAnchor,
        finish: LineAnchor,
        points: &[TaggedPoint],
        waypoints: &[Waypoint],
        labels: &[WireLabel],
    ) -> Self {
        // Scan through the set of points, and create a set of edges.
        // Each edge should be either horizontal or vertical,
        // and should continue as long as possible until the direction changes.
        let mut edges = Vec::new();
        let mut edge_id = 0;
        for windows in points.windows(2) {
            let start = windows[0];
            let end = windows[1];
            if start.segment != end.segment {
                continue;
            }
            edges.push(RouteEdge {
                id: EdgeId(edge_id),
                start: start.pos.into(),
                end: end.pos.into(),
                kind: start.segment,
            });
            edge_id += 1;
        }
        let start_pos = if let Some(point) = points.first() {
            point.pos.into()
        } else {
            pos2(0.0, 0.0)
        };
        let end_pos = if let Some(point) = points.last() {
            point.pos.into()
        } else {
            pos2(0.0, 0.0)
        };
        // Scan through the edges, and as long as they have the same kind
        // and the same direction, merge them into a single edge.
        let mut merged_edges = Vec::new();
        let mut current_edge: Option<RouteEdge> = None;
        for edge in edges {
            if let Some(current) = &mut current_edge {
                if current.kind == edge.kind && current.direction() == edge.direction() {
                    current.end = edge.end;
                } else {
                    merged_edges.push(current.clone());
                    current_edge = Some(edge);
                }
            } else {
                current_edge = Some(edge);
            }
        }
        if let Some(current) = current_edge {
            merged_edges.push(current);
        }
        Self {
            start,
            edges: merged_edges,
            finish,
            start_pos,
            end_pos,
            waypoints: waypoints.to_vec(),
            labels: labels.to_vec(),
        }
    }
    pub fn hit_text_anchor(&self, hover_pos: Pos2) -> Option<EdgeId> {
        self.edges.iter().find_map(|edge| {
            if let Some(anchor) = edge.text_anchor()
                && (anchor.x - hover_pos.x).abs() <= ROUTE_TEXT_SIZE
                && (anchor.y - hover_pos.y).abs() <= ROUTE_TEXT_SIZE
            {
                return Some(edge.id);
            }
            None
        })
    }
}

impl State {
    pub fn cursor(&self) -> CursorIcon {
        match self {
            State::PotentialResize(PotentialResize { mode, .. })
            | State::ResizingRect(ResizingRect { mode, .. }) => match mode {
                ResizeMode::LeftTop | ResizeMode::RightBottom => CursorIcon::ResizeNwSe,
                ResizeMode::RightTop | ResizeMode::LeftBottom => CursorIcon::ResizeNeSw,
                ResizeMode::CenterTop | ResizeMode::CenterBottom => CursorIcon::ResizeVertical,
            },
            State::PortLabelHovered { .. }
            | State::RouteLabelHovered { .. }
            | State::AddText
            | State::AddTextHoveredRoute { .. } => CursorIcon::Text,
            State::PortLabelGripHovered { .. } => CursorIcon::Grab,
            State::PortPinHovered { .. } => CursorIcon::Crosshair,
            State::PortDragged { .. } => CursorIcon::Grabbing,
            State::RouteEdgeHovered(inner) => match inner.direction {
                RouteDirection::Horizontal => CursorIcon::ResizeVertical,
                RouteDirection::Vertical => CursorIcon::ResizeHorizontal,
            },
            _ => CursorIcon::Default,
        }
    }
}
