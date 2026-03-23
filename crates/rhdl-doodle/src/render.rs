use egui::{Color32, Rect, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::{
    drawing::ResizeMode,
    grid::{GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_RADIUS, SHIM, grid_rect},
    label::{LabelId, LabelSide},
    rectbox::{RectBox, resize_rect},
};

pub fn draw_resizing_rect(rect: &RectBox, ui: &mut Ui, mode: ResizeMode, delta: Vec2) {
    let resized_rect = resize_rect(&rect.inner, mode, delta);
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
    ui.painter().text(
        resized_rect.center_top() + vec2(0.0, SHIM),
        egui::Align2::CENTER_TOP,
        &rect.name,
        egui::FontId::monospace(10.0),
        Color32::BLACK,
    );
}

pub fn draw_moving_rect(rect: &RectBox, ui: &mut Ui, delta: Vec2) {
    let shifted_rect = rect.inner.translate(delta);
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
        &rect.name,
        egui::FontId::monospace(10.0),
        Color32::BLACK,
    );
    render_labels_with_box(rect, shifted_rect, ui);
}

pub fn render_labels_with_box(rect: &RectBox, bbox: Rect, ui: &mut Ui) {
    for label in &rect.labels {
        let y_coord = bbox.top() + GRID_SIZE + label.offset;
        let (align, text_pos, shift, stem) = match label.side {
            LabelSide::East => (
                egui::Align2::RIGHT_CENTER,
                pos2(bbox.right(), y_coord),
                vec2(SHIM, 0.0),
                vec2(GRID_SIZE, 0.0),
            ),
            LabelSide::West => (
                egui::Align2::LEFT_CENTER,
                pos2(bbox.left(), y_coord),
                vec2(-SHIM, 0.0),
                vec2(-GRID_SIZE, 0.0),
            ),
        };
        ui.painter()
            .line_segment([text_pos, text_pos + stem], (0.5, Color32::DARK_RED));
        ui.painter().text(
            text_pos - shift,
            align,
            &label.text,
            egui::FontId::monospace(8.0),
            Color32::BLACK,
        );
    }
}

pub fn render_still(rect: &RectBox, ui: &mut Ui) {
    let egui_box = rect.inner;
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
        &rect.name,
        egui::FontId::monospace(10.0),
        Color32::BLACK,
    );
    render_labels_with_box(rect, egui_box, ui);
}

pub fn draw_dragged_label(rrect: &RectBox, label: LabelId, delta_pos: Vec2, ui: &mut Ui) {
    let bbox = rrect.inner;
    let label = rrect.labels.iter().find(|l| l.id == label).unwrap();
    let y_coord = bbox.top() + GRID_SIZE + label.offset + delta_pos.y;
    let (text_pos, stem) = match label.side {
        LabelSide::East => (pos2(bbox.right(), y_coord), vec2(GRID_SIZE, 0.0)),
        LabelSide::West => (pos2(bbox.left(), y_coord), vec2(-GRID_SIZE, 0.0)),
    };
    ui.painter()
        .line_segment([text_pos, text_pos + stem], (0.5, Color32::DARK_RED));
    ui.painter().circle(
        text_pos + stem,
        PORT_RADIUS,
        Color32::DARK_GRAY,
        (0.5, Color32::DARK_RED),
    );
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        &label.text,
        egui::FontId::monospace(8.0),
        Color32::BLACK,
    );
}

pub fn draw_control_frame(rrect: &RectBox, ui: &mut Ui) -> Option<()> {
    let bbox = rrect.inner;
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
    [
        rrect.control_pin_location_east(),
        rrect.control_pin_location_west(),
    ]
    .iter()
    .flatten()
    .for_each(|&pin_pos| {
        ui.painter()
            .circle(pin_pos, PORT_RADIUS, Color32::WHITE, (0.5, Color32::BLACK));
        ui.painter().line_segment(
            [
                pin_pos + vec2(-PORT_RADIUS / 2.0, 0.0),
                pin_pos + vec2(PORT_RADIUS / 2.0, 0.0),
            ],
            (1.0, Color32::BLACK),
        );
        ui.painter().line_segment(
            [
                pin_pos + vec2(0.0, -PORT_RADIUS / 2.0),
                pin_pos + vec2(0.0, PORT_RADIUS / 2.0),
            ],
            (1.0, Color32::BLACK),
        );
    });
    for label in &rrect.labels {
        let y_coord = bbox.top() + GRID_SIZE + label.offset;
        let (text_pos, stem) = match label.side {
            LabelSide::East => (pos2(bbox.right(), y_coord), vec2(GRID_SIZE, 0.0)),
            LabelSide::West => (pos2(bbox.left(), y_coord), vec2(-GRID_SIZE, 0.0)),
        };
        ui.painter()
            .line_segment([text_pos, text_pos + stem], (0.5, Color32::DARK_RED));
        ui.painter().circle(
            text_pos + stem,
            PORT_RADIUS,
            Color32::DARK_GRAY,
            (0.5, Color32::DARK_RED),
        );
    }
    Some(())
}
