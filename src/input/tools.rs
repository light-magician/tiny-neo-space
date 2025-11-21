use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::core::*;
use crate::rendering::CanvasRenderer;

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
    mouse_world: &Vec2,
    is_erasing: bool,
    canvas_renderer: &mut CanvasRenderer,
) {
    // Only respond to mouse button down
    if is_mouse_button_down(MouseButton::Left) {
        // Calculate which cell the mouse is over (floor to get cell indices)
        let cell_coords = (mouse_world.x.floor() as i32, mouse_world.y.floor() as i32);

        // Set cell state: Some(Cell) for paint, None for erase
        let new_cell = if is_erasing {
            None
        } else {
            Some(Cell::with_color(state.current_color))
        };

        set_cell(state, cell_coords, new_cell, canvas_renderer);
    }
}
