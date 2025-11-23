use macroquad::prelude::*;
use crate::state::{Mode, ApplicationState};
use crate::rendering::CanvasRenderer;
use super::tools::perform_drawing;
use super::selection::handle_select_tool;

/// Central input dispatcher that handles all user input based on current application state
pub fn handle_input(
    state: &mut ApplicationState,
    canvas_renderer: &mut CanvasRenderer,
) {
    // Clipboard operations (check before mode hotkeys to avoid conflicts)
    if ctrl_or_cmd() && is_key_pressed(KeyCode::C) {
        crate::input::clipboard::copy_selection(state);
    }

    if ctrl_or_cmd() && is_key_pressed(KeyCode::X) {
        crate::input::clipboard::cut_selection(state, canvas_renderer);
    }

    if ctrl_or_cmd() && is_key_pressed(KeyCode::V) {
        crate::input::clipboard::paste_clipboard_at_cursor(state, canvas_renderer);
    }

    if ctrl_or_cmd() && is_key_pressed(KeyCode::Z) {
        undo_last(state, canvas_renderer);
    }

    // Hotkeys for mode switching (check before mode dispatch)
    if is_key_pressed(KeyCode::B) {
        state.mode = Mode::Paint;
    }
    if is_key_pressed(KeyCode::E) {
        state.mode = Mode::Erase;
    }
    if !ctrl_or_cmd() && is_key_pressed(KeyCode::V) {
        state.mode = Mode::Select;
    }
    if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::Space) {
        state.mode = Mode::Pan;
    }

    // Delete selection hotkey
    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
        crate::input::selection::delete_selection(state, canvas_renderer);
    }

    // Existing mode-based dispatch
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);

    match state.mode {
        Mode::Paint => perform_drawing(state, &world_mouse_pos, false, canvas_renderer),
        Mode::Erase => perform_drawing(state, &world_mouse_pos, true, canvas_renderer),
        Mode::Pan => handle_pan_tool(state, screen_mouse_pos),
        Mode::Select => handle_select_tool(state, canvas_renderer),
    }
}

/// Apply changes to cells and record them in history for undo
pub fn apply_changes_and_record(
    state: &mut ApplicationState,
    canvas: &mut CanvasRenderer,
    mut changes: Vec<crate::state::CellChange>,
) {
    // Fill in 'before' values if not set
    for ch in changes.iter_mut() {
        if ch.before.is_none() {
            ch.before = state.cells.get(&ch.coord).cloned();
        }
    }

    // Apply changes
    for ch in changes.iter() {
        match ch.after {
            Some(cell) => {
                state.cells.insert(ch.coord, cell);
            }
            None => {
                state.cells.remove(&ch.coord);
            }
        }
        canvas.mark_dirty(ch.coord);
    }

    // Record in history
    state.history.push(crate::state::Command { changes });
}

/// Undo the last command in history
pub fn undo_last(state: &mut ApplicationState, canvas: &mut CanvasRenderer) {
    if let Some(cmd) = state.history.pop() {
        for ch in cmd.changes {
            match ch.before {
                Some(cell) => {
                    state.cells.insert(ch.coord, cell);
                }
                None => {
                    state.cells.remove(&ch.coord);
                }
            }
            canvas.mark_dirty(ch.coord);
        }
    }
}

/// Helper to check if Ctrl (Windows/Linux) or Cmd (Mac) is pressed
fn ctrl_or_cmd() -> bool {
    is_key_down(KeyCode::LeftControl)
        || is_key_down(KeyCode::RightControl)
        || is_key_down(KeyCode::LeftSuper)
        || is_key_down(KeyCode::RightSuper)
}

/// Handle pan tool interaction
fn handle_pan_tool(state: &mut ApplicationState, screen_mouse: Vec2) {
    if is_mouse_button_pressed(MouseButton::Left) {
        state.pan_drag_start_screen = Some(screen_mouse);
        state.pan_drag_start_origin = Some(state.camera.origin);
    }

    if is_mouse_button_down(MouseButton::Left) {
        if let (Some(start_screen), Some(start_origin)) =
            (state.pan_drag_start_screen, state.pan_drag_start_origin)
        {
            let delta_screen = screen_mouse - start_screen;
            let delta_world = delta_screen / state.camera.pixel_scale();
            state.camera.origin = start_origin - delta_world;
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        state.pan_drag_start_screen = None;
        state.pan_drag_start_origin = None;
    }
}

/// Handle zoom via mouse wheel
pub fn handle_zoom(state: &mut ApplicationState) {
    let (_scroll_x, scroll_y) = mouse_wheel();

    if scroll_y != 0.0 {
        let cursor_screen = Vec2::from(mouse_position());
        let zoom_factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        state.camera.zoom_around_cursor(cursor_screen, zoom_factor);
    }
}
