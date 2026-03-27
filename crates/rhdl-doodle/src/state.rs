use egui::{CursorIcon, Pos2, Vec2, pos2, vec2};
use nonempty::NonEmpty;

use crate::{
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
    pub turns: Vec<Vec2>,
    pub head: Pos2,
}

impl AddingRoute {
    pub fn last_point(&self, start: Pos2) -> Pos2 {
        self.turns
            .iter()
            .fold(start, |current_pos, delta| current_pos + *delta)
    }
    pub fn points(&self, start: Pos2) -> impl Iterator<Item = Pos2> {
        orthogonalize(start, self.turns.iter().copied()).chain(std::iter::once(self.head))
    }
    pub fn add_point(&mut self, start_pos: Pos2, new_pos: Pos2) {
        let last_point = self.last_point(start_pos);
        let delta = new_pos - last_point;
        if delta.x.abs() > delta.y.abs() {
            self.turns.push(vec2(delta.x, 0.0));
        } else {
            self.turns.push(vec2(0.0, delta.y));
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Route {
    pub start: LineAnchor,
    pub turns: NonEmpty<Vec2>,
    pub tail: f32,
    pub finish: LineAnchor,
}

fn hvec(del: Vec2) -> Vec2 {
    vec2(del.x, 0.0)
}

fn vvec(del: Vec2) -> Vec2 {
    vec2(0.0, del.y)
}

fn orthogonalize(start: Pos2, deltas: impl Iterator<Item = Vec2>) -> impl Iterator<Item = Pos2> {
    deltas
        .flat_map(|delta| [hvec(delta), vvec(delta)])
        .scan(start, |current_pos, delta| {
            *current_pos += delta;
            Some(*current_pos)
        })
}

impl Route {
    fn last_point(&self, start: Pos2) -> Pos2 {
        self.turns
            .iter()
            .fold(start, |current_pos, delta| current_pos + *delta)
            + vec2(self.tail, 0.0)
    }
    pub fn points(&self, start: Pos2) -> impl Iterator<Item = Pos2> {
        let turns = self
            .turns
            .iter()
            .copied()
            .chain(std::iter::once(vec2(self.tail, 0.0)));
        std::iter::once(start)
            .chain(orthogonalize(start, turns))
            .chain(std::iter::once(self.last_point(start)))
    }
    pub fn from_adding_route(
        start: Pos2,
        end: Pos2,
        adding: AddingRoute,
        finish: LineAnchor,
    ) -> Self {
        let tail = end.x - adding.last_point(start).x;
        let turns = NonEmpty::from_vec(adding.turns).unwrap_or(NonEmpty::singleton(vec2(0.0, 0.0)));
        Self {
            start: adding.anchor,
            turns,
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
