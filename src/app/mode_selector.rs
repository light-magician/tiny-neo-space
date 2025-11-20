use macroquad::math::Vec2;

use super::{drawing::perform_drawing, state::ApplicationState, canvas_renderer::CanvasRenderer};

#[derive(PartialEq)]
pub(crate) enum Mode {
    Paint,
    Erase,
}

pub fn perform_action_based_on_application_state(
    state: &mut ApplicationState,
    mouse_pos: &Vec2,
    canvas_renderer: &mut CanvasRenderer,
) {
    match state.mode {
        Mode::Paint => perform_drawing(state, mouse_pos, false, canvas_renderer),
        Mode::Erase => perform_drawing(state, mouse_pos, true, canvas_renderer),
    }
} 
