use macroquad::math::Vec2;

use super::{dragging::perform_dragging, drawing::perform_drawing, state::ApplicationState};

#[derive(PartialEq)]
pub(crate) enum Mode {
    Draw,
    Grab,
}

pub fn perform_action_based_on_application_state(state: &mut ApplicationState, mouse_pos: &Vec2) {
    match state.mode {
        Mode::Draw => perform_drawing(state, mouse_pos),
        Mode::Grab => perform_dragging(state, mouse_pos)
    }
} 
