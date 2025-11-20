use macroquad::prelude::*;
use super::state::ApplicationState;
use super::cell::{Cell, grid_position_to_cell_coords};
use super::canvas_renderer::CanvasRenderer;

const GRID_SIZE: f32 = 10.0;

/// Set a cell to a specific color or clear it (None = erase)
/// This is the unified abstraction for both painting and erasing
fn set_cell(
    state: &mut ApplicationState,
    cell_coords: (i32, i32),
    new_cell: Option<Cell>,
    canvas_renderer: &mut CanvasRenderer,
) {
    match new_cell {
        Some(cell) => {
            // Painting: check if we're actually changing the cell
            let needs_update = match state.cells.get(&cell_coords) {
                Some(existing_cell) => existing_cell.color != cell.color,
                None => true,
            };

            if needs_update {
                state.cells.insert(cell_coords, cell);
                canvas_renderer.mark_dirty(cell_coords);
            }
        }
        None => {
            // Erasing: remove cell if it exists
            if state.cells.remove(&cell_coords).is_some() {
                canvas_renderer.mark_dirty(cell_coords);
            }
        }
    }
}

/// Handle mouse input for painting or erasing
pub fn perform_drawing(
    state: &mut ApplicationState,
    mouse_pos: &Vec2,
    is_erasing: bool,
    canvas_renderer: &mut CanvasRenderer,
) {
    // Check if mouse is within screen bounds
    if mouse_pos.x < 0.0 || mouse_pos.y < 0.0 ||
       mouse_pos.x >= screen_width() || mouse_pos.y >= screen_height() {
        return;
    }

    // Only respond to mouse button down
    if is_mouse_button_down(MouseButton::Left) {
        // Calculate which cell the mouse is over
        let cell_coords = grid_position_to_cell_coords(mouse_pos, GRID_SIZE);

        // Set cell state: Some(Cell) for paint, None for erase
        let new_cell = if is_erasing {
            None
        } else {
            Some(Cell::with_color(state.current_color))
        };

        set_cell(state, cell_coords, new_cell, canvas_renderer);
    }
}