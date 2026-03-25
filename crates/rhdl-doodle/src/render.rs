use egui::{Color32, Rect, Stroke, StrokeKind, TextEdit, Ui, Vec2, pos2, vec2};

use crate::{
    drawing::{ResizeMode, State},
    grid::{
        CONTROL_HANDLE_SIZE, GRID_SIZE, GRIP_SIZE, PORT_RADIUS, PORT_TEXT_SIZE, SHIM,
        TITLE_TEXT_SIZE, grid_rect,
    },
    label::{Label, LabelId, LabelSide},
    rectbox::{RectBox, resize_rect},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GripState {
    Hidden,
    Drawn,
}

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
        egui::FontId::monospace(TITLE_TEXT_SIZE),
        Color32::BLACK,
    );
    render_labels_with_box(rect.labels.iter(), resized_rect, GripState::Hidden, ui);
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
        egui::FontId::monospace(TITLE_TEXT_SIZE),
        Color32::BLACK,
    );
    render_labels_with_box(rect.labels.iter(), shifted_rect, GripState::Hidden, ui);
}

fn render_labels_with_box<'a>(
    iter: impl Iterator<Item = &'a Label>,
    bbox: Rect,
    grip_state: GripState,
    ui: &mut Ui,
) {
    for label in iter {
        draw_label_and_pin(bbox, label, grip_state, 0.0, ui);
    }
}

fn render_frame(rect: &RectBox, ui: &mut Ui) {
    let egui_box = rect.inner;
    ui.painter().rect(
        egui_box,
        3.0,
        Color32::LIGHT_GRAY,
        (1.0, Color32::BLUE),
        StrokeKind::Middle,
    );
    ui.painter().text(
        egui_box.center_top() + vec2(0.0, SHIM),
        egui::Align2::CENTER_TOP,
        &rect.name,
        egui::FontId::monospace(TITLE_TEXT_SIZE),
        Color32::BLACK,
    );
}

fn render_normal(rect: &RectBox, grip_state: GripState, ui: &mut Ui) {
    render_frame(rect, ui);
    render_labels_with_box(rect.labels.iter(), rect.inner, grip_state, ui);
}

// Draw
//          v anchor point
// port [s] |----
fn draw_label_and_pin(bbox: Rect, label: &Label, grip_state: GripState, offset: f32, ui: &mut Ui) {
    let y_coord = bbox.top() + GRID_SIZE + label.offset + offset;
    let (text_pos, stem, align) = match label.side {
        LabelSide::East => (
            pos2(bbox.right() - GRID_SIZE, y_coord),
            vec2(GRID_SIZE, 0.0),
            egui::Align2::RIGHT_CENTER,
        ),
        LabelSide::West => (
            pos2(bbox.left() + GRID_SIZE, y_coord),
            vec2(-GRID_SIZE, 0.0),
            egui::Align2::LEFT_CENTER,
        ),
    };
    ui.painter().line_segment(
        [text_pos + stem, text_pos + 2.0 * stem],
        (0.5, Color32::DARK_RED),
    );
    ui.painter().text(
        text_pos,
        align,
        &label.text,
        egui::FontId::monospace(PORT_TEXT_SIZE),
        Color32::BLACK,
    );
    let hamburger_rect = get_hamburger_rect(bbox.translate(vec2(0.0, offset)), label);
    // Draw a hamburger grip.
    match grip_state {
        GripState::Hidden => {}
        GripState::Drawn => {
            let bun_height = hamburger_rect.height() / 5.0;
            for i in 0..3 {
                let bun_rect = Rect::from_center_size(
                    pos2(
                        hamburger_rect.center().x,
                        hamburger_rect.top() + bun_height / 2.0 + 2.0 * i as f32 * bun_height,
                    ),
                    vec2(hamburger_rect.width(), bun_height),
                );
                ui.painter().rect(
                    bun_rect,
                    bun_height / 2.0,
                    Color32::DARK_GRAY.gamma_multiply(0.3),
                    Stroke::NONE,
                    StrokeKind::Middle,
                );
            }
        }
    }
}

pub fn estimate_bbox_for_label(bbox: Rect, label: &Label) -> Rect {
    let y_coord = bbox.top() + GRID_SIZE + label.offset;
    let text_width = label.text.len() as f32 * PORT_TEXT_SIZE * 0.6;
    match label.side {
        LabelSide::East => Rect::from_min_max(
            pos2(
                bbox.right() - GRID_SIZE - text_width,
                y_coord - PORT_TEXT_SIZE / 2.0,
            ),
            pos2(bbox.right() - GRID_SIZE, y_coord + PORT_TEXT_SIZE / 2.0),
        ),
        LabelSide::West => Rect::from_min_max(
            pos2(bbox.left() + GRID_SIZE, y_coord - PORT_TEXT_SIZE / 2.0),
            pos2(
                bbox.left() + GRID_SIZE + text_width,
                y_coord + PORT_TEXT_SIZE / 2.0,
            ),
        ),
    }
}

