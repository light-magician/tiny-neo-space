use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::core::camera::Camera as AppCamera;
use crate::input::delete_selection;

pub fn draw_selection_overlay(state: &ApplicationState) {
    let camera = &state.camera;

    // Draw active drag rectangle (translucent)
    if state.selection.active_drag {
        if let (Some(start), Some(end)) = (state.selection.drag_start, state.selection.drag_end) {
            draw_selection_rect(camera, start, end, Color::new(0.3, 0.6, 1.0, 0.15), 2.0);
        }
    }

    // Draw finalized selection
    if let Some(sel) = &state.selection.current {
        let rect = &sel.rect;
        let min_screen = camera.cell_to_screen((rect.min_x, rect.min_y));
        let max_screen = camera.cell_to_screen((rect.max_x + 1, rect.max_y + 1));
        let w = max_screen.x - min_screen.x;
        let h = max_screen.y - min_screen.y;

        // Fill
        draw_rectangle(min_screen.x, min_screen.y, w, h, Color::new(0.3, 0.6, 1.0, 0.1));

        // Outline
        draw_rectangle_lines(min_screen.x, min_screen.y, w, h, 2.0,
            Color::new(0.5, 0.8, 1.0, 0.8));

        // During move: show preview offset
        if state.selection.is_moving {
            let pixel_scale = camera.pixel_scale();
            let offset_px_x = state.selection.move_offset_x * pixel_scale;
            let offset_px_y = state.selection.move_offset_y * pixel_scale;

            draw_rectangle_lines(
                min_screen.x + offset_px_x,
                min_screen.y + offset_px_y,
                w, h, 1.0,
                Color::new(1.0, 1.0, 0.3, 0.6), // Yellow preview
            );
        }
    }
}

fn draw_selection_rect(
    camera: &AppCamera,
    p1: (i32, i32),
    p2: (i32, i32),
    fill_color: Color,
    border_width: f32,
) {
    let min_x = p1.0.min(p2.0);
    let max_x = p1.0.max(p2.0);
    let min_y = p1.1.min(p2.1);
    let max_y = p1.1.max(p2.1);

    let min_screen = camera.cell_to_screen((min_x, min_y));
    let max_screen = camera.cell_to_screen((max_x + 1, max_y + 1));

    let w = max_screen.x - min_screen.x;
    let h = max_screen.y - min_screen.y;

    draw_rectangle(min_screen.x, min_screen.y, w, h, fill_color);
    draw_rectangle_lines(min_screen.x, min_screen.y, w, h, border_width,
        Color::new(fill_color.r, fill_color.g, fill_color.b, 0.9));
}

/// Draw action bar for selection
pub fn draw_selection_action_bar(state: &mut ApplicationState) {
    if let Some(sel) = &state.selection.current {
        let rect = &sel.rect;
        let camera = &state.camera;

        // Position bar below selection
        let min_screen = camera.cell_to_screen((rect.min_x, rect.min_y));
        let max_screen = camera.cell_to_screen((rect.max_x + 1, rect.max_y + 1));

        let bar_y = max_screen.y + 4.0;
        let bar_x = min_screen.x;
        let bar_width = (max_screen.x - min_screen.x).max(80.0);
        let bar_height = 28.0;

        // Don't draw if off-screen
        if bar_y > screen_height() || bar_y + bar_height < 0.0 {
            return;
        }

        // Background
        draw_rectangle(bar_x, bar_y, bar_width, bar_height,
            Color::from_rgba(80, 80, 120, 200));
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.0, BLACK);

        // Delete button
        if draw_action_button("Delete", bar_x + 4.0, bar_y + 2.0, 70.0, 24.0) {
            delete_selection(state);
        }
    }
}

fn draw_action_button(label: &str, x: f32, y: f32, w: f32, h: f32) -> bool {
    let mouse_pos = Vec2::from(mouse_position());
    let rect = Rect::new(x, y, w, h);
    let is_hovered = rect.contains(mouse_pos);

    let color = if is_hovered {
        Color::from_rgba(100, 100, 150, 255)
    } else {
        Color::from_rgba(70, 70, 110, 255)
    };

    draw_rectangle(x, y, w, h, color);
    draw_rectangle_lines(x, y, w, h, 1.0, BLACK);

    let text_size = measure_text(label, None, 14, 1.0);
    let text_x = x + (w - text_size.width) / 2.0;
    let text_y = y + (h + text_size.height) / 2.0;
    draw_text(label, text_x, text_y, 14.0, WHITE);

    is_mouse_button_pressed(MouseButton::Left) && is_hovered
}
