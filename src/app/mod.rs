// src/app/mod.rs
mod buttons;
mod cursor;
mod drawing;
mod hud;
mod mode_selector;
mod state;
mod dragging;
mod screen_object;
mod grid;

use macroquad::prelude::*;

pub use self::state::ApplicationState;

pub async fn run() {
    let mut state = ApplicationState::new();
    let mut hud = hud::Hud::new();
    let mut grid_renderer = grid::GridRenderer::new();

    loop {
        let dt = get_frame_time();
        hud.update(dt);

        // White background
        clear_background(WHITE);

        // Ensure and draw cached grid overlay
        grid_renderer.update_if_needed();
        grid_renderer.draw();
        buttons::render_ui_buttons(&mut state);

        let mouse_pos = Vec2::from(mouse_position());
        mode_selector::perform_action_based_on_application_state(&mut state, &mouse_pos);
        drawing::render_strokes(&mut state);
        cursor::draw_cursor_based_on_mode(&state.mode, mouse_pos);

        hud.draw();

        next_frame().await
    }
}
