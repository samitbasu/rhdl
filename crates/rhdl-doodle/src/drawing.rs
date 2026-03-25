use egui::{
    Color32, CursorIcon, PointerButton, Pos2, Rect, Response, StrokeKind, TextEdit, Ui, Vec2, pos2,
    vec2,
};

use crate::{
    grid::{
        GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_RADIUS, PORT_TEXT_SIZE, SHIM, TITLE_TEXT_SIZE, grid,
        grid_rect,
    },
    label::{LabelId, LabelSide},
    polyline::{LineId, PolyLine},
    rectbox::{LineAnchor, RectBox, RectId, control_corner, resize_rect},
    render::{
        FocusResult, GripState, draw_control_frame, draw_dragged_label, draw_moving_rect,
        draw_resizing_rect, estimate_bbox_for_label, get_hamburger_rect, render_rect_box,
    },
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
    PortDragged {
        rect: RectId,
        label: LabelId,
        delta_pos: Vec2,
    },
    PortLabelHovered {
        rect: RectId,
        label: LabelId,
    },
    PortLabelGripHovered {
        rect: RectId,
        label: LabelId,
    },
    EditingLabelText {
        rect: RectId,
        label: LabelId,
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
            State::PortLabelHovered { .. } => CursorIcon::Text,
            State::PortLabelGripHovered { .. } => CursorIcon::Grab,
            State::PortDragged { .. } => CursorIcon::Grabbing,
            _ => CursorIcon::Default,
        }
    }
    pub fn selected_id(&self) -> Option<RectId> {
        match self {
            State::Selected { rect } => Some(*rect),
            State::PotentialResize { rect, .. } | State::ResizingRect { rect, .. } => Some(*rect),
            State::PortDragged { rect, .. } => Some(*rect),
            State::PortLabelHovered { rect, .. } => Some(*rect),
            State::PortLabelGripHovered { rect, .. } => Some(*rect),
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
            if render_rect_box(rect_box, &self.state, ui) == FocusResult::LostFocus {
                self.state = State::Selected {
                    rect: rect_box.id(),
                };
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
        if response.drag_started() {
            eprintln!("Drag started this frame");
        }
        if response.drag_stopped() {
            eprintln!("Drag stopped this frame");
        }
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
                        && let Some(pin) = bbox.control_pin_location_east()
                        && pos.distance(pin) <= PORT_RADIUS
                        && let Some(next_offset) = bbox.next_port_offset(LabelSide::East)
                    {
                        bbox.add_label("port".into(), LabelSide::East, next_offset);
                        return;
                    }
                    if let Some(bbox) = self.rect_mut(rect)
                        && let Some(pin) = bbox.control_pin_location_west()
                        && pos.distance(pin) <= PORT_RADIUS
                        && let Some(next_offset) = bbox.next_port_offset(LabelSide::West)
                    {
                        bbox.add_label("port".into(), LabelSide::West, next_offset);
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
                    && let Some(hbox) = self.rect(rect)
                    && let Some(label) = hbox.labels.iter().find_map(|l| {
                        if hbox.control_pin_for_label(l.id)?.distance(pos) <= PORT_RADIUS {
                            Some(l)
                        } else {
                            None
                        }
                    })
                {
                    eprintln!("Starting to drag port {}", label.text);
                    self.state = State::PortDragged {
                        rect,
                        label: label.id,
                        delta_pos: vec2(0.0, 0.0),
                    };
                    return;
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
                    for label in &bbox.labels {
                        let label_bbox = estimate_bbox_for_label(bbox.inner, label);
                        if label_bbox.contains(hover_pos) {
                            eprintln!("Hovering over label {}", label.text);
                            self.state = State::PortLabelHovered {
                                rect,
                                label: label.id,
                            };
                            return;
                        }
                        let hamburger_rect = get_hamburger_rect(bbox.inner, label).expand(2.0);
                        if hamburger_rect.contains(hover_pos) {
                            eprintln!("Hovering over grip for label {}", label.text);
                            self.state = State::PortLabelGripHovered {
                                rect,
                                label: label.id,
                            };
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
            State::PortLabelHovered { rect, label } => {
                if let Some(hover_pos) = response.hover_pos()
                    && let Some(bbox) = self.rect(rect)
                    && let Some(label) = bbox.labels.iter().find(|l| l.id == label)
                {
                    let label_bbox = estimate_bbox_for_label(bbox.inner, label);
                    if !label_bbox.contains(hover_pos) {
                        self.state = State::Selected { rect };
                    }
                }
                if response.double_clicked_by(egui::PointerButton::Primary) {
                    self.state = State::EditingLabelText { rect, label };
                }
            }
            State::PortLabelGripHovered { rect, label } => {
                if response.drag_started_by(egui::PointerButton::Primary) || response.dragged() {
                    eprintln!("Starting to drag port label grip");
                    self.state = State::PortDragged {
                        rect,
                        label,
                        delta_pos: vec2(0.0, 0.0),
                    };
                    return;
                }
                if let Some(hover_pos) = response.hover_pos()
                    && let Some(bbox) = self.rect(rect)
                    && let Some(label) = bbox.labels.iter().find(|l| l.id == label)
                {
                    let hamburger_rect = get_hamburger_rect(bbox.inner, label).expand(2.0);
                    if !hamburger_rect.contains(hover_pos) {
                        eprintln!("No longer hovering over grip for label {}", label.text);
                        self.state = State::Selected { rect };
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
                        end_pos: grid(pos),
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
            State::EditingLabelText { rect, .. } => {
                if response.clicked() {
                    self.state = State::Selected { rect };
                }
            }
            State::PortDragged {
                rect,
                label,
                delta_pos,
            } => {
                if response.dragged_by(egui::PointerButton::Primary) {
                    let delta = response.drag_delta();
                    self.state = State::PortDragged {
                        rect,
                        label,
                        delta_pos: delta_pos + delta,
                    };
                } else if response.drag_stopped_by(egui::PointerButton::Primary)
                    || !response.dragged()
                {
                    if let Some(rbox) = self.rect_mut(rect) {
                        rbox.update_label_offset(label, delta_pos.y);
                    }
                    self.state = State::Selected { rect };
                }
                if let Some(pos) = response.interact_pointer_pos()
                    && let Some(rbox) = self.rect_mut(rect)
                {
                    let center_line = rbox.inner.center().x;
                    if let Some(label) = rbox.label_mut(label) {
                        if pos.x < center_line {
                            label.side = LabelSide::West;
                        } else {
                            label.side = LabelSide::East;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn demo() -> Self {
        demo_drawing()
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
