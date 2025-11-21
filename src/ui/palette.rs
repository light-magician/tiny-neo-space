use macroquad::prelude::*;
use crate::core::color::{GBA_PALETTE, GBA_PALETTE_ROWS, GBA_PALETTE_COLS};
use crate::state::ApplicationState;

pub fn render_palette_window(state: &mut ApplicationState) -> bool {
    if !state.show_palette {
        return false;
    }

    let palette_x = state.palette_position.x;
    let palette_y = state.palette_position.y;

    // Palette dimensions
    let palette_width = 200.0;
    let palette_height = 160.0;
    let title_bar_height = 25.0;

    let mouse_pos = Vec2::from(mouse_position());

    // Title bar for dragging
    let title_bar_rect = Rect::new(palette_x, palette_y, palette_width, title_bar_height);

    // Handle dragging
    if is_mouse_button_pressed(MouseButton::Left) && title_bar_rect.contains(mouse_pos) {
        state.palette_dragging = true;
        state.palette_drag_offset = mouse_pos - state.palette_position;
    }

    if is_mouse_button_released(MouseButton::Left) {
        state.palette_dragging = false;
    }

    if state.palette_dragging {
        state.palette_position = mouse_pos - state.palette_drag_offset;
    }

    // Draw title bar
    let title_color = if title_bar_rect.contains(mouse_pos) || state.palette_dragging {
        Color::from_rgba(100, 100, 180, 255)
    } else {
        Color::from_rgba(80, 80, 150, 255)
    };

    draw_rectangle(palette_x, palette_y, palette_width, title_bar_height, title_color);
    draw_rectangle_lines(palette_x, palette_y, palette_width, title_bar_height, 2.0, BLACK);

    // Draw title text
    let title = "GBA Color Palette";
    let title_size = measure_text(title, None, 16, 1.0);
    draw_text(
        title,
        palette_x + (palette_width - title_size.width) / 2.0,
        palette_y + (title_bar_height + title_size.height) / 2.0,
        16.0,
        WHITE,
    );

    // Draw palette background
    let content_y = palette_y + title_bar_height;
    let content_height = palette_height - title_bar_height;
    draw_rectangle(palette_x, content_y, palette_width, content_height, Color::from_rgba(230, 230, 230, 255));
    draw_rectangle_lines(palette_x, content_y, palette_width, content_height, 2.0, BLACK);

    // Draw color buttons
    let color_size = 20.0;
    let padding = 4.0;
    let start_x = palette_x + padding;
    let start_y = content_y + padding;

    for row in 0..GBA_PALETTE_ROWS {
        for col in 0..GBA_PALETTE_COLS {
            let rgba = GBA_PALETTE[row][col];
            let mq_color = rgba.to_mq_color();

            let x = start_x + col as f32 * (color_size + padding);
            let y = start_y + row as f32 * (color_size + padding);

            // Draw color square
            draw_rectangle(x, y, color_size, color_size, mq_color);

            // Highlight if this is the current color
            let border_width = if colors_match(state.current_color, mq_color) { 3.0 } else { 1.5 };
            let border_color = if colors_match(state.current_color, mq_color) {
                Color::from_rgba(255, 255, 0, 255) // Yellow highlight
            } else {
                BLACK
            };

            draw_rectangle_lines(x, y, color_size, color_size, border_width, border_color);

            // Check if clicked (only if not dragging title bar)
            if !state.palette_dragging {
                let rect = Rect::new(x, y, color_size, color_size);
                if is_mouse_button_pressed(MouseButton::Left) && rect.contains(mouse_pos) {
                    state.current_color = mq_color;
                }
            }
        }
    }

    // Check if mouse is over palette window
    let full_rect = Rect::new(palette_x, palette_y, palette_width, palette_height);
    full_rect.contains(mouse_pos)
}

fn colors_match(c1: Color, c2: Color) -> bool {
    (c1.r - c2.r).abs() < 0.01
        && (c1.g - c2.g).abs() < 0.01
        && (c1.b - c2.b).abs() < 0.01
        && (c1.a - c2.a).abs() < 0.01
}
