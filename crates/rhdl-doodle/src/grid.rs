use egui::{Pos2, Rect, pos2};

pub const GRID_SIZE: f32 = 10.0;
pub const SHIM: f32 = 4.0;

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

pub fn round_to_even_grid(value: f32) -> f32 {
    let n = (0.5 * (value / GRID_SIZE - 1.0)).round() as i32;
    (n as f32 * 2.0) * GRID_SIZE
}

pub fn grid_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        grid(pos2(rect.min.x, rect.min.y)),
        grid(pos2(rect.max.x, rect.max.y)),
    )
}
