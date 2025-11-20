use macroquad::prelude::*;

use super::mode_selector::Mode;
use super::cell::grid_position_to_cell_coords;

const GRID_SIZE: f32 = 10.0;

pub fn draw_cursor_based_on_mode(mode: &Mode, mouse_pos: Vec2) {
    // Highlight the cell that will be painted/erased
    let cell_coords = grid_position_to_cell_coords(&mouse_pos, GRID_SIZE);
    let cell_x = cell_coords.0 as f32 * GRID_SIZE;
    let cell_y = cell_coords.1 as f32 * GRID_SIZE;

    match mode {
        Mode::Paint => {
            // Draw highlight box around the cell
            draw_rectangle_lines(cell_x, cell_y, GRID_SIZE, GRID_SIZE, 2.0, Color::from_rgba(0, 0, 0, 150));
            // Small cursor dot
            draw_circle(mouse_pos.x, mouse_pos.y, 3.0, BLACK);
        }
        Mode::Erase => {
            // Draw red highlight for eraser
            draw_rectangle_lines(cell_x, cell_y, GRID_SIZE, GRID_SIZE, 2.0, Color::from_rgba(255, 100, 100, 200));
            // Eraser cursor
            draw_rectangle(mouse_pos.x - 5.0, mouse_pos.y - 5.0, 10.0, 10.0, Color::from_rgba(255, 100, 100, 150));
        }
    }
}