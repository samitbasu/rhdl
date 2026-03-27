use egui::{Pos2, Rect, pos2};

pub const GRID_SIZE: f32 = 10.0;
pub const SHIM: f32 = 7.0;
pub const MOVE_HOVER_DISTANCE: f32 = GRID_SIZE * 0.8;
pub const PORT_RADIUS: f32 = 3.0;
pub const TITLE_TEXT_SIZE: f32 = 10.0;
pub const PORT_TEXT_SIZE: f32 = 8.0;
pub const CONTROL_HANDLE_SIZE: f32 = 3.0;
pub const GRIP_SIZE: f32 = 6.0;

pub fn grid(pos: Pos2) -> Pos2 {
    Pos2::new(round_to_grid(pos.x), round_to_grid(pos.y))
}

pub fn round_to_grid(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

pub fn snap(rect: Rect) -> Rect {
    Rect::from_min_max(
        grid(pos2(rect.min.x, rect.min.y)),
        grid(pos2(rect.max.x, rect.max.y)),
    )
}

pub fn grid_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        grid(pos2(rect.min.x, rect.min.y)),
        grid(pos2(rect.max.x, rect.max.y)),
    )
}
