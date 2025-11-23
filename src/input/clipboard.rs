use std::collections::HashMap;
use macroquad::prelude::*;
use crate::state::{ApplicationState, CellChange};
use crate::rendering::CanvasRenderer;
use crate::core::selection::{SelectionKind, Selection, compute_bounding_rect};

/// Copy the current selection to the clipboard
/// Stores cells with relative coordinates (offset from rect.min_x, rect.min_y)
pub fn copy_selection(state: &mut ApplicationState) {
    if let Some(sel) = &state.selection.current {
        if let SelectionKind::Cells(set) = &sel.kind {
            let rect = sel.rect;
            let mut cells = HashMap::new();

            // Copy cells with relative coordinates
            for &(x, y) in set.iter() {
                if let Some(cell) = state.cells.get(&(x, y)).cloned() {
                    let rel_x = x - rect.min_x;
                    let rel_y = y - rect.min_y;
                    cells.insert((rel_x, rel_y), cell);
                }
            }

            // Update clipboard
            state.clipboard.width = rect.max_x - rect.min_x + 1;
            state.clipboard.height = rect.max_y - rect.min_y + 1;
            state.clipboard.cells = cells;
            state.clipboard.has_data = true;
        }
    }
}

/// Cut the current selection (copy then delete)
/// For now, manually handles deletion - will integrate with history in Phase 3
pub fn cut_selection(state: &mut ApplicationState, canvas: &mut CanvasRenderer) {
    // First copy to clipboard
    copy_selection(state);

    // Then delete the cells
    if let Some(sel) = &state.selection.current {
        if let SelectionKind::Cells(set) = &sel.kind {
            // Delete each cell from the grid
            for &(x, y) in set.iter() {
                state.cells.remove(&(x, y));
                canvas.mark_dirty((x, y));
            }

            // Clear the selection
            state.selection.current = None;
        }
    }
}

/// Paste clipboard contents at the cursor position
/// Creates a new selection at the pasted location
/// For now, manually handles insertion - will integrate with history in Phase 3
pub fn paste_clipboard_at_cursor(state: &mut ApplicationState, canvas: &mut CanvasRenderer) {
    if !state.clipboard.has_data {
        return;
    }

    // Get cursor position in world coordinates
    let mouse = Vec2::from(mouse_position());
    let world = state.camera.screen_to_cell(mouse);
    let anchor = (world.x.floor() as i32, world.y.floor() as i32);

    // Place clipboard cells offset from anchor
    let mut placed_coords = Vec::new();

    for (rel_coord, cell) in state.clipboard.cells.iter() {
        let dest_x = anchor.0 + rel_coord.0;
        let dest_y = anchor.1 + rel_coord.1;
        let dest = (dest_x, dest_y);

        // Insert cell into grid
        state.cells.insert(dest, *cell);
        canvas.mark_dirty(dest);
        placed_coords.push(dest);
    }

    // Create selection at pasted location
    use std::collections::HashSet;
    let set: HashSet<(i32, i32)> = placed_coords.into_iter().collect();

    if let Some(rect) = compute_bounding_rect(&set) {
        state.selection.current = Some(Selection {
            rect,
            kind: SelectionKind::Cells(set),
            preview: None,
        });
    }
}
