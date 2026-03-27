use egui::{Color32, PointerButton, Pos2, Rect, Response, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_RADIUS, grid, grid_rect},
    label::{LabelId, LabelSide},
    polyline::{LineId, PolyLine},
    rectbox::{LineAnchor, RectBox, RectId, control_corner, resize_rect},
    render::{
        FocusResult, estimate_bbox_for_label, get_control_pin_bbox, get_hamburger_rect,
        render_rect_box,
    },
    state::{
        AddingRect, AddingRoute, EditingLabelText, EditingName, MovingRect, PortDragged,
        PortLabelGripHovered, PortLabelHovered, PortPinHovered, PotentialResize, ResizeMode,
        ResizingRect, Route, RouteEdge, Selected, State,
    },
};

const RESIZE_MODES: &[ResizeMode] = &[
    ResizeMode::LeftTop,
    ResizeMode::RightTop,
    ResizeMode::LeftBottom,
    ResizeMode::RightBottom,
    ResizeMode::CenterTop,
    ResizeMode::CenterBottom,
];

#[derive(Default)]
pub struct Drawing {
    rect_id: RectId,
    rect_boxes: Vec<RectBox>,
    poly_lines: Vec<PolyLine>,
    routes: Vec<Route>,
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
    // When the geometry of a rect changes, we need to rescan the routes
    // to ensure that they remain orthogonal.  Because routes are
    // defined relative to the start point, the only place they can become
    // non-orthogonal is at the end point.  So we only need to check the last
    // edge of the route.  If the route contains only 1 edge, we need to add
    // an additional edge to allow for vertical/horizontal displacement of
    // the end point.
    pub fn reorthogonalize_routes(&mut self) {
        /*         for route in self.routes.iter_mut() {
                   let start_point = self.anchor(route.start);
                   let end_point = self.anchor(route.finish);
                   if route.edges.len() == 1 {
                       let to_add = match route.edges[0] {
                           RouteEdge::Horizontal(_) => RouteEdge::Vertical(0.0),
                           RouteEdge::Vertical(_) => RouteEdge::Horizontal(0.0),
                       };
                       route.edges.push(to_add);
                   }
                   // Compute the last point in the route (excluding the end point)
                   let last_point = route
                       .edges
                       .iter()
                       .fold(start_point, |current_pos, edge| match edge {
                           RouteEdge::Horizontal(offset) => pos2(current_pos.x + offset, current_pos.y),
                           RouteEdge::Vertical(offset) => pos2(current_pos.x, current_pos.y + offset),
                       });
                   // Calculate the delta run from the last point to the end point
                   let delta = end_point - last_point;
                   // If the last edge of the run is vertical,
               }
        */
    }
    pub fn render(&mut self, ui: &mut Ui) {
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
        for route in &self.routes {
            let start_pos = self.anchor(route.start);
            let end_pos = self.anchor(route.finish);
            let points = route.points(start_pos).collect();
            ui.painter()
                .add(egui::Shape::line(points, (1.0, Color32::DARK_BLUE)));
        }
        for rect_box in self.rect_boxes.iter_mut() {
            if render_rect_box(rect_box, &self.state, ui) == FocusResult::LostFocus {
                self.state = State::selected(rect_box.id());
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
        if let State::AddingRect(AddingRect { start_pos, end_pos }) = &self.state {
            let rect = Rect::from_two_pos(*start_pos, *end_pos);
            ui.painter().rect(
                rect,
                3.0,
                Color32::TRANSPARENT,
                (1.0, Color32::DARK_RED),
                StrokeKind::Middle,
            );
        }
        if let State::AddingRoute(inner) = &self.state {
            let points = inner.points(self.anchor(inner.anchor)).collect::<Vec<_>>();
            ui.painter().line(points, (0.5, Color32::DARK_RED));
        }
        if let State::ProposedRoute(inner) = &self.state {
            let start_pos = self.anchor(inner.start);
            let points = inner.points(start_pos).collect::<Vec<_>>();
            ui.painter().line(points, (0.5, Color32::DARK_RED));
            let end_pos = self.anchor(inner.finish);
            ui.painter().circle(
                end_pos,
                PORT_RADIUS,
                Color32::DARK_RED,
                (0.5, Color32::DARK_RED),
            );
        }
    }
    fn handle_idle_state(&self, response: Response) -> State {
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(pos_start) = response.interact_pointer_pos()
        {
            if let Some(rbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos_start)) {
                return State::moving_rect(rbox.id(), vec2(0.0, 0.0));
            }
            return State::adding_rect(pos_start, grid(pos_start));
        } else if response.is_pointer_button_down_on()
            && response
                .ctx
                .input(|i| i.pointer.button_down(PointerButton::Secondary))
        {
            return State::Panning;
        }
        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            for rect_box in &self.rect_boxes {
                if rect_box.inner.contains(pos) {
                    return State::selected(rect_box.id());
                }
            }
        }
        State::Idle
    }
    fn handle_selected_state(&mut self, rect: RectId, response: Response) -> State {
        if response.double_clicked_by(PointerButton::Primary) {
            return State::editing_name(rect);
        }
        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some(bbox) = self.rect_mut(rect)
                && let Some(pin) = bbox.control_pin_location_east()
                && pos.distance(pin) <= PORT_RADIUS
                && let Some(next_offset) = bbox.next_port_offset(LabelSide::East)
            {
                bbox.add_label("port".into(), LabelSide::East, next_offset);
                return State::selected(rect);
            }
            if let Some(bbox) = self.rect_mut(rect)
                && let Some(pin) = bbox.control_pin_location_west()
                && pos.distance(pin) <= PORT_RADIUS
                && let Some(next_offset) = bbox.next_port_offset(LabelSide::West)
            {
                bbox.add_label("port".into(), LabelSide::West, next_offset);
                return State::selected(rect);
            }
            if let Some(bbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos)) {
                return State::selected(bbox.id());
            } else {
                return State::idle();
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
            return State::port_dragged(rect, label.id, vec2(0.0, 0.0));
        }
        if response.drag_started_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some(bbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos)) {
                return State::moving_rect(bbox.id(), vec2(0.0, 0.0));
            }
            return State::adding_rect(pos, grid(pos));
        }
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
        {
            for mode in RESIZE_MODES {
                if hover_pos.distance(control_corner(&bbox.inner, *mode)) < MOVE_HOVER_DISTANCE {
                    return State::potential_resize(rect, *mode);
                }
            }
            for label in &bbox.labels {
                let label_bbox = estimate_bbox_for_label(bbox.inner, label);
                if label_bbox.contains(hover_pos) {
                    eprintln!("Hovering over label {}", label.text);
                    return State::port_label_hovered(rect, label.id);
                }

                let hamburger_rect = get_hamburger_rect(bbox.inner, label).expand(2.0);
                if hamburger_rect.contains(hover_pos) {
                    eprintln!("Hovering over grip for label {}", label.text);
                    return State::port_label_grip_hovered(rect, label.id);
                }
                let pin_location = get_control_pin_bbox(bbox.inner, label);
                if pin_location.contains(hover_pos) {
                    eprintln!("Hovering over pin for label {}", label.text);
                    return State::port_pin_hovered(rect, label.id);
                }
            }
        }
        State::selected(rect)
    }
    fn handle_potential_resize(&self, rect: RectId, mode: ResizeMode, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && hover_pos.distance(control_corner(&bbox.inner, mode)) >= MOVE_HOVER_DISTANCE
        {
            return State::selected(rect);
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            return State::resizing_rect(rect, mode, vec2(0.0, 0.0));
        }
        State::potential_resize(rect, mode)
    }
    fn handle_port_label_hovered(&self, rect: RectId, label: LabelId, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(label) = bbox.labels.iter().find(|l| l.id == label)
        {
            let label_bbox = estimate_bbox_for_label(bbox.inner, label);
            if !label_bbox.contains(hover_pos) {
                return State::selected(rect);
            }
        }
        if response.double_clicked_by(egui::PointerButton::Primary) {
            return State::editing_label_text(rect, label);
        }
        State::port_label_hovered(rect, label)
    }
    fn handle_port_label_grip_hovered(
        &self,
        rect: RectId,
        label: LabelId,
        response: Response,
    ) -> State {
        if response.drag_started_by(egui::PointerButton::Primary) || response.dragged() {
            eprintln!("Starting to drag port label grip");
            return State::port_dragged(rect, label, vec2(0.0, 0.0));
        }
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(label) = bbox.label(label)
        {
            let hamburger_rect = get_hamburger_rect(bbox.inner, label).expand(2.0);
            if !hamburger_rect.contains(hover_pos) {
                return State::selected(rect);
            }
        }
        State::port_label_grip_hovered(rect, label)
    }
    fn handle_port_pin_hovered(&self, rect: RectId, label: LabelId, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(label) = bbox.label(label)
        {
            let pin_location = get_control_pin_bbox(bbox.inner, label);
            if !pin_location.contains(hover_pos) {
                return State::selected(rect);
            }
        }
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(head) = response.interact_pointer_pos()
        {
            return AddingRoute {
                anchor: LineAnchor { rect, label },
                turns: Vec::new(),
                head,
            }
            .into();
        }
        State::port_pin_hovered(rect, label)
    }
    fn handle_resizing_rect(
        &mut self,
        rect: RectId,
        mode: ResizeMode,
        delta_pos: Vec2,
        response: Response,
    ) -> State {
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            return State::resizing_rect(rect, mode, delta_pos + delta);
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(bbox) = self.rect_mut(rect)
        {
            bbox.inner = grid_rect(resize_rect(&bbox.inner, mode, delta_pos));
            return State::selected(rect);
        }
        State::resizing_rect(rect, mode, delta_pos)
    }
    fn handle_moving_rect(&mut self, rect: RectId, delta_pos: Vec2, response: Response) -> State {
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            return State::moving_rect(rect, delta_pos + delta);
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(bbox) = self.rect_mut(rect)
        {
            bbox.inner = grid_rect(bbox.inner.translate(delta_pos));
            return State::selected(rect);
        }
        State::moving_rect(rect, delta_pos)
    }
    fn handle_adding_rect(&mut self, start_pos: Pos2, end_pos: Pos2, response: Response) -> State {
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            return State::adding_rect(grid(start_pos), grid(pos));
        } else if response.drag_stopped_by(egui::PointerButton::Primary) {
            let candidate_rect = Rect::from_two_pos(start_pos, end_pos);
            if candidate_rect.width() > GRID_SIZE && candidate_rect.height() > GRID_SIZE {
                let rect = self.add_rect_box(start_pos, end_pos);
                return State::selected(rect.id());
            } else {
                return State::idle();
            }
        }
        if response
            .ctx
            .input(|i| i.pointer.button_down(PointerButton::Secondary))
        {
            return State::panning();
        }
        State::adding_rect(start_pos, end_pos)
    }
    fn handle_panning(&self, response: Response) -> State {
        if response.drag_stopped() {
            return State::idle();
        }
        State::panning()
    }
    fn handle_editing_name(&self, rect: RectId, response: Response) -> State {
        if response.clicked() {
            return State::selected(rect);
        }
        State::editing_name(rect)
    }
    fn handle_editing_label_text(&self, rect: RectId, label: LabelId, response: Response) -> State {
        if response.clicked() {
            return State::selected(rect);
        }
        State::editing_label_text(rect, label)
    }
    fn handle_port_dragged(
        &mut self,
        rect: RectId,
        label: LabelId,
        delta_pos: Vec2,
        response: Response,
    ) -> State {
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
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            return State::port_dragged(rect, label, delta_pos + delta);
        } else if response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged() {
            if let Some(rbox) = self.rect_mut(rect) {
                rbox.update_label_offset(label, delta_pos.y);
            }
            return State::selected(rect);
        }
        State::port_dragged(rect, label, delta_pos)
    }
    fn handle_adding_route(&self, mut adding_route: AddingRoute, response: Response) -> State {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            let start_pos = self.anchor(adding_route.anchor);
            adding_route.add_point(start_pos, grid(pos));
            return adding_route.into();
        }
        if response.ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return State::selected(adding_route.anchor.rect);
        }
        if let Some(pos) = response.hover_pos() {
            let start_pos = self.anchor(adding_route.anchor);
            let last_point = adding_route.last_point(start_pos);
            let delta = grid(pos) - last_point;
            let delta = if delta.x.abs() > delta.y.abs() {
                vec2(delta.x, 0.0)
            } else {
                vec2(0.0, delta.y)
            };
            let head = last_point + delta;

            if let Some(anchor) = self
                .rect_boxes
                .iter()
                .flat_map(|rect| rect.anchors())
                .find(|anchor| self.anchor(*anchor).distance(pos) < PORT_RADIUS)
            {
                return Route::from_adding_route(start_pos, last_point, adding_route, anchor)
                    .into();
            }

            return AddingRoute {
                head,
                ..adding_route
            }
            .into();
        }
        adding_route.into()
    }
    fn handle_proposed_route(&mut self, proposed_route: Route, response: Response) -> State {
        if response.clicked_by(egui::PointerButton::Primary) {
            self.routes.push(proposed_route);
            return State::idle();
        }
        if let Some(pos) = response.hover_pos() {
            let last_point = self.anchor(proposed_route.finish);
            if last_point.distance(pos) > PORT_RADIUS {
                return AddingRoute {
                    anchor: proposed_route.start,
                    turns: proposed_route.turns.into(),
                    head: pos,
                }
                .into();
            }
        }
        proposed_route.into()
    }
    pub fn update_state(&mut self, response: Response) {
        let old_state = std::mem::take(&mut self.state);
        self.state = match old_state {
            State::Idle => self.handle_idle_state(response),
            State::Selected(Selected { rect }) => self.handle_selected_state(rect, response),
            State::PotentialResize(PotentialResize { rect, mode }) => {
                self.handle_potential_resize(rect, mode, response)
            }

            State::PortLabelHovered(PortLabelHovered { rect, label }) => {
                self.handle_port_label_hovered(rect, label, response)
            }
            State::PortLabelGripHovered(PortLabelGripHovered { rect, label }) => {
                self.handle_port_label_grip_hovered(rect, label, response)
            }
            State::PortPinHovered(PortPinHovered { rect, label }) => {
                self.handle_port_pin_hovered(rect, label, response)
            }
            State::ResizingRect(ResizingRect {
                rect,
                mode,
                delta_pos,
            }) => self.handle_resizing_rect(rect, mode, delta_pos, response),
            State::MovingRect(MovingRect { rect, delta_pos }) => {
                self.handle_moving_rect(rect, delta_pos, response)
            }
            State::AddingRect(AddingRect { start_pos, end_pos }) => {
                self.handle_adding_rect(start_pos, end_pos, response)
            }
            State::Panning => self.handle_panning(response),
            State::EditingName(EditingName { rect }) => self.handle_editing_name(rect, response),
            State::EditingLabelText(EditingLabelText { rect, label }) => {
                self.handle_editing_label_text(rect, label, response)
            }
            State::PortDragged(PortDragged {
                rect,
                label,
                delta_pos,
            }) => self.handle_port_dragged(rect, label, delta_pos, response),
            State::AddingRoute(inner) => self.handle_adding_route(inner, response),
            State::ProposedRoute(inner) => self.handle_proposed_route(inner, response),
        };
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
