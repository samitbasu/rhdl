use egui::{Color32, Id, Pos2, Rect, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::{
    geometry::minimum_distance_and_location,
    grid::{GRID_SIZE, SHIM, grid, round_to_grid},
    label::LabelSide,
    polyline::{LineId, PolyLine},
    rectbox::{LineAnchor, ModificationKind, RectBox, RectId},
};

pub enum State {
    Idle,
    AddingRect,
    ResizingRect,
    EditingLine,
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
}

const PORT_RADIUS: f32 = 3.0;
const HIT_DISTANCE: f32 = 10.0;

fn grid_vec(vec: Vec2) -> Vec2 {
    Vec2::new(round_to_grid(vec.x), round_to_grid(vec.y))
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
    pub fn add_rect_box(&mut self, center: Pos2, size: Vec2) -> &mut RectBox {
        let center = grid(center);
        let size = grid_vec(size);
        let id = self.rect_id;
        self.rect_id = self.rect_id.next();
        self.rect_boxes.push(RectBox::new(
            "Untitled".to_string(),
            Rect::from_center_size(center, size),
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
}

pub fn demo_drawing() -> Drawing {
    let mut drawing = Drawing::default();
    let box1 = drawing.add_rect_box(pos2(100.0, 100.0), vec2(80.0, 60.0));
    let anchor1 = box1.add_label(
        "i.1.write_logic".to_string(),
        LabelSide::East,
        -GRID_SIZE * 2.0,
    );
    let box2 = drawing.add_rect_box(pos2(300.0, 200.0), vec2(120.0, 90.0));
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
