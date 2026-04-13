use egui::{CursorIcon, Pos2, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, LINE_RADIUS, snap_to_grip},
    label::LabelId,
    rectbox::{LineAnchor, RectId},
    router_ng::Point,
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
    #[default]
    Idle,
    Panning,
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
    RouteEdgeHovered(RouteEdgeHovered),
    RouteEdgeDragged(RouteEdgeDragged),
    WaypointHovered(WaypointHovered),
    RouteLabelHovered(RouteLabelHovered),
    EditingRouteLabelText(EditingRouteLabelText),
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

impl From<RouteEdgeHovered> for State {
    fn from(value: RouteEdgeHovered) -> Self {
        State::RouteEdgeHovered(value)
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
    pub edge_index: EdgeId,
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
pub struct RouteLabelHovered {
    pub id: RouteId,
    pub edge_index: EdgeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingRouteLabelText {
    pub id: RouteId,
    pub edge_index: EdgeId,
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

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub struct EdgeId(usize);

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
    pub label: String,
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
}

#[derive(Clone, PartialEq, Debug)]
pub struct InProgressAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Vec<Pos2>,
    pub head: Pos2,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProposedAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Vec<Pos2>,
    pub finish: LineAnchor,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WaypointHovered {
    pub route: AutoRoute,
    pub waypoint: Pos2,
}

#[derive(Clone, PartialEq, Hash, Copy, Debug, Eq, PartialOrd, Ord, Default)]
pub struct RouteId(usize);

impl RouteId {
    pub fn next(&self) -> Self {
        RouteId(self.0 + 1)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteEdgeHovered {
    pub id: RouteId,
    pub edge_index: EdgeId,
    pub direction: RouteDirection,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AutoRoute {
    pub start: LineAnchor,
    pub edges: Vec<RouteEdge>,
    pub finish: LineAnchor,
    pub start_pos: Pos2,
    pub end_pos: Pos2,
    pub waypoints: Vec<Pos2>,
}

impl AutoRoute {
    pub fn points(&self) -> Vec<Pos2> {
        // We do not want the start and end points duplicated for internal edges.
        let mut points = Vec::new();
        points.push(self.start_pos);
        for edge in &self.edges {
            points.push(edge.end);
        }
        points
    }
    // If we consider the path as a parametric function of distance travelled, then
    // for any given point, there is a closest point on the path to it.
    // This function computes the distance travelled along the path to the first point
    // where the target is within tolerance distance.
    pub fn travel_distance_to_closest_point(&self, pos: Pos2, tolerance: f32) -> Option<f32> {
        let mut distance_travelled = 0.0;
        for edge in &self.edges {
            let edge_length = (edge.end - edge.start).length();
            let distance_to_edge = edge.distance(pos);
            if distance_to_edge <= tolerance {
                return Some(distance_travelled + (edge_length - distance_to_edge));
            }
            distance_travelled += edge_length;
        }
        None
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
            edge.start = snap_to_grip(edge.start);
            edge.end = snap_to_grip(edge.end);
        });
        if let Some((start, end)) = self.edge(edge_index).map(|edge| (edge.start, edge.end)) {
            self.waypoints.push(start);
            self.waypoints.push(end);
        }
        // Drop all waypoints that are no longer on the path.
        self.update_waypoints();
    }
    pub fn update_waypoints(&mut self) {
        // For each waypoint, retain only those that have `self.travel_distance_to_closest_point`
        // within `GRID_SIZE * 0.5` is Some(t).  And the store the values of `t` in a separate array.
        let mut waypoint_distances = self
            .waypoints
            .iter()
            .flat_map(|&wp| {
                self.travel_distance_to_closest_point(wp, GRID_SIZE * 0.5)
                    .map(|t| (wp, t))
            })
            .collect::<Vec<_>>();
        // Sort the waypoints in the order they appear on the path.
        waypoint_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        self.waypoints = waypoint_distances.into_iter().map(|(wp, _)| wp).collect();
    }
    pub fn grid_points(&self) -> Vec<Point> {
        self.points().into_iter().map(|pos| pos.into()).collect()
    }
    pub fn hovered_edge(&self, hover_pos: Pos2) -> Option<EdgeId> {
        self.edges.iter().find_map(|edge| {
            if edge.distance(hover_pos) <= LINE_RADIUS {
                Some(edge.id)
            } else {
                None
            }
        })
    }
    pub fn build(
        start: LineAnchor,
        finish: LineAnchor,
        points: &[Pos2],
        waypoints: &[Pos2],
    ) -> Self {
        // Scan through the set of points, and create a set of edges.
        // Each edge should be either horizontal or vertical,
        // and should continue as long as possible until the direction changes.
        let mut edges = Vec::new();
        let start_pos = points.first().copied().unwrap_or(pos2(0.0, 0.0));
        let end_pos = points.last().copied().unwrap_or(pos2(0.0, 0.0));

        if points.is_empty() {
            return Self {
                start,
                edges,
                finish,
                start_pos,
                end_pos,
                waypoints: waypoints.to_vec(),
            };
        }

        let mut edge_id = 0;
        let mut segment_start = points[0];
        let mut i = 1;

        while i < points.len() {
            let current = points[i];
            let delta = current - segment_start;
            let current_direction = if delta.x.abs() > delta.y.abs() {
                RouteDirection::Horizontal
            } else {
                RouteDirection::Vertical
            };

            // Look ahead to see how far we can extend this edge
            let mut segment_end = current;
            let mut j = i + 1;

            while j < points.len() {
                let next = points[j];
                let next_delta = next - points[j - 1];
                let next_direction = if next_delta.x.abs() > next_delta.y.abs() {
                    RouteDirection::Horizontal
                } else {
                    RouteDirection::Vertical
                };

                if next_direction == current_direction {
                    segment_end = next;
                    j += 1;
                } else {
                    break;
                }
            }
            edges.push(RouteEdge {
                id: EdgeId(edge_id),
                start: segment_start,
                end: segment_end,
                label: String::new(),
            });

            edge_id += 1;
            segment_start = segment_end;
            i = j;
        }

        Self {
            start,
            edges,
            finish,
            start_pos,
            end_pos,
            waypoints: waypoints.to_vec(),
        }
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
            State::PortLabelHovered { .. } | State::RouteLabelHovered { .. } => CursorIcon::Text,
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
