use egui::{
    Color32, CursorIcon, Id, PointerButton, Pos2, Rect, Response, StrokeKind, TextEdit, Ui, Vec2,
    pos2, vec2,
};

use crate::{
    geometry::minimum_distance_and_location,
    grid::{GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_RADIUS, SHIM, grid, grid_rect, round_even_grid},
    label::LabelSide,
    polyline::{LineId, PolyLine},
    rectbox::{LineAnchor, ModificationKind, RectBox, RectId, control_corner, resize_rect},
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

const RESIZE_MODES: &[ResizeMode] = &[
    ResizeMode::LeftTop,
    ResizeMode::RightTop,
    ResizeMode::LeftBottom,
    ResizeMode::RightBottom,
    ResizeMode::CenterTop,
    ResizeMode::CenterBottom,
];

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum State {
    #[default]
    Idle,
    Panning,
    AddingRect {
        start_pos: Pos2,
        end_pos: Pos2,
    },
    MovingRect {
        rect: RectId,
        delta_pos: Vec2,
    },
    Selected {
        rect: RectId,
    },
    PotentialResize {
        rect: RectId,
        mode: ResizeMode,
    },
    ResizingRect {
        rect: RectId,
        mode: ResizeMode,
        delta_pos: Vec2,
    },
    EditingName {
        rect: RectId,
    },
    PinPlusClicked {
        rect: RectId,
    },
}

impl State {
    pub fn cursor(&self) -> CursorIcon {
        match self {
            State::PotentialResize { mode, .. } | State::ResizingRect { mode, .. } => match mode {
                ResizeMode::LeftTop | ResizeMode::RightBottom => CursorIcon::ResizeNwSe,
                ResizeMode::RightTop | ResizeMode::LeftBottom => CursorIcon::ResizeNeSw,
                ResizeMode::CenterTop | ResizeMode::CenterBottom => CursorIcon::ResizeVertical,
            },
            _ => CursorIcon::Default,
        }
    }
    pub fn selected_id(&self) -> Option<RectId> {
        match self {
            State::Selected { rect } => Some(*rect),
            State::PotentialResize { rect, .. } | State::ResizingRect { rect, .. } => Some(*rect),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Drawing {
    rect_id: RectId,
    rect_boxes: Vec<RectBox>,
    poly_lines: Vec<PolyLine>,
    pub selected: Option<RectId>,
    pub line_add_anchor: Option<LineAnchor>,
    pub line_add_set: Vec<Pos2>,
    pub line_add_current: Option<Pos2>,
    state: State,
}

const HIT_DISTANCE: f32 = 10.0;

impl Drawing {
    pub fn set_selected(&mut self, id: Option<RectId>) {
        self.selected = id;
    }
    pub fn line_add_anchor(&self) -> Option<Pos2> {
        let anchor = self.line_add_anchor?;
        let rbox = self.rect_boxes.iter().find(|r| r.id() == anchor.rect)?;
        Some(rbox.anchor_point(anchor.label))
    }
    pub fn rect(&self, id: RectId) -> Option<&RectBox> {
        self.rect_boxes.iter().find(|r| r.id() == id)
    }
    pub fn rect_mut(&mut self, id: RectId) -> Option<&mut RectBox> {
        self.rect_boxes.iter_mut().find(|r| r.id() == id)
    }
    pub fn selected_rect_mut(&mut self) -> Option<&mut RectBox> {
        self.selected
            .and_then(|id| self.rect_boxes.iter_mut().find(|r| r.id() == id))
    }
    pub fn delete_selected(&mut self) {
        if let Some(selected_id) = self.selected {
            self.rect_boxes.retain(|r| r.id() != selected_id);
        }
    }
    pub fn add_new_label(&mut self) {
        if let Some(selected) = self.selected
            && let Some(rect_box) = self.rect_boxes.iter_mut().find(|r| r.id() == selected)
        {
            let offset = rect_box
                .labels
                .iter()
                .filter_map(|l| {
                    if l.side == LabelSide::East {
                        Some(l.offset.abs())
                    } else {
                        None
                    }
                })
                .fold(0.0, f32::max);
            rect_box.add_label("new label".to_string(), LabelSide::East, offset + 10.0);
        }
    }
    pub fn add_rect_box(&mut self, start: Pos2, end: Pos2) -> &mut RectBox {
        let id = self.rect_id;
        self.rect_id = self.rect_id.next();
        self.rect_boxes.push(RectBox::new(
            "Untitled".to_string(),
            Rect::from_two_pos(start, end),
            id,
        ));
        self.rect_boxes.last_mut().unwrap()
    }
    pub fn anchor(&self, anchor: LineAnchor) -> Pos2 {
        let rect = self
            .rect_boxes
            .iter()
            .find(|r| r.id() == anchor.rect)
            .unwrap();
        rect.anchor_point(anchor.label)
    }
    pub fn add_line(&mut self, start: LineAnchor, points: &[Pos2], end: LineAnchor) {
        let id = self
            .poly_lines
            .last()
            .map_or(LineId::default(), |l| l.id().next());
        self.poly_lines.push(PolyLine {
            start,
            points: points.iter().map(|&p| grid(p)).collect(),
            end,
            id,
            editing: None,
        });
    }
    pub fn update(&mut self, ui: &mut Ui) {
        (-100..=100).map(|y| y as f32 * GRID_SIZE).for_each(|h| {
            ui.painter().hline(
                -10_000.0f32..=10_000.0f32,
                h,
                (0.15, Color32::LIGHT_GRAY.linear_multiply(0.3)),
            );
        });
        (-100..=100).map(|x| x as f32 * GRID_SIZE).for_each(|v| {
            ui.painter().vline(
                v,
                -10_000.0f32..=10_000.0f32,
                (0.15, Color32::LIGHT_GRAY.linear_multiply(0.3)),
            );
        });
        // Draw the wires
        let mut poly_lines = std::mem::take(&mut self.poly_lines);
        let mut edit_active = false;
        let mut bound_rect = Rect::NOTHING;
        for poly_line in poly_lines.iter_mut() {
            if poly_line.editing.is_some() {
                edit_active = true;
                let points = std::iter::once(self.anchor(poly_line.start))
                    .chain(poly_line.points.iter().map(|p| grid(*p)))
                    .chain(std::iter::once(self.anchor(poly_line.end)))
                    .collect::<Vec<_>>();
                ui.painter()
                    .add(egui::Shape::line(points, (1.0, Color32::DARK_GRAY)));
            }
            let points = std::iter::once(self.anchor(poly_line.start))
                .chain(poly_line.points.iter().cloned())
                .chain(std::iter::once(self.anchor(poly_line.end)))
                .collect::<Vec<_>>();
            let rect = Rect::from_points(&points).expand(HIT_DISTANCE);
            bound_rect = bound_rect.union(rect);
            ui.painter().add(egui::Shape::line(
                points.clone(),
                (1.0, Color32::DARK_GREEN),
            ));
            if let Some(edit_pos) = poly_line.edit_point() {
                ui.painter().circle(
                    edit_pos,
                    1.5,
                    Color32::LIGHT_GRAY,
                    (0.5, Color32::DARK_BLUE),
                );
                let response = ui.interact(
                    Rect::from_center_size(edit_pos, vec2(5.0, 5.0)),
                    Id::new("poly_line"),
                    egui::Sense::click_and_drag(),
                );
                if response.drag_stopped() {
                    poly_line.finish_edit();
                } else {
                    poly_line.shift_edit_point(response.drag_delta());
                }
            }
        }

        if !edit_active {
            let response = ui.interact(
                bound_rect,
                Id::new("poly_line"),
                egui::Sense::click_and_drag(),
            );
            if let Some(pos) = response.hover_pos() {
                for poly_line in &poly_lines {
                    let points = std::iter::once(self.anchor(poly_line.start))
                        .chain(poly_line.points.iter().cloned())
                        .chain(std::iter::once(self.anchor(poly_line.end)))
                        .collect::<Vec<_>>();
                    let contact = minimum_distance_and_location(&points, pos);
                    if contact.distance < HIT_DISTANCE {
                        ui.painter().circle(
                            contact.location,
                            3.0,
                            Color32::TRANSPARENT,
                            (0.5, Color32::DARK_GREEN),
                        );
                    }
                }
            }
            if response.drag_started()
                && let Some(pos) = response.interact_pointer_pos()
            {
                for poly_line in &mut poly_lines {
                    let points = std::iter::once(self.anchor(poly_line.start))
                        .chain(poly_line.points.iter().cloned())
                        .chain(std::iter::once(self.anchor(poly_line.end)))
                        .collect::<Vec<_>>();
                    let contact = minimum_distance_and_location(&points, pos);
                    if contact.distance < HIT_DISTANCE {
                        poly_line.start_edit(contact);
                    }
                }
            }
        }

        self.poly_lines = poly_lines;

        for rect_box in self.rect_boxes.iter_mut() {
            let mut editing = false;
            let egui_box = rect_box.inner;
            let is_dragging = rect_box.edit_kind.is_some();
            let is_selected = self.selected == Some(rect_box.id());
            let stroke = if is_dragging {
                (2.0, Color32::DARK_RED)
            } else if is_selected {
                (2.0, Color32::YELLOW)
            } else {
                (1.0, Color32::DARK_BLUE)
            };
            let predicted_rect = rect_box.predicted_rect();
            if is_dragging {
                ui.painter().rect(
                    predicted_rect,
                    3.0,
                    Color32::TRANSPARENT,
                    (1.0, Color32::DARK_GRAY),
                    StrokeKind::Middle,
                );
            }
            if is_selected
                && !ui.ctx().wants_keyboard_input()
                && ui.input(|i| {
                    i.key_pressed(egui::Key::Delete) | i.key_pressed(egui::Key::Backspace)
                })
            {
                rect_box.save_state(ModificationKind::DeletePending);
                editing = true;
            }

            /*             rect_box.edit_kind.is_none() {
                           let hit_center = rect_box.inner.left_top() + vec2(-8.0, -8.0);
                           let delete_radius = 4.0;
                           let hit_box = Rect::from_center_size(
                               hit_center,
                               vec2(delete_radius * 2.0, delete_radius * 2.0),
                           );
                           let response = ui.interact(hit_box, Id::new("delete_button"), egui::Sense::click());
                           let stroke = if response.hovered() { 2.0 } else { 1.0 };
                           let delete_radius = if response.is_pointer_button_down_on() {
                               5.0
                           } else {
                               4.0
                           };
                           if response.clicked() {
                               rect_box.save_state(ModificationKind::DeletePending);
                               editing = true;
                           }
                           ui.painter()
                               .circle_stroke(hit_center, delete_radius, (stroke, Color32::DARK_RED));
                           let delete_radius = delete_radius / 2.0_f32.sqrt();
                           ui.painter().line_segment(
                               [
                                   hit_center + vec2(-delete_radius, -delete_radius),
                                   hit_center + vec2(delete_radius, delete_radius),
                               ],
                               (stroke, Color32::DARK_RED),
                           );
                           ui.painter().line_segment(
                               [
                                   hit_center + vec2(-delete_radius, delete_radius),
                                   hit_center + vec2(delete_radius, -delete_radius),
                               ],
                               (stroke, Color32::DARK_RED),
                           );
                       }
            */
            ui.painter().rect(
                egui_box,
                3.0,
                Color32::LIGHT_GRAY,
                stroke,
                StrokeKind::Middle,
            );
            ui.painter().text(
                egui_box.center_top() + vec2(0.0, SHIM),
                egui::Align2::CENTER_TOP,
                &rect_box.name,
                egui::FontId::monospace(10.0),
                Color32::BLACK,
            );
            let id = rect_box.id();
            let mut labels = std::mem::take(&mut rect_box.labels);
            for label in labels.iter_mut() {
                let (align, text_pos, shift) = match label.side {
                    LabelSide::East => (
                        egui::Align2::RIGHT_CENTER,
                        pos2(egui_box.right() - SHIM, egui_box.center().y + label.offset),
                        vec2(SHIM, 0.0),
                    ),
                    LabelSide::West => (
                        egui::Align2::LEFT_CENTER,
                        pos2(egui_box.left() + SHIM, egui_box.center().y + label.offset),
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
                let label_ndx = label.id;
                let response = ui
                    .interact(
                        Rect::from_center_size(
                            text_pos + shift,
                            vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0),
                        ),
                        Id::new(("label", id, label_ndx)),
                        egui::Sense::click_and_drag(),
                    )
                    .on_hover_cursor(egui::CursorIcon::Crosshair);
                if response.hovered() {
                    ui.painter().circle(
                        text_pos + shift,
                        3.0,
                        Color32::TRANSPARENT,
                        (0.5, Color32::WHITE),
                    );
                }
                if response.dragged_by(egui::PointerButton::Secondary) {
                    let delta = response.drag_delta();
                    rect_box.save_state(ModificationKind::LabelDrag);
                    label.offset += delta.y;
                    if let Some(pointer) = response.interact_pointer_pos() {
                        if label.side == LabelSide::West && pointer.x > egui_box.center().x {
                            label.side = LabelSide::East;
                        } else if label.side == LabelSide::East && pointer.x < egui_box.center().x {
                            label.side = LabelSide::West;
                        }
                    }
                    editing = true;
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
                }
                if response.clicked_by(egui::PointerButton::Primary) {
                    if let Some(start) = self.line_add_anchor {
                        let new_poly = PolyLine {
                            start,
                            points: std::mem::take(&mut self.line_add_set),
                            end: LineAnchor {
                                rect: rect_box.id(),
                                label: label.id,
                            },
                            id: self
                                .poly_lines
                                .last()
                                .map_or(LineId::default(), |l| l.id.next()),
                            editing: None,
                        };
                        self.poly_lines.push(new_poly);
                        self.line_add_anchor = None;
                        self.line_add_current = None;
                    } else {
                        self.line_add_anchor = Some(LineAnchor {
                            rect: rect_box.id(),
                            label: label.id,
                        });
                        self.line_add_current = response.interact_pointer_pos();
                    }
                }
            }
            rect_box.labels = labels;
            let id = rect_box.id();
            let response = ui
                .interact(
                    egui::Rect::from_center_size(egui_box.right_bottom(), egui::vec2(5.0, 5.0)),
                    Id::new(("rect_box_right_bottom", id)),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
            if response.dragged() {
                let delta = response.drag_delta();
                rect_box.drag_right_bottom(delta);
                editing = true;
            }
            let response = ui
                .interact(
                    egui::Rect::from_center_size(egui_box.right_top(), egui::vec2(5.0, 5.0)),
                    Id::new(("rect_box_right_top", id)),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNeSw);
            if response.dragged() {
                let delta = response.drag_delta();
                rect_box.drag_right_top(delta);
                editing = true;
            }
            let response = ui
                .interact(
                    egui::Rect::from_center_size(egui_box.left_bottom(), egui::vec2(5.0, 5.0)),
                    Id::new(("rect_box_left_bottom", id)),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNeSw);
            if response.dragged() {
                let delta = response.drag_delta();
                rect_box.drag_left_bottom(delta);
                editing = true;
            }
            let response = ui
                .interact(
                    egui::Rect::from_center_size(egui_box.left_top(), vec2(5.0, 5.0)),
                    Id::new(("rect_box_left_top", id)),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
            if response.dragged() {
                let delta = response.drag_delta();
                rect_box.drag_left_top(delta);
                editing = true;
            }
            let response = ui
                .interact(
                    egui_box.shrink(5.0),
                    Id::new(("rect_box", id)),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::Grab);
            if response.dragged() {
                let delta = response.drag_delta();
                rect_box.drag_center(delta);
                editing = true;
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
            } else if response.clicked() {
                self.selected = Some(rect_box.id());
            }
            if !editing {
                rect_box.complete_edit();
            }
            edit_active = edit_active || editing;
        }
        if let Some(edit_pos) = self.rect_boxes.iter().position(|r| r.edit_kind.is_some()) {
            let last = self.rect_boxes.len() - 1;
            self.rect_boxes.swap(edit_pos, last);
        }
        self.rect_boxes.retain(|r| !r.delete_pending());
        if let Some(start_anchor) = self.line_add_anchor
            && let Some(current_pos) = self.line_add_current
        {
            let points = std::iter::once(self.anchor(start_anchor))
                .chain(self.line_add_set.iter().copied())
                .chain(std::iter::once(current_pos))
                .collect::<Vec<_>>();
            for segment in points.windows(2) {
                ui.painter()
                    .line_segment([segment[0], segment[1]], (0.5, Color32::DARK_RED));
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.line_add_anchor = None;
            self.line_add_set.clear();
            self.line_add_current = None;
        }
    }

    pub fn update_ro(&mut self, ui: &mut Ui) {
        ui.output_mut(|o| o.cursor_icon = self.state.cursor());
        (-100..=100).map(|y| y as f32 * GRID_SIZE).for_each(|h| {
            ui.painter().hline(
                -10_000.0f32..=10_000.0f32,
                h,
                (0.15, Color32::LIGHT_GRAY.linear_multiply(0.3)),
            );
        });
        (-100..=100).map(|x| x as f32 * GRID_SIZE).for_each(|v| {
            ui.painter().vline(
                v,
                -10_000.0f32..=10_000.0f32,
                (0.15, Color32::LIGHT_GRAY.linear_multiply(0.3)),
            );
        });
        // Draw the wires
        for poly_line in self.poly_lines.iter() {
            if poly_line.editing.is_some() {
                let points = std::iter::once(self.anchor(poly_line.start))
                    .chain(poly_line.points.iter().map(|p| grid(*p)))
                    .chain(std::iter::once(self.anchor(poly_line.end)))
                    .collect::<Vec<_>>();
                ui.painter()
                    .add(egui::Shape::line(points, (1.0, Color32::DARK_GRAY)));
            }
            let points = std::iter::once(self.anchor(poly_line.start))
                .chain(poly_line.points.iter().cloned())
                .chain(std::iter::once(self.anchor(poly_line.end)))
                .collect::<Vec<_>>();
            ui.painter().add(egui::Shape::line(
                points.clone(),
                (1.0, Color32::DARK_GREEN),
            ));
            if let Some(edit_pos) = poly_line.edit_point() {
                ui.painter().circle(
                    edit_pos,
                    1.5,
                    Color32::LIGHT_GRAY,
                    (0.5, Color32::DARK_BLUE),
                );
            }
        }
        for rect_box in self.rect_boxes.iter_mut() {
            let mut is_resizing = false;
            if let State::MovingRect { rect, delta_pos } = self.state
                && rect == rect_box.id()
            {
                rect_box.render_moving(ui, delta_pos);
            } else if let State::ResizingRect {
                rect,
                delta_pos,
                mode,
            } = self.state
                && rect == rect_box.id()
            {
                is_resizing = true;
                rect_box.render_resizing(ui, mode, delta_pos);
            } else {
                rect_box.render_still(ui);
            }
            if self.state.selected_id() == Some(rect_box.id()) && !is_resizing {
                rect_box.render_control_frame(ui);
            }
            if let State::EditingName { rect } = self.state
                && rect == rect_box.id()
            {
                let rect_name_width = rect_box.name.len() as f32 * 10.0 + 10.0;
                let editor_position =
                    rect_box.inner.center_top() + vec2(-rect_name_width / 2.0, SHIM);
                let editor_rect = Rect::from_min_size(editor_position, vec2(rect_name_width, 20.0));
                let response = ui.place(
                    editor_rect,
                    TextEdit::singleline(&mut rect_box.name)
                        .font(egui::FontId::monospace(10.0))
                        .desired_width(f32::INFINITY),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.state = State::Selected { rect };
                } else {
                    response.request_focus();
                }
            }
        }
        if let Some(start_anchor) = self.line_add_anchor
            && let Some(current_pos) = self.line_add_current
        {
            let points = std::iter::once(self.anchor(start_anchor))
                .chain(self.line_add_set.iter().copied())
                .chain(std::iter::once(current_pos))
                .collect::<Vec<_>>();
            for segment in points.windows(2) {
                ui.painter()
                    .line_segment([segment[0], segment[1]], (0.5, Color32::DARK_RED));
            }
        }
        if let State::AddingRect { start_pos, end_pos } = &self.state {
            let rect = Rect::from_two_pos(*start_pos, *end_pos);
            ui.painter().rect(
                rect,
                3.0,
                Color32::TRANSPARENT,
                (1.0, Color32::DARK_RED),
                StrokeKind::Middle,
            );
        }
    }
    pub fn update_state(&mut self, response: Response) {
        match self.state {
            State::Idle => {
                if response.drag_started_by(egui::PointerButton::Primary)
                    && let Some(pos_start) = response.interact_pointer_pos()
                {
                    if let Some(rbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos_start))
                    {
                        self.state = State::MovingRect {
                            rect: rbox.id(),
                            delta_pos: vec2(0.0, 0.0),
                        };
                        return;
                    }
                    self.state = State::AddingRect {
                        start_pos: grid(pos_start),
                        end_pos: grid(pos_start),
                    };
                } else if response.is_pointer_button_down_on()
                    && response
                        .ctx
                        .input(|i| i.pointer.button_down(PointerButton::Secondary))
                {
                    self.state = State::Panning;
                }
                if response.clicked_by(PointerButton::Primary)
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    for rect_box in &self.rect_boxes {
                        if rect_box.inner.contains(pos) {
                            self.state = State::Selected {
                                rect: rect_box.id(),
                            };
                            break;
                        }
                    }
                }
            }
            State::Selected { rect } => {
                if response.clicked_by(PointerButton::Primary)
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    if let Some(bbox) = self.rect_mut(rect)
                        && bbox.control_pin_location_east().distance(pos) <= PORT_RADIUS
                    {
                        bbox.add_label(
                            "port".into(),
                            LabelSide::East,
                            bbox.next_port_offset(LabelSide::East),
                        );
                        return;
                    }
                    if let Some(bbox) = self.rect_mut(rect)
                        && bbox.control_pin_location_west().distance(pos) <= PORT_RADIUS
                    {
                        bbox.add_label(
                            "port".into(),
                            LabelSide::West,
                            bbox.next_port_offset(LabelSide::West),
                        );
                        return;
                    }
                    if let Some(bbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos)) {
                        self.state = State::Selected { rect: bbox.id() }
                    } else {
                        self.state = State::Idle;
                    }
                }
                if response.drag_started_by(PointerButton::Primary)
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    if let Some(bbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos)) {
                        self.state = State::MovingRect {
                            rect: bbox.id(),
                            delta_pos: vec2(0.0, 0.0),
                        };
                        return;
                    }
                    self.state = State::AddingRect {
                        start_pos: grid(pos),
                        end_pos: grid(pos),
                    }
                }
                if let Some(hover_pos) = response.hover_pos()
                    && let Some(bbox) = self.rect(rect)
                {
                    for mode in RESIZE_MODES {
                        if hover_pos.distance(control_corner(&bbox.inner, *mode))
                            < MOVE_HOVER_DISTANCE
                        {
                            self.state = State::PotentialResize { rect, mode: *mode };
                            return;
                        }
                    }
                }
                if response.double_clicked_by(PointerButton::Primary) {
                    self.state = State::EditingName { rect };
                }
            }
            State::PotentialResize { rect, mode } => {
                if let Some(hover_pos) = response.hover_pos()
                    && let Some(bbox) = self.rect(rect)
                    && hover_pos.distance(control_corner(&bbox.inner, mode)) >= MOVE_HOVER_DISTANCE
                {
                    self.state = State::Selected { rect };
                }
                if response.drag_started_by(egui::PointerButton::Primary) {
                    self.state = State::ResizingRect {
                        rect,
                        delta_pos: vec2(0.0, 0.0),
                        mode,
                    }
                }
            }
            State::ResizingRect {
                rect,
                mode,
                delta_pos,
            } => {
                if response.dragged_by(egui::PointerButton::Primary) {
                    let delta = response.drag_delta();
                    self.state = State::ResizingRect {
                        rect,
                        mode,
                        delta_pos: delta_pos + delta,
                    }
                } else if (response.drag_stopped_by(egui::PointerButton::Primary)
                    || !response.dragged())
                    && let Some(bbox) = self.rect_mut(rect)
                {
                    bbox.inner = grid_rect(resize_rect(&bbox.inner, mode, delta_pos));
                    self.state = State::Selected { rect };
                }
            }
            State::MovingRect { rect, delta_pos } => {
                if response.dragged_by(egui::PointerButton::Primary) {
                    let delta = response.drag_delta();
                    self.state = State::MovingRect {
                        rect,
                        delta_pos: delta_pos + delta,
                    }
                } else if (response.drag_stopped_by(egui::PointerButton::Primary)
                    || !response.dragged())
                    && let Some(rbox) = self.rect_mut(rect)
                {
                    rbox.inner = grid_rect(rbox.inner.translate(delta_pos));
                    self.state = State::Selected { rect };
                }
            }
            State::AddingRect { start_pos, end_pos } => {
                if response.dragged_by(egui::PointerButton::Primary)
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    self.state = State::AddingRect {
                        start_pos,
                        end_pos: round_even_grid(start_pos, pos),
                    };
                } else if response.drag_stopped_by(egui::PointerButton::Primary) {
                    let candidate_rect = Rect::from_two_pos(start_pos, end_pos);
                    if candidate_rect.width() > GRID_SIZE && candidate_rect.height() > GRID_SIZE {
                        let rect = self.add_rect_box(start_pos, end_pos);
                        self.state = State::Selected { rect: rect.id() };
                    } else {
                        self.state = State::Idle;
                    }
                }
                if response
                    .ctx
                    .input(|i| i.pointer.button_down(PointerButton::Secondary))
                {
                    self.state = State::Panning;
                }
            }
            State::Panning => {
                if response.drag_stopped() {
                    self.state = State::Idle;
                }
            }
            State::EditingName { rect } => {
                if response.clicked() {
                    self.state = State::Selected { rect };
                }
            }
            _ => {}
        }
    }
}

pub fn demo_drawing() -> Drawing {
    let mut drawing = Drawing::default();
    let box1 = drawing.add_rect_box(pos2(100.0, 100.0), pos2(180.0, 160.0));
    let anchor1 = box1.add_label(
        "i.1.write_logic".to_string(),
        LabelSide::East,
        -GRID_SIZE * 2.0,
    );
    let box2 = drawing.add_rect_box(pos2(300.0, 200.0), pos2(420.0, 290.0));
    let anchor2 = box2.add_label(
        "o.1.read_logic".to_string(),
        LabelSide::West,
        -GRID_SIZE * 2.0,
    );
    let anchor3 = box2.add_label("o.1.ram.read@.clock".to_string(), LabelSide::West, 0.0);
    let anchor4 = box2.add_label(
        "o.1.ram.read@.addr".to_string(),
        LabelSide::West,
        GRID_SIZE * 2.0,
    );
    drawing.add_line(anchor1, &[pos2(200.0, 100.0), pos2(200.0, 200.0)], anchor2);
    drawing.add_line(anchor3, &[pos2(320.0, 200.0)], anchor4);
    drawing
}
