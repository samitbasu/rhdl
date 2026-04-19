use std::collections::BTreeMap;

use egui::{
    Color32, PointerButton, Pos2, Rect, Response, StrokeKind, TextEdit, Ui, Vec2, pos2, vec2,
};

use crate::{
    grid::{
        GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_RADIUS, ROUTE_TEXT_SIZE, grid_rect, round_to_grid,
        snap_to_grid,
    },
    label::{LabelId, LabelSide},
    rectbox::{LineAnchor, RectBox, RectId, control_corner, resize_rect},
    render::{
        FocusResult, estimate_bbox_for_label, get_control_pin_bbox, get_hamburger_rect,
        render_path_with_chamfered_corners, render_rect_box,
    },
    router_ng::{COST_ZERO, Point, RouterNGBuilder, SegmentKind, TaggedPoint, WIRE_COST},
    state::{
        AddTextButtonHovered, AddingRect, AutoRoute, EditingLabelText, EditingName,
        EditingRouteLabelText, InProgressAutoRoute, MovingRect, PortDragged, PortLabelGripHovered,
        PortLabelHovered, PortPinHovered, PotentialResize, ProposedAutoRoute, ResizeMode,
        ResizingRect, RouteCornerHovered, RouteDirection, RouteEdgeDragged, RouteEdgeHovered,
        RouteHovered, RouteId, RouteLabelHovered, RouteSelected, Selected, State, Waypoint,
        WaypointDragged, WaypointHovered, next_waypoint_id,
    },
    turtle::Mark,
};

