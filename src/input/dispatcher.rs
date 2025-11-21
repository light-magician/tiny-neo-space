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
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);

    match state.mode {
        Mode::Paint => perform_drawing(state, &world_mouse_pos, false, canvas_renderer),
        Mode::Erase => perform_drawing(state, &world_mouse_pos, true, canvas_renderer),
        Mode::Pan => handle_pan_tool(state, screen_mouse_pos),
        Mode::Select => handle_select_tool(state),
    }
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
