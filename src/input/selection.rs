use macroquad::prelude::*;
use std::collections::HashSet;
use crate::state::ApplicationState;
use crate::rendering::CanvasRenderer;
use crate::core::selection::{SelectionKind, Selection, SelectionRect, compute_bounding_rect, LiftedCell};

pub fn handle_select_tool(state: &mut ApplicationState, canvas: &mut CanvasRenderer) {
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);
    let cell_coords = (world_mouse_pos.x.floor() as i32, world_mouse_pos.y.floor() as i32);
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

    // Mouse pressed: start drag or move
    if is_mouse_button_pressed(MouseButton::Left) {
        if state.selection.contains_point(cell_coords.0, cell_coords.1) {
            // Click inside selection → start move with lift
            start_move_with_lift(state, canvas, (world_mouse_pos.x, world_mouse_pos.y));
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
            drop_lifted(state, canvas);
        } else if state.selection.active_drag {
            finalize_selection_drag_tight(state, shift);
        } else if shift {
            // Shift-click adds single cell if filled
            if let Some(c) = state.cells.get(&cell_coords) {
                if c.is_filled {
                    let mut set: HashSet<(i32, i32)> = match &state.selection.current {
                        Some(sel) => match &sel.kind {
                            SelectionKind::Cells(s) => s.clone(),
                        },
                        None => HashSet::new(),
                    };
                    set.insert(cell_coords);
                    if let Some(rect) = compute_bounding_rect(&set) {
                        state.selection.current = Some(Selection {
                            rect,
                            kind: SelectionKind::Cells(set),
                            preview: None,
                        });
                    }
                }
            }
        }
    }
}

/// Finalize drag with tight bounding box (only filled cells) and optional Shift-additive selection
fn finalize_selection_drag_tight(state: &mut ApplicationState, additive: bool) {
    state.selection.active_drag = false;

    if let (Some(start), Some(end)) = (state.selection.drag_start, state.selection.drag_end) {
        let rect = SelectionRect::from_points(start, end);

        // Pick only filled cells within drag rect
        let mut picked: HashSet<(i32, i32)> = HashSet::new();
        for (&coord, cell) in state.cells.iter() {
            if cell.is_filled && rect.contains(coord.0, coord.1) {
                picked.insert(coord);
            }
        }

        if picked.is_empty() {
            if !additive {
                state.selection.current = None;
            }
            return;
        }

        // Union for additive (Shift), replace otherwise
        let final_set = if additive {
            if let Some(sel) = &state.selection.current {
                if let SelectionKind::Cells(existing) = &sel.kind {
                    existing.union(&picked).cloned().collect()
                } else {
                    picked
                }
            } else {
                picked
            }
        } else {
            picked
        };

        // Compute tight bounding rect
        if let Some(rect) = compute_bounding_rect(&final_set) {
            let mut selection = Selection {
                rect,
                kind: SelectionKind::Cells(final_set),
                preview: None,
            };

            // Build preview for the new selection
            if let SelectionKind::Cells(ref cell_set) = selection.kind {
                selection.preview = crate::rendering::selection::build_selection_preview(
                    &state.cells,
                    &selection.rect,
                    cell_set
                );
            }

            state.selection.current = Some(selection);
        }
    }
}

/// Start move with lift: removes selected cells from canvas and stores in lifted_cells
fn start_move_with_lift(state: &mut ApplicationState, canvas: &mut CanvasRenderer, mouse_world: (f32, f32)) {
    if state.selection.is_moving {
        return;
    }

    if let Some(sel) = &mut state.selection.current {
        // Ensure preview exists before moving
        if sel.preview.is_none() {
            if let SelectionKind::Cells(ref cell_set) = sel.kind {
                sel.preview = crate::rendering::selection::build_selection_preview(
                    &state.cells,
                    &sel.rect,
                    cell_set
                );
            }
        }

        if let SelectionKind::Cells(set) = &sel.kind {
            state.selection.lifted_cells.clear();
            for &(x, y) in set.iter() {
                if let Some(cell) = state.cells.remove(&(x, y)) {
                    state.selection.lifted_cells.push(LiftedCell {
                        coord: (x, y),
                        cell,
                    });
                    canvas.mark_dirty((x, y));
                }
            }
        }

        state.selection.is_lifted = true;
        state.selection.is_moving = true;
        state.selection.move_offset_x = 0.0;
        state.selection.move_offset_y = 0.0;
        state.selection.last_move_mouse = Some(mouse_world);
    }
}

/// Drop lifted cells: reinserts at new snapped position and updates selection
fn drop_lifted(state: &mut ApplicationState, canvas: &mut CanvasRenderer) -> Option<(i32, i32)> {
    if !state.selection.is_lifted {
        return None;
    }

    let dx = state.selection.move_offset_x.round() as i32;
    let dy = state.selection.move_offset_y.round() as i32;

    let mut new_set: HashSet<(i32, i32)> = HashSet::new();
    for lifted in state.selection.lifted_cells.drain(..) {
        let dest = (lifted.coord.0 + dx, lifted.coord.1 + dy);
        state.cells.insert(dest, lifted.cell);
        canvas.mark_dirty(dest);
        new_set.insert(dest);
    }

    if let Some(sel) = &mut state.selection.current {
        sel.kind = SelectionKind::Cells(new_set.clone());
        if let Some(rect) = compute_bounding_rect(&new_set) {
            sel.rect = rect;
        }
        sel.preview = None; // Lazily rebuild on next move if needed
    }

    state.selection.is_lifted = false;
    state.selection.is_moving = false;
    state.selection.move_offset_x = 0.0;
    state.selection.move_offset_y = 0.0;

    Some((dx, dy))
}

/// Delete selected cells (called from dispatcher with canvas access)
pub fn delete_selection(state: &mut ApplicationState, canvas: &mut CanvasRenderer) {
    if let Some(sel) = &state.selection.current {
        if let SelectionKind::Cells(coords) = &sel.kind {
            for &coord in coords {
                if state.cells.remove(&coord).is_some() {
                    canvas.mark_dirty(coord);
                }
            }
        }
        state.selection.current = None;
        state.selection.is_moving = false;
    }
}
