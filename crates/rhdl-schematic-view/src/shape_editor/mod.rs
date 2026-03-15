use egui::{Color32, Id, Pos2, Rect, StrokeKind, Ui, Vec2, pos2, vec2};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct RectId(usize);

impl RectId {
    pub fn next(self) -> Self {
        RectId(self.0 + 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct LabelId(usize);

impl LabelId {
    pub fn next(self) -> Self {
        LabelId(self.0 + 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabelSide {
    East,
    West,
}

#[derive(Clone)]
pub struct Label {
    text: String,
    side: LabelSide,
    offset: f32,
    id: LabelId,
}

#[derive(Clone, Copy)]
pub enum ModificationKind {
    Move,
    ResizeRightBottom,
    ResizeRightTop,
    ResizeLeftBottom,
    ResizeLeftTop,
    LabelDrag,
}

pub struct SaveState {
    kind: ModificationKind,
    orig: Rect,
}

pub struct RectBox {
    inner: Rect,
    labels: Vec<Label>,
    edit_kind: Option<SaveState>,
    id: RectId,
    label_id: LabelId,
}

impl RectBox {
    pub fn new(inner: Rect, id: RectId) -> Self {
        Self {
            inner: snap(inner),
            labels: Vec::new(),
            edit_kind: None,
            id,
            label_id: LabelId::default(),
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
                ModificationKind::LabelDrag => grid_rect(self.inner),
            }
        } else {
            self.inner
        }
    }
    pub fn complete_edit(&mut self) {
        self.inner = self.predicted_rect();
        self.labels.iter_mut().for_each(|label| {
            label.offset = round_to_grid(label.offset);
        });
        self.edit_kind = None;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct LineAnchor {
    rect: RectId,
    label: LabelId,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct LineId(usize);

impl LineId {
    pub fn next(self) -> Self {
        LineId(self.0 + 1)
    }
}

pub struct PolyLine {
    start: LineAnchor,
    points: Vec<Pos2>,
    end: LineAnchor,
    id: LineId,
    editing: Option<usize>,
}

impl PolyLine {
    pub fn start_edit(&mut self, contact: Contact) {
        match contact.kind {
            ContactKind::Start(ndx) => {
                if ndx != 0 {
                    self.editing = Some(ndx - 1);
                }
            }
            ContactKind::Middle(ndx, pos) => {
                self.points.insert(ndx, pos);
                self.editing = Some(ndx);
            }
            ContactKind::End(ndx) => {
                if ndx != self.points.len() + 1 {
                    self.editing = Some(ndx - 1);
                }
            }
        }
    }
    pub fn shift_edit_point(&mut self, delta: Vec2) {
        if let Some(ndx) = self.editing {
            self.points[ndx] += delta;
        }
    }
    pub fn edit_point(&self) -> Option<Pos2> {
        self.editing.map(|ndx| self.points[ndx])
    }
    pub fn finish_edit(&mut self) {
        self.points.iter_mut().for_each(|p| *p = grid(*p));
        // Remove duplicate points
        self.points.dedup();
        // Remove points that are on the same horizontal or vertical
        // line as their neighbors, since they don't affect the shape of the line.
        let drop_flag = self
            .points
            .windows(3)
            .map(|w| {
                (w[0].x == w[1].x && w[1].x == w[2].x) || (w[0].y == w[1].y && w[1].y == w[2].y)
            })
            .collect::<Vec<_>>();
        // Drop the points based on drop_flag.  Drop_flag[0] corresponds to points[1], drop_flag[1] corresponds to points[2], etc.
        let points = std::mem::take(&mut self.points);
        let point_len = points.len();
        self.points = points
            .into_iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if i == 0 || i == point_len - 1 {
                    Some(p)
                } else if drop_flag[i - 1] {
                    None
                } else {
                    Some(p)
                }
            })
            .collect();
        self.editing = None;
    }
}

#[derive(Default)]
pub struct Drawing {
    rect_id: RectId,
    rect_boxes: Vec<RectBox>,
    poly_lines: Vec<PolyLine>,
}

pub const GRID_SIZE: f32 = 10.0;
const SHIM: f32 = 4.0;
const HIT_DISTANCE: f32 = 10.0;

fn snap(rect: Rect) -> Rect {
    Rect::from_min_max(
        grid(pos2(rect.min.x, rect.min.y)),
        grid(pos2(rect.max.x, rect.max.y)),
    )
}

fn round_to_grid(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

fn round_to_even_grid(value: f32) -> f32 {
    let n = (0.5 * (value / GRID_SIZE - 1.0)).round() as i32;
    (n as f32 * 2.0) * GRID_SIZE
}

fn grid(pos: Pos2) -> Pos2 {
    Pos2::new(round_to_grid(pos.x), round_to_grid(pos.y))
}

fn grid_vec(vec: Vec2) -> Vec2 {
    Vec2::new(round_to_grid(vec.x), round_to_grid(vec.y))
}

fn grid_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        grid(pos2(rect.min.x, rect.min.y)),
        grid(pos2(rect.max.x, rect.max.y)),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContactKind {
    Start(usize),
    Middle(usize, Pos2),
    End(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Contact {
    kind: ContactKind,
    distance: f32,
    location: Pos2,
}

// Find the closest point to the line segment along with the distance to it.
fn minimum_distance_and_location_on_segment(
    ndx: usize,
    start: Pos2,
    end: Pos2,
    target: Pos2,
    hit_dist: f32,
) -> Contact {
    let line_vec = end - start;
    let line_len = line_vec.length();
    if line_len == 0.0 {
        return Contact {
            kind: ContactKind::Start(ndx),
            distance: start.distance(target),
            location: start,
        };
    }
    let line_unit_vec = line_vec / line_len;
    let projection = (target - start).dot(line_unit_vec);
    if projection < hit_dist * 2.0 {
        Contact {
            kind: ContactKind::Start(ndx),
            distance: start.distance(target),
            location: start,
        }
    } else if projection > line_len - hit_dist * 2.0 {
        Contact {
            kind: ContactKind::End(ndx + 1),
            distance: end.distance(target),
            location: end,
        }
    } else {
        let closest_point = start + line_unit_vec * projection;
        Contact {
            kind: ContactKind::Middle(ndx, closest_point),
            distance: closest_point.distance(target),
            location: closest_point,
        }
    }
}

// Find the closest point to the polyline along with the distance to it.
fn minimum_distance_and_location(points: &[Pos2], target: Pos2) -> Contact {
    points
        .windows(2)
        .enumerate()
        .map(|(i, w)| minimum_distance_and_location_on_segment(i, w[0], w[1], target, 5.0))
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
        .unwrap()
}

impl Drawing {
    pub fn add_rect_box(&mut self, center: Pos2, size: Vec2) -> &mut RectBox {
        let center = grid(center);
        let size = grid_vec(size);
        let id = self.rect_id;
        self.rect_id = self.rect_id.next();
        self.rect_boxes
            .push(RectBox::new(Rect::from_center_size(center, size), id));
        self.rect_boxes.last_mut().unwrap()
    }
    pub fn anchor(&self, anchor: LineAnchor) -> Pos2 {
        let rect = self
            .rect_boxes
            .iter()
            .find(|r| r.id == anchor.rect)
            .unwrap();
        rect.anchor_point(anchor.label)
    }
    pub fn add_line(&mut self, start: LineAnchor, points: &[Pos2], end: LineAnchor) {
        let id = self
            .poly_lines
            .last()
            .map_or(LineId::default(), |l| l.id.next());
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
            ui.painter()
                .hline(-10_000.0f32..=10_000.0f32, h, (0.15, Color32::LIGHT_GRAY));
        });
        (-100..=100).map(|x| x as f32 * GRID_SIZE).for_each(|v| {
            ui.painter()
                .vline(v, -10_000.0f32..=10_000.0f32, (0.15, Color32::LIGHT_GRAY));
        });
        // Draw the wires
        let mut poly_lines = std::mem::take(&mut self.poly_lines);
        let mut is_editing = false;
        let mut bound_rect = Rect::NOTHING;
        for poly_line in poly_lines.iter_mut() {
            if poly_line.editing.is_some() {
                is_editing = true;
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

        if !is_editing {
            ui.painter()
                .rect_filled(bound_rect, 0.0, Color32::LIGHT_GRAY.linear_multiply(0.25));
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
            let stroke = if is_dragging {
                (2.0, Color32::DARK_RED)
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
            ui.painter().rect(
                egui_box,
                3.0,
                Color32::LIGHT_GRAY,
                stroke,
                StrokeKind::Middle,
            );
            let id = rect_box.id;
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
                    2.0,
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
                        Rect::from_center_size(text_pos + shift, vec2(3.0, 3.0)),
                        Id::new(("label", id, label_ndx)),
                        egui::Sense::click_and_drag(),
                    )
                    .on_hover_cursor(egui::CursorIcon::Grab);
                if response.hovered() {
                    ui.painter().circle(
                        text_pos + shift,
                        3.0,
                        Color32::TRANSPARENT,
                        (0.5, Color32::WHITE),
                    );
                }
                if response.dragged() {
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
            }
            rect_box.labels = labels;
            let id = rect_box.id;
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
                    egui::Rect::from_center_size(egui_box.left_top(), egui::vec2(5.0, 5.0)),
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
            }
            if !editing {
                rect_box.complete_edit();
            }
        }
        if let Some(edit_pos) = self.rect_boxes.iter().position(|r| r.edit_kind.is_some()) {
            let last = self.rect_boxes.len() - 1;
            self.rect_boxes.swap(edit_pos, last);
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