pub fn get_hamburger_rect(bbox: Rect, label: &Label) -> Rect {
    let y_coord = bbox.top() + GRID_SIZE + label.offset;
    let (text_pos, stem) = match label.side {
        LabelSide::East => (
            pos2(bbox.right() - GRID_SIZE, y_coord),
            vec2(GRID_SIZE, 0.0),
        ),
        LabelSide::West => (
            pos2(bbox.left() + GRID_SIZE, y_coord),
            vec2(-GRID_SIZE, 0.0),
        ),
    };
    Rect::from_center_size(text_pos + stem / 2.0, vec2(GRIP_SIZE, GRIP_SIZE))
}

pub fn draw_dragged_label(rrect: &RectBox, label: LabelId, delta_pos: Vec2, ui: &mut Ui) {
    let Some(label) = rrect.label(label) else {
        return;
    };
    draw_label_and_pin(rrect.inner, label, GripState::Drawn, delta_pos.y, ui);
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
            Rect::from_center_size(pos, vec2(CONTROL_HANDLE_SIZE, CONTROL_HANDLE_SIZE)),
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
    Some(())
}

fn render_selected(target: &RectBox, ui: &mut Ui) {
    render_normal(target, GripState::Drawn, ui);
    draw_control_frame(target, ui);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FocusResult {
    #[default]
    KeptFocus,
    LostFocus,
}

pub fn render_rect_box(target: &mut RectBox, state: &State, ui: &mut Ui) -> FocusResult {
    match state {
        State::MovingRect { rect, delta_pos } if target.id() == *rect => {
            draw_moving_rect(target, ui, *delta_pos);
        }
        State::Selected { rect }
        | State::PotentialResize { rect, .. }
        | State::PortLabelHovered { rect, .. }
        | State::PortLabelGripHovered { rect, .. }
            if target.id() == *rect =>
        {
            render_selected(target, ui);
        }
        State::ResizingRect {
            rect,
            mode,
            delta_pos,
        } if target.id() == *rect => {
            draw_resizing_rect(target, ui, *mode, *delta_pos);
        }
        State::PortDragged {
            rect,
            label,
            delta_pos,
        } if target.id() == *rect => {
            render_frame(target, ui);
            render_labels_with_box(
                target.labels.iter().filter(|l| l.id != *label),
                target.inner,
                GripState::Drawn,
                ui,
            );
            draw_control_frame(target, ui);
            ui.painter().line_segment(
                [target.inner.center_top(), target.inner.center_bottom()],
                (2.0, Color32::DARK_GRAY.gamma_multiply(0.3)),
            );
            draw_dragged_label(target, *label, *delta_pos, ui);
        }
        State::EditingName { rect } if target.id() == *rect => {
            render_selected(target, ui);
            let rect_name_width = target.name.len() as f32 * 10.0 + 10.0;
            let editor_position = target.inner.center_top() + vec2(-rect_name_width / 2.0, SHIM);
            let editor_rect = Rect::from_min_size(editor_position, vec2(rect_name_width, 20.0));
            let response = ui.place(
                editor_rect,
                TextEdit::singleline(&mut target.name)
                    .font(egui::FontId::monospace(TITLE_TEXT_SIZE))
                    .desired_width(f32::INFINITY),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                return FocusResult::LostFocus;
            } else {
                response.request_focus();
            }
        }
        State::EditingLabelText { rect, label } if target.id() == *rect => {
            render_selected(target, ui);
            let target_inner = target.inner;
            let Some(label_ref) = target.label_mut(*label) else {
                return FocusResult::KeptFocus;
            };
            let editor_width =
                ((label_ref.text.len() as f32 * PORT_TEXT_SIZE * 0.6 + 10.0) / 2.0).max(20.0);
            let editor_position =
                get_hamburger_rect(target_inner, label_ref).expand2(vec2(editor_width, 0.0));
            let response = ui.place(
                editor_position,
                TextEdit::singleline(&mut label_ref.text)
                    .font(egui::FontId::monospace(PORT_TEXT_SIZE))
                    .desired_width(f32::INFINITY),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                return FocusResult::LostFocus;
            } else {
                response.request_focus();
            }
        }
        _ => render_normal(target, GripState::Hidden, ui),
    }
    FocusResult::KeptFocus
}
