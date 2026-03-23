use egui::{Color32, Pos2, Rect, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::{
    drawing::ResizeMode,
    grid::{GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_RADIUS, SHIM, grid_rect, round_to_grid, snap},
    label::{Label, LabelId, LabelSide},
};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct RectId(usize);

impl RectId {
    pub fn next(self) -> Self {
        RectId(self.0 + 1)
    }
}

#[derive(Clone, Copy)]
pub enum ModificationKind {
    Move,
    ResizeLeftTop,
    ResizeRightBottom,
    ResizeRightTop,
    ResizeLeftBottom,
    LabelDrag,
    Create,
    DeletePending,
}

pub struct SaveState {
    pub kind: ModificationKind,
    pub orig: Rect,
}

pub struct RectBox {
    pub name: String,
    pub inner: Rect,
    pub labels: Vec<Label>,
    pub edit_kind: Option<SaveState>,
    id: RectId,
    label_id: LabelId,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct LineAnchor {
    pub rect: RectId,
    pub label: LabelId,
}

pub fn control_corner(rect: &Rect, mode: ResizeMode) -> Pos2 {
    match mode {
        ResizeMode::LeftTop => rect.left_top(),
        ResizeMode::RightTop => rect.right_top(),
        ResizeMode::LeftBottom => rect.left_bottom(),
        ResizeMode::RightBottom => rect.right_bottom(),
        ResizeMode::CenterTop => rect.center_top(),
        ResizeMode::CenterBottom => rect.center_bottom(),
    }
}

pub fn resize_rect(rect: &Rect, mode: ResizeMode, delta: Vec2) -> Rect {
    match mode {
        ResizeMode::LeftTop => Rect::from_two_pos(rect.left_top() + delta, rect.right_bottom()),
        ResizeMode::RightTop => Rect::from_two_pos(rect.right_top() + delta, rect.left_bottom()),
        ResizeMode::LeftBottom => Rect::from_two_pos(rect.left_bottom() + delta, rect.right_top()),
        ResizeMode::RightBottom => Rect::from_two_pos(rect.right_bottom() + delta, rect.left_top()),
        ResizeMode::CenterTop => {
            Rect::from_two_pos(rect.left_top() + vec2(0.0, delta.y), rect.right_bottom())
        }
        ResizeMode::CenterBottom => {
            Rect::from_two_pos(rect.left_bottom() + vec2(0.0, delta.y), rect.right_top())
        }
    }
}

impl RectBox {
    pub fn id(&self) -> RectId {
        self.id
    }
    pub fn new(name: String, inner: Rect, id: RectId) -> Self {
        Self {
            name,
            inner: snap(inner),
            labels: Vec::new(),
            edit_kind: None,
            id,
            label_id: LabelId::default(),
        }
    }
    pub fn next_port_offset(&self, side: LabelSide) -> Option<f32> {
        let max_pos = (self.inner.height() / GRID_SIZE) as i32 - 1;
        if max_pos <= 0 {
            return None;
        }
        (0_u32..max_pos as u32).find_map(|ndx| {
            let offset = ndx as f32 * GRID_SIZE;
            if self
                .labels
                .iter()
                .any(|l| l.side == side && (l.offset - offset).abs() < GRID_SIZE * 0.6)
            {
                None
            } else {
                Some(offset)
            }
        })
    }
    pub fn control_pin_location_east(&self) -> Option<Pos2> {
        // Find the first free offset
        // We want to check 0, -1, 1, -2, 2,..
        let offset = self.next_port_offset(LabelSide::East)?;
        Some(self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn control_pin_location_west(&self) -> Option<Pos2> {
        let offset = self.next_port_offset(LabelSide::West)?;
        Some(self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn control_pin_for_label(&self, label_id: LabelId) -> Option<Pos2> {
        self.labels
            .iter()
            .find(|l| l.id == label_id)
            .map(|label| match label.side {
                LabelSide::East => {
                    self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + label.offset)
                }
                LabelSide::West => {
                    self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + label.offset)
                }
            })
    }
    pub fn anchor_point(&self, id: LabelId) -> Pos2 {
        let label = self.labels.iter().find(|l| l.id == id).unwrap();
        match label.side {
            LabelSide::East => pos2(
                self.inner.right(),
                self.inner.top() + GRID_SIZE + label.offset,
            ),
            LabelSide::West => pos2(
                self.inner.left(),
                self.inner.top() + GRID_SIZE + label.offset,
            ),
        }
    }
    pub fn add_label(&mut self, text: String, side: LabelSide, offset: f32) -> LineAnchor {
        let id = self.label_id;
        self.label_id = self.label_id.next();
        self.labels.push(Label {
            text,
            side,
            offset,
            id,
        });
        LineAnchor {
            rect: self.id,
            label: id,
        }
    }
    pub fn save_state(&mut self, kind: ModificationKind) {
        if self.edit_kind.is_some() {
            return;
        }
        self.edit_kind = Some(SaveState {
            kind,
            orig: self.inner,
        });
    }
    pub fn drag_right_bottom(&mut self, delta: Vec2) {
        self.save_state(ModificationKind::ResizeRightBottom);
        self.inner.set_bottom(self.inner.bottom() + delta.y);
        self.inner.set_right(self.inner.right() + delta.x);
    }
    pub fn drag_right_top(&mut self, delta: Vec2) {
        self.save_state(ModificationKind::ResizeRightTop);
        self.inner.set_top(self.inner.top() + delta.y);
        self.inner.set_right(self.inner.right() + delta.x);
    }
    pub fn drag_left_bottom(&mut self, delta: Vec2) {
        self.save_state(ModificationKind::ResizeLeftBottom);
        self.inner.set_bottom(self.inner.bottom() + delta.y);
        self.inner.set_left(self.inner.left() + delta.x);
    }
    pub fn drag_left_top(&mut self, delta: Vec2) {
        self.save_state(ModificationKind::ResizeLeftTop);
        self.inner.set_top(self.inner.top() + delta.y);
        self.inner.set_left(self.inner.left() + delta.x);
    }
    pub fn drag_center(&mut self, delta: Vec2) {
        self.save_state(ModificationKind::Move);
        self.inner = self.inner.translate(delta);
    }
    pub fn predicted_rect(&self) -> Rect {
        grid_rect(self.inner)
    }
    pub fn complete_edit(&mut self) {
        self.inner = self.predicted_rect();
        self.labels.iter_mut().for_each(|label| {
            label.offset = round_to_grid(label.offset);
        });
        self.edit_kind = None;
    }
    pub fn delete_pending(&self) -> bool {
        matches!(
            self.edit_kind.as_ref().map(|s| s.kind),
            Some(ModificationKind::DeletePending)
        )
    }
}
