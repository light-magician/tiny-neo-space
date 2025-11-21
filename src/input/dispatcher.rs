use macroquad::prelude::*;
use crate::state::{Mode, ApplicationState};
use crate::rendering::CanvasRenderer;
use super::tools::perform_drawing;

/// Central input dispatcher that handles all user input based on current application state
pub fn handle_input(
    state: &mut ApplicationState,
    canvas_renderer: &mut CanvasRenderer,
) {
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = screen_mouse_pos + state.camera_offset;

    match state.mode {
        Mode::Paint => perform_drawing(state, &world_mouse_pos, false, canvas_renderer),
        Mode::Erase => perform_drawing(state, &world_mouse_pos, true, canvas_renderer),
    }
}
