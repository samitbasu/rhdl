use egui::{Color32, Pos2, Rect, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::{
    drawing::ResizeMode,
    grid::{
        MOVE_HOVER_DISTANCE, PORT_RADIUS, SHIM, grid, grid_rect, round_to_even_grid, round_to_grid,
        snap,
    },
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
    pub fn render_control_frame(&self, ui: &mut Ui) {
        let bbox = self.inner;
        ui.painter().rect(
            bbox,
            0.0,
            Color32::TRANSPARENT,
            (0.5, Color32::DARK_RED),
            StrokeKind::Middle,
        );
        for pos in [
            bbox.left_top(),
            bbox.right_top(),
            bbox.left_bottom(),
            bbox.right_bottom(),
            bbox.center_top(),
            bbox.center_bottom(),
        ] {
            ui.painter().rect(
                Rect::from_center_size(pos, vec2(3.0, 3.0)),
                0.0,
                Color32::WHITE,
                (0.5, Color32::BLACK),
                StrokeKind::Middle,
            );
        }
    }
    pub fn render_resizing(&self, ui: &mut Ui, mode: ResizeMode, delta: Vec2) {
        let resized_rect = resize_rect(&self.inner, mode, delta);
        let predicted_rect = grid_rect(resized_rect);
        ui.painter().rect(
            predicted_rect,
            3.0,
            Color32::TRANSPARENT,
            (1.0, Color32::DARK_GRAY),
            StrokeKind::Middle,
        );
        ui.painter().rect(
            resized_rect,
            3.0,
            Color32::LIGHT_GRAY,
            (2.0, Color32::DARK_RED),
            StrokeKind::Middle,
        );
    }
    pub fn render_moving(&self, ui: &mut Ui, delta: Vec2) {
        let shifted_rect = self.inner.translate(delta);
        let predicted_rect = grid_rect(shifted_rect);
        ui.painter().rect(
            predicted_rect,
            3.0,
            Color32::TRANSPARENT,
            (1.0, Color32::DARK_GRAY),
            StrokeKind::Middle,
        );
        ui.painter().rect(
            shifted_rect,
            3.0,
            Color32::LIGHT_GRAY,
            (2.0, Color32::DARK_RED),
            StrokeKind::Middle,
        );
        ui.painter().text(
            shifted_rect.center_top() + vec2(0.0, SHIM),
            egui::Align2::CENTER_TOP,
            &self.name,
            egui::FontId::monospace(10.0),
            Color32::BLACK,
        );
        self.render_labels_with_box(shifted_rect, ui);
    }
    pub fn render_still(&self, ui: &mut Ui) {
        let egui_box = self.inner;
        ui.painter().rect(
            egui_box,
            3.0,
            Color32::LIGHT_GRAY,
            (1.0, Color32::BLUE),
            StrokeKind::Middle,
        );
        ui.painter().line_segment(
            [
                egui_box.center() + vec2(-MOVE_HOVER_DISTANCE / 2.0, 0.0),
                egui_box.center() + vec2(MOVE_HOVER_DISTANCE / 2.0, 0.0),
            ],
            (0.5, Color32::LIGHT_RED.gamma_multiply(0.4)),
        );
        ui.painter().line_segment(
            [
                egui_box.center() + vec2(0.0, -MOVE_HOVER_DISTANCE / 2.0),
                egui_box.center() + vec2(0.0, MOVE_HOVER_DISTANCE / 2.0),
            ],
            (0.5, Color32::LIGHT_RED.gamma_multiply(0.4)),
        );
        ui.painter().text(
            egui_box.center_top() + vec2(0.0, SHIM),
            egui::Align2::CENTER_TOP,
            &self.name,
            egui::FontId::monospace(10.0),
            Color32::BLACK,
        );
        self.render_labels_with_box(egui_box, ui);
    }
    fn render_labels_with_box(&self, bbox: Rect, ui: &mut Ui) {
        for label in &self.labels {
            let (align, text_pos, shift) = match label.side {
                LabelSide::East => (
                    egui::Align2::RIGHT_CENTER,
                    pos2(bbox.right() - SHIM, bbox.center().y + label.offset),
                    vec2(SHIM, 0.0),
                ),
                LabelSide::West => (
                    egui::Align2::LEFT_CENTER,
                    pos2(bbox.left() + SHIM, bbox.center().y + label.offset),
                    vec2(-SHIM, 0.0),
                ),
            };
            ui.painter().circle(
                text_pos + shift,
                PORT_RADIUS,
                Color32::DARK_GRAY,
                (0.5, Color32::DARK_RED),
            );
            ui.painter().text(
                text_pos,
                align,
                &label.text,
                egui::FontId::monospace(8.0),
                Color32::BLACK,
            );
        }
    }
    pub fn anchor_point(&self, id: LabelId) -> Pos2 {
        let label = self.labels.iter().find(|l| l.id == id).unwrap();
        match label.side {
            LabelSide::East => pos2(self.inner.right(), self.inner.center().y + label.offset),
            LabelSide::West => pos2(self.inner.left(), self.inner.center().y + label.offset),
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
        let min_height = round_to_even_grid(
            self.labels
                .iter()
                .map(|label| label.offset.abs() * 2.0)
                .fold(0.0, f32::max)
                + SHIM * 2.0,
        );
        if let Some(save_state) = &self.edit_kind {
            match save_state.kind {
                ModificationKind::Move => grid_rect(self.inner),
                ModificationKind::ResizeRightBottom => {
                    let top_left = grid(save_state.orig.left_top());
                    let mut bottom_right = grid(self.inner.right_bottom());
                    let height = round_to_even_grid((bottom_right.y - top_left.y).max(min_height));
                    bottom_right.y = top_left.y + height;
                    Rect::from_two_pos(top_left, bottom_right)
                }
                ModificationKind::ResizeRightTop => {
                    let bottom_left = grid(save_state.orig.left_bottom());
                    let mut top_right = grid(self.inner.right_top());
                    let height = round_to_even_grid((bottom_left.y - top_right.y).max(min_height));
                    top_right.y = bottom_left.y - height;
                    Rect::from_two_pos(bottom_left, top_right)
                }
                ModificationKind::ResizeLeftBottom => {
                    let top_right = grid(save_state.orig.right_top());
                    let mut bottom_left = grid(self.inner.left_bottom());
                    let height = round_to_even_grid((bottom_left.y - top_right.y).max(min_height));
                    bottom_left.y = top_right.y + height;
                    Rect::from_two_pos(top_right, bottom_left)
                }
                ModificationKind::ResizeLeftTop => {
                    let bottom_right = grid(save_state.orig.right_bottom());
                    let mut top_left = grid(self.inner.left_top());
                    let height = round_to_even_grid((bottom_right.y - top_left.y).max(min_height));
                    top_left.y = bottom_right.y - height;
                    Rect::from_two_pos(top_left, bottom_right)
                }
                _ => grid_rect(self.inner),
            }
        } else {
            grid_rect(self.inner)
        }
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
