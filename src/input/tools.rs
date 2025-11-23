use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::core::*;
use crate::rendering::CanvasRenderer;

/// Bresenham line algorithm - returns all grid cells between two points
fn bresenham(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        cells.push((x0, y0));

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }

    cells
}

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

/// Handle mouse input for painting or erasing with stroke interpolation
pub fn perform_drawing(
    state: &mut ApplicationState,
    mouse_world: &Vec2,
    is_erasing: bool,
    canvas_renderer: &mut CanvasRenderer,
) {
    let cell_coords = (mouse_world.x.floor() as i32, mouse_world.y.floor() as i32);

    // Mouse just pressed - start new stroke
    if is_mouse_button_pressed(MouseButton::Left) {
        state.last_painted_cell = Some(cell_coords);

        let new_cell = if is_erasing {
            None
        } else {
            Some(Cell::with_color(state.current_color))
        };
        set_cell(state, cell_coords, new_cell, canvas_renderer);
    }
    // Mouse held - interpolate stroke
    else if is_mouse_button_down(MouseButton::Left) {
        if let Some(last_cell) = state.last_painted_cell {
            // Interpolate all cells between last and current
            let cells_to_paint = bresenham(last_cell, cell_coords);

            for coords in cells_to_paint {
                let new_cell = if is_erasing {
                    None
                } else {
                    Some(Cell::with_color(state.current_color))
                };
                set_cell(state, coords, new_cell, canvas_renderer);
            }

            state.last_painted_cell = Some(cell_coords);
        }
    }
    // Mouse released - end stroke
    else if is_mouse_button_released(MouseButton::Left) {
        state.last_painted_cell = None;
    }
}
