// src/app/mod.rs
mod buttons;
mod cursor;
mod drawing;
mod hud;
mod mode_selector;
mod state;
mod screen_object;
mod grid;
mod cell;
mod canvas_renderer;

use macroquad::prelude::*;

pub use self::state::ApplicationState;

pub async fn run() {
    let mut state = ApplicationState::new();
    let mut hud = hud::Hud::new();
    let mut grid_renderer = grid::GridRenderer::new();
    let mut canvas_renderer = canvas_renderer::CanvasRenderer::new();

    loop {
        let dt = get_frame_time();
        hud.update(dt);

        // White background
        clear_background(WHITE);

        // Ensure grid is sized correctly
        grid_renderer.update_if_needed();
        grid_renderer.draw();

        // Handle user input (painting/erasing)
        let mouse_pos = Vec2::from(mouse_position());
        mode_selector::perform_action_based_on_application_state(&mut state, &mouse_pos, &mut canvas_renderer);

        // Update canvas renderer (only redraws dirty cells)
        canvas_renderer.update_if_screen_resized();
        canvas_renderer.update(&state.cells);

        // Draw the canvas
        canvas_renderer.draw();

        // Draw cursor
        cursor::draw_cursor_based_on_mode(&state.mode, mouse_pos);

        // Draw buttons and UI last so they appear above everything
        buttons::render_ui_buttons(&mut state);
        hud.draw();

        next_frame().await
    }
}
