// src/app/mod.rs
// Clean game loop using new modular architecture

use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::rendering::{CanvasRenderer, GridRenderer, Hud, draw_cursor_based_on_mode};
use crate::input::{handle_input, render_ui_buttons};

pub async fn run() {
    let mut state = ApplicationState::new();
    let mut hud = Hud::new();
    let mut grid_renderer = GridRenderer::new();
    let mut canvas_renderer = CanvasRenderer::new();

    loop {
        let dt = get_frame_time();
        hud.update(dt);

        // White background
        clear_background(WHITE);

        // Ensure grid is sized correctly
        grid_renderer.update_if_needed();
        grid_renderer.draw(state.camera_offset);

        // Handle user input (painting/erasing)
        handle_input(&mut state, &mut canvas_renderer);

        // Update canvas renderer (only redraws dirty cells)
        canvas_renderer.update_if_screen_resized();
        canvas_renderer.update(&state.cells);

        // Draw the canvas
        canvas_renderer.draw(&state.cells, state.camera_offset);

        // Draw cursor (in screen space)
        let screen_mouse_pos = Vec2::from(mouse_position());
        draw_cursor_based_on_mode(&state.mode, screen_mouse_pos);

        // Draw buttons and UI last so they appear above everything
        render_ui_buttons(&mut state);
        hud.draw();

        next_frame().await
    }
}
