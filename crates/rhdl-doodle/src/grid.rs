use egui::{Pos2, Rect, pos2};

pub const GRID_SIZE: f32 = 15.0;
pub const SHIM: f32 = GRID_SIZE * 0.7;
pub const MOVE_HOVER_DISTANCE: f32 = GRID_SIZE * 0.8;
pub const PORT_RADIUS: f32 = GRID_SIZE * 0.3;
pub const LINE_RADIUS: f32 = GRID_SIZE * 0.5;
pub const TITLE_TEXT_SIZE: f32 = GRID_SIZE;
pub const PORT_TEXT_SIZE: f32 = GRID_SIZE * 0.8;
pub const ROUTE_TEXT_SIZE: f32 = GRID_SIZE * 0.6;
pub const CONTROL_HANDLE_SIZE: f32 = GRID_SIZE * 0.3;
pub const GRIP_SIZE: f32 = GRID_SIZE * 0.6;
pub const MIN_TEXT_EDGE_LENGTH: f32 = GRID_SIZE * 4.0;

pub fn snap_to_grid(pos: Pos2) -> Pos2 {
    Pos2::new(round_to_grid(pos.x), round_to_grid(pos.y))
}

pub fn round_to_grid(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

pub fn snap(rect: Rect) -> Rect {
    Rect::from_min_max(
        snap_to_grid(pos2(rect.min.x, rect.min.y)),
        snap_to_grid(pos2(rect.max.x, rect.max.y)),
    )
}

pub fn grid_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        snap_to_grid(pos2(rect.min.x, rect.min.y)),
        snap_to_grid(pos2(rect.max.x, rect.max.y)),
    )
}
