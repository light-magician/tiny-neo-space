use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::core::selection::SelectionKind;

pub fn handle_select_tool(state: &mut ApplicationState) {
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);
    let cell_coords = (world_mouse_pos.x.floor() as i32, world_mouse_pos.y.floor() as i32);

    // Mouse pressed: start drag or move
    if is_mouse_button_pressed(MouseButton::Left) {
        if state.selection.contains_point(cell_coords.0, cell_coords.1) {
            // Click inside selection → start move
            state.selection.start_move((world_mouse_pos.x, world_mouse_pos.y));
        } else {
            // Click outside → start new selection drag
            state.selection.start_drag(cell_coords);
        }
    }

    // During drag: update end point
    if state.selection.active_drag && is_mouse_button_down(MouseButton::Left) {
        state.selection.update_drag_end(cell_coords);
    }

    // During move: accumulate delta
    if state.selection.is_moving && is_mouse_button_down(MouseButton::Left) {
        if let Some((prev_x, prev_y)) = state.selection.last_move_mouse {
            let delta_x = world_mouse_pos.x - prev_x;
            let delta_y = world_mouse_pos.y - prev_y;
            state.selection.update_move(delta_x, delta_y);
        }
        state.selection.last_move_mouse = Some((world_mouse_pos.x, world_mouse_pos.y));
    }

    // Mouse released: finalize
    if is_mouse_button_released(MouseButton::Left) {
        if state.selection.is_moving {
            if let Some((offset_x, offset_y)) = state.selection.finalize_move() {
                apply_selection_move(state, offset_x, offset_y);
            }
        } else if state.selection.active_drag {
            state.selection.finalize_drag(&state.cells);
        }
    }
}

fn apply_selection_move(state: &mut ApplicationState, offset_x: i32, offset_y: i32) {
    if let Some(sel) = &state.selection.current {
        if let SelectionKind::Cells(coords) = &sel.kind {
            // Collect old cell data
            let mut cell_data = Vec::new();
            for &(x, y) in coords.iter() {
                let old_coord = (x - offset_x, y - offset_y);
                if let Some(cell) = state.cells.remove(&old_coord) {
                    cell_data.push(((x, y), cell));
                }
            }

            // Place at new coordinates
            for (new_coord, cell) in cell_data {
                state.cells.insert(new_coord, cell);
            }
        }
    }
}

/// Delete selected cells
pub fn delete_selection(state: &mut ApplicationState) {
    if let Some(sel) = &state.selection.current {
        if let SelectionKind::Cells(coords) = &sel.kind {
            for &coord in coords {
                state.cells.remove(&coord);
            }
        }
        state.selection.current = None;
    }
}