const GRIP_SHIM: f32 = 4.0;

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
    auto_routes: BTreeMap<RouteId, AutoRoute>,
    pub selected: Option<RectId>,
    pub line_add_anchor: Option<LineAnchor>,
    pub line_add_set: Vec<Pos2>,
    pub line_add_current: Option<Pos2>,
    marks: Vec<Mark>,
    state: State,
    auto_route: Vec<TaggedPoint>,
    reroute: bool,
    ripup_set: Vec<RouteId>,
    route_id: RouteId,
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
    pub fn routing_box(&self, rect: RectId) -> Rect {
        if let State::MovingRect(inner) = &self.state
            && inner.rect == rect
        {
            grid_rect(self.rect(rect).unwrap().inner.translate(inner.delta_pos))
        } else if let State::ResizingRect(inner) = &self.state
            && inner.rect == rect
        {
            grid_rect(resize_rect(
                &self.rect(rect).unwrap().inner,
                inner.mode,
                inner.delta_pos,
            ))
        } else {
            self.rect(rect).unwrap().inner
        }
    }
    pub fn anchor(&self, anchor: LineAnchor) -> Pos2 {
        let effective_rect = self.routing_box(anchor.rect);
        if let State::PortDragged(inner) = &self.state
            && anchor.rect == inner.rect
            && anchor.label == inner.label
        {
            let center_line = effective_rect.center().x;
            let current_pos =
                self.rect(anchor.rect).unwrap().anchor_point(anchor.label) + inner.delta_pos;
            let anchor_x = if current_pos.x < center_line {
                effective_rect.left() - GRID_SIZE
            } else {
                effective_rect.right() + GRID_SIZE
            };
            let anchor_y = round_to_grid(current_pos.y);
            snap_to_grid(Pos2::new(anchor_x, anchor_y))
        } else {
            self.rect(anchor.rect)
                .unwrap()
                .anchor_point_with_rect(effective_rect, anchor.label)
        }
    }
    pub fn iter_anchors(&self) -> impl Iterator<Item = LineAnchor> + '_ {
        self.rect_boxes.iter().flat_map(|rect| rect.anchors())
    }
    pub fn iter_anchor_positions(&self) -> impl Iterator<Item = (LineAnchor, Pos2)> + '_ {
        self.rect_boxes.iter().flat_map(move |rect| {
            rect.anchors()
                .map(move |anchor| (anchor, self.anchor(anchor)))
        })
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
        for route in self.auto_routes.values() {
            let points = render_path_with_chamfered_corners(&route.points());
            points.render(ui, (1.0, Color32::DARK_GREEN));
            // for edge in &route.edges {
            //     let label_pos: Pos2 = edge.center();
            //     if let Some(label_id) = &edge.label
            //         && let Some(label) = route.label(*label_id)
            //     {
            //         ui.painter().text(
            //             label_pos,
            //             egui::Align2::CENTER_BOTTOM,
            //             &label.text,
            //             egui::FontId::monospace(ROUTE_TEXT_SIZE),
            //             Color32::LIGHT_GREEN,
            //         );
            //     }
            // }
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
        if let State::InProgressAutoRoute(inner) = &self.state {
            let points = self
                .auto_route
                .iter()
                .map(|p| p.pos.into())
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&points);
            points.render(ui, (0.5, Color32::LIGHT_YELLOW));
            inner.waypoints.iter().for_each(|wp| {
                ui.painter()
                    .circle_filled(wp.pos, PORT_RADIUS, Color32::LIGHT_YELLOW);
            });
        }
        if let State::ProposedAutoRoute(inner) = &self.state {
            let points = self
                .auto_route
                .iter()
                .map(|p| p.pos.into())
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&points);
            points.render(ui, (1.5, Color32::LIGHT_YELLOW));
            let start_pos = self.anchor(inner.start);
            ui.painter().circle(
                start_pos,
                PORT_RADIUS,
                Color32::DARK_RED,
                (0.5, Color32::DARK_RED),
            );
            let end_pos = self.anchor(inner.finish);
            ui.painter().circle(
                end_pos,
                PORT_RADIUS,
                Color32::DARK_RED,
                (0.5, Color32::DARK_RED),
            );
        }
        if let State::RouteHovered(target) = &self.state {
            let route = &self.auto_routes[&target.id];
            let points = render_path_with_chamfered_corners(&route.points());
            points.render(ui, (2.5, Color32::DARK_GREEN));
            // ui.painter().circle(
            //     target.pos,
            //     PORT_RADIUS,
            //     Color32::LIGHT_GRAY.linear_multiply(0.5),
            //     (0.5, Color32::BLACK),
            // );
            // for tp in route.text_anchors() {
            //     // Draw a T in a box to indicate that this is a text anchor.
            //     ui.painter().text(
            //         tp,
            //         egui::Align2::CENTER_CENTER,
            //         "T",
            //         egui::FontId::monospace(ROUTE_TEXT_SIZE),
            //         Color32::LIGHT_GREEN,
            //     );
            //     ui.painter().rect(
            //         Rect::from_center_size(tp, vec2(ROUTE_TEXT_SIZE, ROUTE_TEXT_SIZE)),
            //         3.0,
            //         Color32::TRANSPARENT,
            //         (0.5, Color32::LIGHT_GREEN),
            //         StrokeKind::Middle,
            //     );
            // }
        }
        if let State::RouteSelected(target) = &self.state {
            let route = &self.auto_routes[&target.id];
            let points = render_path_with_chamfered_corners(&route.points());
            points.render(ui, (2.5, Color32::LIGHT_GREEN));
            for wp in &route.waypoints {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
            for tp in &route.text_anchors() {
                // Draw a T in a box to indicate that this is a text anchor.
                ui.painter().text(
                    *tp,
                    egui::Align2::CENTER_CENTER,
                    "T",
                    egui::FontId::monospace(ROUTE_TEXT_SIZE),
                    Color32::LIGHT_GREEN,
                );
                ui.painter().rect(
                    Rect::from_center_size(*tp, vec2(ROUTE_TEXT_SIZE, ROUTE_TEXT_SIZE)),
                    3.0,
                    Color32::TRANSPARENT,
                    (0.5, Color32::LIGHT_GREEN),
                    StrokeKind::Middle,
                );
            }
        }
        if let State::RouteEdgeHovered(target) = &self.state {
            let route = &self.auto_routes[&target.id];
            let points = render_path_with_chamfered_corners(&route.points());
            points.render(ui, (2.5, Color32::LIGHT_GREEN.gamma_multiply(0.2)));
            for wp in &route.waypoints {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN
                        .linear_multiply(0.5)
                        .gamma_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
            for wp in &route.waypoints {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
            if let Some(edge) = route.edge(target.edge_index) {
                let edge_start: Pos2 = edge.start;
                let edge_end: Pos2 = edge.end;
                let edge_dir = (edge_end - edge_start).normalized();
                let edge_start = edge_start + edge_dir * PORT_RADIUS;
                let edge_end = edge_end - edge_dir * PORT_RADIUS;
                ui.painter().line_segment(
                    [edge_start, edge_end],
                    (2.5, Color32::LIGHT_GREEN.gamma_multiply(0.7)),
                );
            }
        }
        if let State::RouteCornerHovered(target) = &self.state {
            let route = &self.auto_routes[&target.id];
            for wp in &route.waypoints {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
            if let Some(edge_1) = route.edge(target.edge_1) {
                let edge_1_end: Pos2 = edge_1.end;
                ui.painter().circle(
                    edge_1_end,
                    PORT_RADIUS,
                    Color32::LIGHT_RED.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
        }
        if let State::RouteEdgeDragged(target) = &self.state {
            let projected_path = self.auto_routes[&target.id]
                .points()
                .into_iter()
                .map(snap_to_grid)
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&projected_path);
            points.render(ui, (1.5, Color32::GRAY.gamma_multiply(0.2)));
        }
        if let State::WaypointHovered(target) = &self.state {
            let route = &self.auto_routes[&target.route];
            let points = render_path_with_chamfered_corners(&route.points());
            points.render(ui, (2.5, Color32::DARK_GREEN));
            if let Some(wp) = route.waypoint(target.waypoint) {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::WHITE),
                );
            }
        }
        if let State::AddTextButtonHovered(target) = &self.state {
            let route = &self.auto_routes[&target.route];
            let points = render_path_with_chamfered_corners(&route.points());
            points.render(ui, (2.5, Color32::DARK_GREEN));
            if let Some(tp) = route.text_anchor(target.edge_id) {
                // Draw a T in a box to indicate that this is a text anchor.
                ui.painter().rect(
                    Rect::from_center_size(tp, vec2(ROUTE_TEXT_SIZE, ROUTE_TEXT_SIZE)),
                    3.0,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::LIGHT_GREEN),
                    StrokeKind::Middle,
                );
                ui.painter().text(
                    tp,
                    egui::Align2::CENTER_CENTER,
                    "T",
                    egui::FontId::monospace(ROUTE_TEXT_SIZE),
                    Color32::WHITE,
                );
            }
        }
        if let State::WaypointDragged(target) = &self.state {
            let route = &self.auto_routes[&target.route];
            if let Some(wp) = route.waypoint(target.waypoint) {
                let points = render_path_with_chamfered_corners(&route.points());
                points.render(ui, (2.5, Color32::DARK_GREEN));
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (1.0, Color32::WHITE),
                );
            }
        }
        if let State::EditingRouteLabelText(target) = &self.state
            && let Some(route) = self.auto_routes.get_mut(&target.id)
            && let Some(edge) = route.edge_mut(target.edge_index)
        {
            let edge_center: Pos2 = edge.center();
            let editor_width = 25.0;
            let editor_position =
                Rect::from_center_size(edge_center, vec2(editor_width, ROUTE_TEXT_SIZE));
            if let Some(wire_label) = route.label_mut(target.label_id) {
                let response = ui.place(
                    editor_position,
                    TextEdit::singleline(&mut wire_label.text)
                        .font(egui::FontId::monospace(ROUTE_TEXT_SIZE))
                        .desired_width(f32::INFINITY),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // Do something.
                }
            }
        }
    }
    fn handle_route_hover_check(&self, id: RouteId, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos()
            && let Some(route) = self.auto_routes.get(&id)
        {
            for waypoint in &route.waypoints {
                if waypoint.pos.distance(hover_pos) <= PORT_RADIUS * 1.5 {
                    return WaypointHovered {
                        route: id,
                        waypoint: waypoint.id,
                    }
                    .into();
                }
            }
            if let Some((edge_1, edge_2)) = route.hovered_corner(hover_pos) {
                return RouteCornerHovered { id, edge_1, edge_2 }.into();
            }
            if let Some(edge_id) = route.hovered_edge(hover_pos)
                && let Some(edge) = route.edge(edge_id)
            {
                return RouteEdgeHovered {
                    id,
                    edge_index: edge_id,
                    direction: edge.direction(),
                }
                .into();
            }
            if let Some(edge_id) = route.hit_text_anchor(hover_pos) {
                return AddTextButtonHovered { route: id, edge_id }.into();
            }
        }
        RouteSelected { id }.into()
    }
    fn handle_idle_state(&self, response: Response) -> State {
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(pos_start) = response.interact_pointer_pos()
        {
            if let Some(rbox) = self.rect_boxes.iter().find(|r| r.inner.contains(pos_start)) {
                return State::moving_rect(rbox.id(), vec2(0.0, 0.0));
            }
            return State::adding_rect(pos_start, snap_to_grid(pos_start));
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
            for (id, route) in &self.auto_routes {
                if route.hovered_edge(pos).is_some() {
                    return RouteSelected { id: *id }.into();
                }
            }
        }
        if let Some(hover_pos) = response.hover_pos() {
            if let Some(id) = self.route_hit(hover_pos) {
                return RouteHovered { id }.into();
            }
        }
        State::Idle
    }
    fn route_hit(&self, pos: Pos2) -> Option<RouteId> {
        for (&id, route) in &self.auto_routes {
            if route.hovered_edge(pos).is_some() {
                return Some(id);
            }
        }
        None
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
                self.reroute = true;
                return State::selected(rect);
            }
            if let Some(bbox) = self.rect_mut(rect)
                && let Some(pin) = bbox.control_pin_location_west()
                && pos.distance(pin) <= PORT_RADIUS
                && let Some(next_offset) = bbox.next_port_offset(LabelSide::West)
            {
                bbox.add_label("port".into(), LabelSide::West, next_offset);
                self.reroute = true;
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
            return State::adding_rect(pos, snap_to_grid(pos));
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

                let hamburger_rect = get_hamburger_rect(bbox.inner, label).expand(GRIP_SHIM);
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
            && let Some(label) = bbox.label(label)
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
    fn handle_route_label_hovered(&self, route: RouteLabelHovered, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos() {
            let Some(edge_index) = self.auto_routes[&route.id].hovered_edge(hover_pos) else {
                return State::Idle;
            };
            if edge_index != route.edge_index {
                return State::Idle;
            }
        }
        route.into()
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
            let hamburger_rect = get_hamburger_rect(bbox.inner, label).expand(GRIP_SHIM);
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
            && let Some(pos) = response.interact_pointer_pos()
        {
            return InProgressAutoRoute {
                start: LineAnchor { rect, label },
                waypoints: Vec::new(),
                head: pos,
            }
            .into();
        }
        State::port_pin_hovered(rect, label)
    }
    fn handle_route_corner_hovered(
        &mut self,
        inner: RouteCornerHovered,
        response: Response,
    ) -> State {
        if (response.drag_started_by(egui::PointerButton::Primary) || response.dragged())
            && let Some(pos) = response.interact_pointer_pos()
        {
            eprintln!("Starting to drag route corner");
            let route = self.auto_routes.get_mut(&inner.id).unwrap();
            if let Some(waypoint_id) = route.hit_waypoint(pos, PORT_RADIUS) {
                route.lock_waypoint(waypoint_id);
                return WaypointDragged {
                    route: inner.id,
                    waypoint: waypoint_id,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
            if let Some(edge) = route.edge(inner.edge_1) {
                let waypoint_id = route.add_waypoint(snap_to_grid(pos));
                route.lock_waypoint(waypoint_id);
                return WaypointDragged {
                    route: inner.id,
                    waypoint: waypoint_id,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
        }
        self.handle_route_hover_check(inner.id, response)
    }
    fn handle_route_hovered(&mut self, inner: RouteHovered, response: Response) -> State {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some(id) = self.route_hit(pos) {
                return RouteSelected { id }.into();
            }
        }
        if let Some(hover_pos) = response.hover_pos()
            && let Some(id) = self.route_hit(hover_pos)
        {
            return RouteHovered { id }.into();
        }
        return State::Idle;
    }
    fn handle_route_selected(&self, inner: RouteSelected, response: Response) -> State {
        if response.clicked_by(egui::PointerButton::Primary) {
            return State::Idle;
        }
        self.handle_route_hover_check(inner.id, response)
    }
    fn handle_route_edge_hovered(&mut self, target: RouteEdgeHovered, response: Response) -> State {
        if (response.drag_started_by(egui::PointerButton::Primary) || response.dragged())
            && let Some(route) = self.auto_routes.get_mut(&target.id)
            && let Some(edge) = route.edge(target.edge_index).cloned()
        {
            eprintln!("Starting to drag route edge");
            eprintln!("Waypoints before adding pair: {:?}", route.waypoints);
            eprintln!("Raw edge: {:?}", edge);
            let wp1 = route.add_waypoint(edge.waypoint_position_start());
            let wp2 = route.add_waypoint(edge.waypoint_position_end());
            route.lock_waypoint(wp1);
            route.lock_waypoint(wp2);
            self.reroute = true;
            self.ripup_set.push(target.id);
            if wp1 == wp2 {
                return WaypointDragged {
                    route: target.id,
                    waypoint: wp1,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
            return RouteEdgeDragged {
                id: target.id,
                direction: edge.direction(),
                start_waypoint: wp1,
                end_waypoint: wp2,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        self.handle_route_hover_check(target.id, response)
    }
    fn handle_add_text_button_hovered(
        &mut self,
        inner: AddTextButtonHovered,
        response: Response,
    ) -> State {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(route) = self.auto_routes.get_mut(&inner.route)
            && let Some(edge) = route.edge(inner.edge_id)
        {
            let center = edge.center();
            let wp_id = route.add_waypoint(center);
            route.lock_waypoint(wp_id);
            self.reroute = true;
            self.ripup_set.push(inner.route);
            let label = route.allocate_label();
            if let Some(wp) = route.waypoint_mut(wp_id) {
                wp.label = Some(label);
                return EditingRouteLabelText {
                    id: inner.route,
                    edge_index: inner.edge_id,
                    label_id: label,
                }
                .into();
            }
        }
        self.handle_route_hover_check(inner.route, response)
    }
    fn handle_waypoint_hovered(&mut self, inner: WaypointHovered, response: Response) -> State {
        if response.drag_started_by(egui::PointerButton::Primary) || response.dragged() {
            eprintln!("Starting to drag waypoint");
            self.auto_routes
                .get_mut(&inner.route)
                .unwrap()
                .lock_waypoint(inner.waypoint);
            return WaypointDragged {
                route: inner.route,
                waypoint: inner.waypoint,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        self.handle_route_hover_check(inner.route, response)
    }
    fn handle_waypoint_dragged(&mut self, inner: WaypointDragged, response: Response) -> State {
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            let route = self.auto_routes.get_mut(&inner.route).unwrap();
            if let Some(wp) = route.waypoint_mut(inner.waypoint) {
                wp.pos += delta;
            }
            self.reroute = true;
            self.ripup_set.push(inner.route);
            return inner.into();
        } else if response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged() {
            let route = self.auto_routes.get_mut(&inner.route).unwrap();
            route.waypoints.iter_mut().for_each(|wp| {
                wp.pos = snap_to_grid(wp.pos);
                wp.unlock();
            });
            self.reroute = true;
            return RouteSelected { id: inner.route }.into();
        }
        inner.into()
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
            return State::adding_rect(snap_to_grid(start_pos), snap_to_grid(pos));
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
    fn handle_editing_route_label_text(
        &self,
        inner: EditingRouteLabelText,
        response: Response,
    ) -> State {
        if response.clicked() {
            return State::Idle;
        }
        inner.into()
    }
    fn handle_route_edge_dragged(&mut self, target: RouteEdgeDragged, response: Response) -> State {
        if response.dragged_by(egui::PointerButton::Primary) {
            let mut delta = response.drag_delta();
            if target.direction == RouteDirection::Horizontal {
                delta.x = 0.0;
            } else {
                delta.y = 0.0;
            }
            let route = self.auto_routes.get_mut(&target.id).unwrap();
            route.waypoint_mut(target.start_waypoint).unwrap().pos += delta;
            route.waypoint_mut(target.end_waypoint).unwrap().pos += delta;
            self.reroute = true;
            self.ripup_set.push(target.id);
            return target.into();
        } else if response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged() {
            let route = self.auto_routes.get_mut(&target.id).unwrap();
            if let Some(start_wp) = route.waypoint_mut(target.start_waypoint) {
                start_wp.pos = snap_to_grid(start_wp.pos);
                start_wp.unlock();
            }
            if let Some(end_wp) = route.waypoint_mut(target.end_waypoint) {
                end_wp.pos = snap_to_grid(end_wp.pos);
                end_wp.unlock();
            }
            self.reroute = true;
            return RouteSelected { id: target.id }.into();
        }
        State::RouteEdgeDragged(target)
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
            self.reroute = true;
            return State::selected(rect);
        }
        State::port_dragged(rect, label, delta_pos)
    }
    fn handle_in_progress_auto_routing(
        &mut self,
        mut auto_route: InProgressAutoRoute,
        response: Response,
    ) -> State {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            let id = next_waypoint_id(&auto_route.waypoints);
            auto_route.waypoints.push(Waypoint {
                pos: snap_to_grid(pos),
                id,
                label: None,
                locked: true,
            });
            return auto_route.into();
        }
        if let Some(pos) = response.hover_pos() {
            auto_route.head = pos;
            if let Some((tail, _)) = self
                .iter_anchor_positions()
                .find(|&(_, anchor_pos)| anchor_pos.distance(pos) < PORT_RADIUS)
                && tail != auto_route.start
            {
                return ProposedAutoRoute {
                    start: auto_route.start,
                    waypoints: auto_route.waypoints,
                    finish: tail,
                }
                .into();
            }
        }
        State::InProgressAutoRoute(auto_route)
    }
    fn handle_proposed_auto_route(
        &mut self,
        mut proposed_route: ProposedAutoRoute,
        response: Response,
    ) -> State {
        if response.clicked_by(egui::PointerButton::Primary) {
            let id = self.route_id;
            self.route_id = self.route_id.next();
            let mut waypoints = std::mem::take(&mut proposed_route.waypoints);
            waypoints.iter_mut().for_each(|wp| wp.unlock());
            let mut route = AutoRoute::build(
                proposed_route.start,
                proposed_route.finish,
                &self.auto_route,
                &waypoints,
            );
            route.update_waypoints();
            self.auto_routes.insert(id, route);
            return RouteSelected { id }.into();
        }
        if let Some(pos) = response.hover_pos() {
            if let Some((tail, _)) = self
                .iter_anchor_positions()
                .find(|&(_, anchor_pos)| anchor_pos.distance(pos) < PORT_RADIUS)
                && tail != proposed_route.start
            {
                return ProposedAutoRoute {
                    start: proposed_route.start,
                    waypoints: proposed_route.waypoints,
                    finish: tail,
                }
                .into();
            }
            return InProgressAutoRoute {
                start: proposed_route.start,
                waypoints: proposed_route.waypoints,
                head: pos,
            }
            .into();
        }
        proposed_route.into()
    }
    pub fn update_state(&mut self, response: Response) {
        let old_state = std::mem::take(&mut self.state);
        let old_state_copy = old_state.clone();
        let mut route_fixup = false;
        self.reroute = false;
        self.state = match old_state {
            State::Idle => self.handle_idle_state(response),
            State::RouteHovered(route) => self.handle_route_hovered(route, response),
            State::RouteSelected(route) => self.handle_route_selected(route, response),
            State::RouteLabelHovered(inner) => self.handle_route_label_hovered(inner, response),
            State::RouteEdgeHovered(route) => self.handle_route_edge_hovered(route, response),
            State::RouteCornerHovered(route) => self.handle_route_corner_hovered(route, response),
            State::WaypointHovered(inner) => self.handle_waypoint_hovered(inner, response),
            State::AddTextButtonHovered(inner) => {
                self.handle_add_text_button_hovered(inner, response)
            }
            State::WaypointDragged(inner) => self.handle_waypoint_dragged(inner, response),
            State::RouteEdgeDragged(inner) => {
                route_fixup = true;
                self.handle_route_edge_dragged(inner, response)
            }
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
            }) => {
                route_fixup = true;
                self.handle_resizing_rect(rect, mode, delta_pos, response)
            }
            State::MovingRect(MovingRect { rect, delta_pos }) => {
                route_fixup = true;
                self.handle_moving_rect(rect, delta_pos, response)
            }
            State::AddingRect(AddingRect { start_pos, end_pos }) => {
                route_fixup = true;
                self.handle_adding_rect(start_pos, end_pos, response)
            }
            State::Panning => self.handle_panning(response),
            State::EditingName(EditingName { rect }) => self.handle_editing_name(rect, response),
            State::EditingLabelText(EditingLabelText { rect, label }) => {
                self.handle_editing_label_text(rect, label, response)
            }
            State::EditingRouteLabelText(inner) => {
                self.handle_editing_route_label_text(inner, response)
            }
            State::PortDragged(PortDragged {
                rect,
                label,
                delta_pos,
            }) => {
                route_fixup = true;
                self.handle_port_dragged(rect, label, delta_pos, response)
            }
            State::InProgressAutoRoute(inner) => {
                route_fixup = true;
                self.handle_in_progress_auto_routing(inner, response)
            }
            State::ProposedAutoRoute(inner) => {
                route_fixup = true;
                self.handle_proposed_auto_route(inner, response)
            }
        };
        if self.state != old_state_copy {
            eprintln!(
                "State changed from {:?} to {:?}",
                old_state_copy, self.state
            );
        }
        if route_fixup || self.reroute {
            self.update_graph();
        }
    }
    fn update_graph(&mut self) {
        let mut builder = RouterNGBuilder::default();
        for rect_box in &self.rect_boxes {
            let effective_rect = self.routing_box(rect_box.id());
            builder.add_block(effective_rect.left_top(), effective_rect.right_bottom());
            for label in &rect_box.labels {
                let anchor_pos = rect_box.anchor_point_with_rect(effective_rect, label.id);
                let anchor_pos = match label.side {
                    LabelSide::East => anchor_pos + vec2(GRID_SIZE, 0.0),
                    LabelSide::West => anchor_pos - vec2(GRID_SIZE, 0.0),
                };
                builder.add_h_channel(anchor_pos, COST_ZERO);
            }
        }
        let mut router = builder.build();
        // First all routes that haven't changed
        let mut routes = std::mem::take(&mut self.auto_routes);
        for (id, route) in routes.iter_mut() {
            let anchor_start = snap_to_grid(self.anchor(route.start));
            let anchor_end = snap_to_grid(self.anchor(route.finish));
            if route.start_pos == self.anchor(route.start)
                && route.end_pos == self.anchor(route.finish)
                && !router.is_route_blocked(&route.edges)
                && !self.ripup_set.contains(id)
            {
                router.add_existing_route(&route.edges, WIRE_COST);
            } else {
                let mut waypoints: Vec<Waypoint> = vec![];
                for wp in &route.waypoints {
                    if wp.is_locked() {
                        waypoints.push(wp.clone());
                    } else if router.is_accessible(wp.pos) {
                        if let Some(last_pos) = waypoints.last().map(|wp| wp.pos) {
                            if last_pos != wp.pos {
                                waypoints.push(wp.clone());
                            }
                        } else {
                            waypoints.push(wp.clone());
                        }
                    }
                }
                let path = router.waypoint_path(anchor_start, &waypoints, anchor_end);
                *route = AutoRoute::build(route.start, route.finish, &path, &waypoints);
                route.update_waypoints();
                route.start_pos = anchor_start;
                route.end_pos = anchor_end;
                router.add_existing_route(&route.edges, WIRE_COST);
            }
        }
        self.auto_routes = routes;
        if let State::InProgressAutoRoute(inner) = &self.state {
            eprintln!("Auto-routing from {:?} to {:?}", inner.start, inner.head);
            let start_pos = self.anchor(inner.start);
            let head_pos = snap_to_grid(inner.head);
            self.auto_route = router.waypoint_path(start_pos, &inner.waypoints, head_pos);
        }
        if let State::ProposedAutoRoute(inner) = &self.state {
            let start_pos = snap_to_grid(self.anchor(inner.start));
            let end = snap_to_grid(self.anchor(inner.finish));
            self.auto_route = router.waypoint_path(start_pos, &inner.waypoints, end);
        }
        self.marks = router.debug_marks();
    }
    pub fn demo() -> Self {
        demo_drawing()
    }
}

pub fn demo_drawing() -> Drawing {
    let mut drawing = Drawing::default();
    let origin_1 = pos2(300.0, 300.0);
    let size = vec2(200.0, 200.0);
    let box1 = drawing.add_rect_box(origin_1, origin_1 + size);
    box1.add_label(
        "i.1.write_logic".to_string(),
        LabelSide::West,
        GRID_SIZE * 1.0,
    );
    box1.add_label(
        "i.0.write_logic".to_string(),
        LabelSide::West,
        GRID_SIZE * 2.0,
    );
    let origin_2 = pos2(0.0, 0.0);
    let box2 = drawing.add_rect_box(origin_2, origin_2 + size);
    box2.add_label(
        "o.1.read_logic".to_string(),
        LabelSide::East,
        GRID_SIZE * 1.0,
    );
    box2.add_label(
        "o.0.read_logic".to_string(),
        LabelSide::East,
        GRID_SIZE * 2.0,
    );
    drawing
}
