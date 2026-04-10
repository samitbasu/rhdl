use egui::{CursorIcon, Pos2, Vec2, pos2, vec2};
use nonempty::NonEmpty;

use crate::{
    grid::GRID_SIZE,
    label::LabelId,
    rectbox::{LineAnchor, RectId},
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
    AddingRoute(AddingRoute),
    ProposedRoute(Route),
    InProgressAutoRoute(InProgressAutoRoute),
    ProposedAutoRoute(ProposedAutoRoute),
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

impl From<AddingRoute> for State {
    fn from(value: AddingRoute) -> Self {
        State::AddingRoute(value)
    }
}

impl From<Route> for State {
    fn from(value: Route) -> Self {
        State::ProposedRoute(value)
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

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum RouteEdge {
    Horizontal(f32),
    Vertical(f32),
}

#[derive(Clone, PartialEq, Debug)]
pub struct AddingRoute {
    pub anchor: LineAnchor,
    pub edges: Vec<RouteEdge>,
    pub head: Pos2,
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

impl AddingRoute {
    pub fn last_point(&self, start: Pos2) -> Pos2 {
        follow(start, self.edges.iter().copied())
            .last()
            .unwrap_or(start)
    }
    pub fn points(&self, start: Pos2) -> impl Iterator<Item = Pos2> {
        follow(start, self.edges.iter().copied()).chain(std::iter::once(self.head))
    }
    pub fn add_point(&mut self, start_pos: Pos2, new_pos: Pos2) {
        let last_point = self.last_point(start_pos);
        let delta = new_pos - last_point;
        if delta.x.abs() > delta.y.abs() {
            self.edges.push(RouteEdge::Horizontal(delta.x));
        } else {
            self.edges.push(RouteEdge::Vertical(delta.y));
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AutoRoute {
    pub start: LineAnchor,
    pub edges: Vec<RouteEdge>,
    pub finish: LineAnchor,
    pub start_pos: Pos2,
    pub end_pos: Pos2,
}

impl AutoRoute {
    pub fn points(&self) -> Vec<Pos2> {
        std::iter::once(self.start_pos)
            .chain(follow(self.start_pos, self.edges.iter().copied()))
            .collect()
    }
    // For display, we diagonally cut GRID/4.0 off the of the route, so that
    // traces that meet at a junction don't visually merge together.
    // Instead of
    //.     | <-B
    //.     |.  v
    //. ----+-----
    //. ^   |
    //. A-> |
    // We instead draw
    //.     | <-B
    //.     |.  v
    //. --   \-----
    //.    \
    //. ^   |
    //. A-> |
    // This visual tweak requires tracking each time the track turns from horizontal
    // to vertical, and then adjusting the display position of the route slightly at those points.
    pub fn display_points(&self, start: Pos2) -> Vec<Pos2> {
        const CHAMFER: f32 = GRID_SIZE / 5.0;
        let mut points = Vec::new();
        let mut pos = start;

        for (i, edge) in self.edges.iter().enumerate() {
            // Check if we're turning at the start of this edge
            let turning_in = i > 0 && {
                let prev = &self.edges[i - 1];
                !matches!(
                    (prev, edge),
                    (RouteEdge::Horizontal(_), RouteEdge::Horizontal(_))
                        | (RouteEdge::Vertical(_), RouteEdge::Vertical(_))
                )
            };

            // Check if we're turning at the end of this edge
            let turning_out = i + 1 < self.edges.len() && {
                let next = &self.edges[i + 1];
                !matches!(
                    (edge, next),
                    (RouteEdge::Horizontal(_), RouteEdge::Horizontal(_))
                        | (RouteEdge::Vertical(_), RouteEdge::Vertical(_))
                )
            };

            // Move forward by CHAMFER if we're turning in
            if turning_in {
                pos = match edge {
                    RouteEdge::Horizontal(dx) => pos + vec2(dx.signum() * CHAMFER, 0.0),
                    RouteEdge::Vertical(dy) => pos + vec2(0.0, dy.signum() * CHAMFER),
                };
            }
            points.push(pos);

            // Move along the edge, stopping CHAMFER short if we're turning out
            let distance = match edge {
                RouteEdge::Horizontal(dx) => dx.abs() - if turning_out { CHAMFER } else { 0.0 },
                RouteEdge::Vertical(dy) => dy.abs() - if turning_out { CHAMFER } else { 0.0 },
            };
            pos = match edge {
                RouteEdge::Horizontal(dx) => pos + vec2(dx.signum() * distance, 0.0),
                RouteEdge::Vertical(dy) => pos + vec2(0.0, dy.signum() * distance),
            };
            points.push(pos);

            // Move to the actual corner position (full edge endpoint)
            pos = match edge {
                RouteEdge::Horizontal(dx) => {
                    pos + vec2(dx.signum() * if turning_out { CHAMFER } else { 0.0 }, 0.0)
                }
                RouteEdge::Vertical(dy) => {
                    pos + vec2(0.0, dy.signum() * if turning_out { CHAMFER } else { 0.0 })
                }
            };
        }

        points
    }
    pub fn build(start: LineAnchor, finish: LineAnchor, points: &[Pos2]) -> Self {
        // Scan through the set of points, and create a set of edges.
        // Each edge should be either horizontal or vertical,
        // and should continue as long as possible until the direction changes.
        let mut edges = Vec::new();
        let mut current_pos = None;
        let mut accumulated_edge: Option<RouteEdge> = None;
        let start_pos = points.first().copied().unwrap_or(pos2(0.0, 0.0));
        let end_pos = points.last().copied().unwrap_or(pos2(0.0, 0.0));

        for &point in points {
            if let Some(prev_pos) = current_pos {
                let delta: Vec2 = point - prev_pos;
                let new_edge = if delta.x.abs() > delta.y.abs() {
                    RouteEdge::Horizontal(delta.x)
                } else {
                    RouteEdge::Vertical(delta.y)
                };

                // Check if we can merge with the accumulated edge
                accumulated_edge = match accumulated_edge {
                    Some(RouteEdge::Horizontal(acc)) => match new_edge {
                        RouteEdge::Horizontal(dx) => Some(RouteEdge::Horizontal(acc + dx)),
                        RouteEdge::Vertical(_) => {
                            edges.push(RouteEdge::Horizontal(acc));
                            Some(new_edge)
                        }
                    },
                    Some(RouteEdge::Vertical(acc)) => match new_edge {
                        RouteEdge::Vertical(dy) => Some(RouteEdge::Vertical(acc + dy)),
                        RouteEdge::Horizontal(_) => {
                            edges.push(RouteEdge::Vertical(acc));
                            Some(new_edge)
                        }
                    },
                    None => Some(new_edge),
                };
            }
            current_pos = Some(point);
        }

        // Don't forget to push the last accumulated edge
        if let Some(edge) = accumulated_edge {
            edges.push(edge);
        }

        Self {
            start,
            edges,
            finish,
            start_pos,
            end_pos,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Route {
    pub start: LineAnchor,
    pub edges: NonEmpty<RouteEdge>,
    pub tail: Vec2,
    pub finish: LineAnchor,
}

pub fn follow(start: Pos2, deltas: impl Iterator<Item = RouteEdge>) -> impl Iterator<Item = Pos2> {
    deltas.scan(start, |current_pos, delta| {
        let new_pos = match delta {
            RouteEdge::Horizontal(offset) => pos2(current_pos.x + offset, current_pos.y),
            RouteEdge::Vertical(offset) => pos2(current_pos.x, current_pos.y + offset),
        };
        *current_pos = new_pos;
        Some(new_pos)
    })
}

impl Route {
    pub fn update_tail(&mut self, start: Pos2, end: Pos2) {
        //        let route = follow(start, self.edges.iter().copied()).chain(std::iter::once(end));
        let last_point_of_route = follow(start, self.edges.iter().copied())
            .last()
            .unwrap_or(start);
        self.tail = end - last_point_of_route;
    }
    pub fn points(&self, start: Pos2) -> Vec<Pos2> {
        let mut body = follow(start, self.edges.iter().copied()).collect::<Vec<_>>();
        let last = body.last().copied().unwrap_or(start);
        if self.tail.x.abs() > self.tail.y.abs() {
            body.push(pos2(last.x + self.tail.x, last.y));
        } else {
            body.push(pos2(last.x, last.y + self.tail.y));
        }
        body.push(pos2(last.x + self.tail.x, last.y + self.tail.y));
        body
    }
    pub fn from_adding_route(
        start: Pos2,
        end: Pos2,
        adding: AddingRoute,
        finish: LineAnchor,
    ) -> Self {
        let last_point_of_route = adding.last_point(start);
        let tail = end - last_point_of_route;
        let turns = NonEmpty::from_vec(adding.edges)
            .unwrap_or(NonEmpty::singleton(RouteEdge::Horizontal(0.0)));
        Self {
            start: adding.anchor,
            edges: turns,
            tail,
            finish,
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
            State::PortLabelHovered { .. } => CursorIcon::Text,
            State::PortLabelGripHovered { .. } => CursorIcon::Grab,
            State::PortPinHovered { .. } | State::AddingRoute { .. } => CursorIcon::Crosshair,
            State::PortDragged { .. } => CursorIcon::Grabbing,
            _ => CursorIcon::Default,
        }
    }
}
