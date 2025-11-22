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

        // LAYER 1: Grid (behind everything except canvas)
        grid_renderer.update_if_needed();
        grid_renderer.draw(&state.camera);

        // LAYER 2: Canvas
        canvas_renderer.update_if_screen_resized();
        canvas_renderer.update(&state.cells);
        canvas_renderer.draw(&state.cells, &state.camera);

        // LAYER 3: Selection overlay
        draw_selection_overlay(&state);

        // Check if mouse is over UI
        let over_buttons = render_ui_buttons(&mut state);
        let over_palette = render_palette_window(&mut state);
        let over_ui = over_buttons || over_palette;

        // Handle zoom (scroll wheel) - only if not over UI
        if !over_ui {
            handle_zoom(&mut state);
        }

        // Handle user input (painting/erasing/panning) - only if not over UI
        if !over_ui {
            handle_input(&mut state, &mut canvas_renderer);
        }

        // LAYER 4: Cursor (only if not over UI)
        if !over_ui {
            let screen_mouse_pos = Vec2::from(mouse_position());
            draw_cursor_based_on_mode(&state.mode, &state.camera, screen_mouse_pos);
        }

        // LAYER 5: Selection action bar (on top of everything)
        draw_selection_action_bar(&mut state);

        // LAYER 6: HUD (with camera info)
        hud.draw(&state.camera);

        next_frame().await
    }
}
