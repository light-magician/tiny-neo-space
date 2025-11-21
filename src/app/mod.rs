// src/app/mod.rs
// Clean game loop using new modular architecture

use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::rendering::{CanvasRenderer, GridRenderer, Hud, draw_cursor_based_on_mode, draw_selection_overlay, draw_selection_action_bar};
use crate::input::{handle_input, handle_zoom, render_ui_buttons};
use crate::ui::render_palette_window;

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

        // Draw buttons and check if we should interact with canvas
        render_ui_buttons(&mut state);

        // Render palette window and check if mouse is over UI
        let over_ui = render_palette_window(&mut state);

        // Handle zoom (scroll wheel) - only if not over UI
        if !over_ui {
            handle_zoom(&mut state);
        }

        // Ensure grid is sized correctly
        grid_renderer.update_if_needed();
        grid_renderer.draw(&state.camera);

        // Handle user input (painting/erasing/panning) - only if not over UI
        if !over_ui {
            handle_input(&mut state, &mut canvas_renderer);
        }

        // Update canvas renderer (only redraws dirty cells)
        canvas_renderer.update_if_screen_resized();
        canvas_renderer.update(&state.cells);

        // Draw the canvas
        canvas_renderer.draw(&state.cells, &state.camera);

        // Draw selection overlay
        draw_selection_overlay(&state);

        // Draw cursor (in screen space) - only if not over UI
        if !over_ui {
            let screen_mouse_pos = Vec2::from(mouse_position());
            draw_cursor_based_on_mode(&state.mode, &state.camera, screen_mouse_pos);
        }

        // Draw selection action bar
        draw_selection_action_bar(&mut state);

        hud.draw();

        next_frame().await
    }
}
