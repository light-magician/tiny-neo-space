// Core constants for the pixel canvas
pub const GRID_SIZE: f32 = 10.0;
pub const GRID_THICKNESS: f32 = 1.0;

// Grid color helper function (since Color::from_rgba is not const)
pub fn grid_color() -> macroquad::prelude::Color {
    macroquad::prelude::Color::from_rgba(210, 225, 255, 255)
}
