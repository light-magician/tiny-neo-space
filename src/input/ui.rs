use macroquad::prelude::*;

use crate::state::{Mode, ApplicationState};

pub fn draw_button(text: &str, x: f32, y: f32, width: f32, height: f32, is_active: bool) -> bool {
    let color = if is_active { DARKGRAY } else { GRAY };
    let rect = Rect::new(x, y, width, height);
    draw_rectangle(x, y, width, height, color);
    draw_rectangle_lines(x, y, width, height, 2.0, BLACK);
    let text_size = measure_text(text, None, 20, 1.0);
    let text_x = x + (width - text_size.width) / 2.0;
    let text_y = y + (height + text_size.height) / 2.0;
    draw_text(text, text_x, text_y, 20.0, BLACK);
    is_mouse_button_pressed(MouseButton::Left) && rect.contains(Vec2::from(mouse_position()))
}

pub fn render_ui_buttons(state: &mut ApplicationState) {
    // Draw buttons
    if draw_button("Paint", 10.0, 10.0, 80.0, 30.0, state.mode == Mode::Paint) {
        state.mode = Mode::Paint;
    }
    if draw_button("Erase", 100.0, 10.0, 80.0, 30.0, state.mode == Mode::Erase) {
        state.mode = Mode::Erase;
    }
    if draw_button("Palette", 190.0, 10.0, 80.0, 30.0, state.show_palette) {
        state.show_palette = !state.show_palette;
    }

    // Draw color palette if visible
    if state.show_palette {
        draw_color_palette(state);
    }
}

/// Game Boy Color inspired palette with a wide variety of colors
fn get_gbc_palette() -> Vec<(&'static str, Color)> {
    vec![
        // Whites and light grays
        ("White", Color::from_rgba(255, 255, 255, 255)),
        ("Light Gray", Color::from_rgba(200, 200, 200, 255)),
        ("Silver", Color::from_rgba(192, 192, 192, 255)),
        ("Gray", Color::from_rgba(128, 128, 128, 255)),

        // Dark grays and black
        ("Dark Gray", Color::from_rgba(80, 80, 80, 255)),
        ("Charcoal", Color::from_rgba(54, 54, 54, 255)),
        ("Black", Color::from_rgba(0, 0, 0, 255)),
        ("Almost Black", Color::from_rgba(20, 20, 20, 255)),

        // Reds
        ("Bright Red", Color::from_rgba(255, 0, 0, 255)),
        ("Red", Color::from_rgba(200, 0, 0, 255)),
        ("Dark Red", Color::from_rgba(139, 0, 0, 255)),
        ("Crimson", Color::from_rgba(220, 20, 60, 255)),

        // Pinks and Magentas
        ("Pink", Color::from_rgba(255, 192, 203, 255)),
        ("Hot Pink", Color::from_rgba(255, 105, 180, 255)),
        ("Magenta", Color::from_rgba(255, 0, 255, 255)),
        ("Dark Magenta", Color::from_rgba(139, 0, 139, 255)),

        // Oranges
        ("Orange", Color::from_rgba(255, 165, 0, 255)),
        ("Dark Orange", Color::from_rgba(255, 140, 0, 255)),
        ("Coral", Color::from_rgba(255, 127, 80, 255)),
        ("Salmon", Color::from_rgba(250, 128, 114, 255)),

        // Yellows
        ("Yellow", Color::from_rgba(255, 255, 0, 255)),
        ("Gold", Color::from_rgba(255, 215, 0, 255)),
        ("Khaki", Color::from_rgba(240, 230, 140, 255)),
        ("Olive", Color::from_rgba(128, 128, 0, 255)),

        // Greens
        ("Lime", Color::from_rgba(0, 255, 0, 255)),
        ("Green", Color::from_rgba(0, 200, 0, 255)),
        ("Forest Green", Color::from_rgba(34, 139, 34, 255)),
        ("Dark Green", Color::from_rgba(0, 100, 0, 255)),

        // Teals and Aquas
        ("Aqua", Color::from_rgba(0, 255, 255, 255)),
        ("Cyan", Color::from_rgba(0, 200, 200, 255)),
        ("Teal", Color::from_rgba(0, 128, 128, 255)),
        ("Dark Cyan", Color::from_rgba(0, 139, 139, 255)),

        // Blues
        ("Sky Blue", Color::from_rgba(135, 206, 235, 255)),
        ("Blue", Color::from_rgba(0, 0, 255, 255)),
        ("Medium Blue", Color::from_rgba(0, 0, 205, 255)),
        ("Navy", Color::from_rgba(0, 0, 128, 255)),

        // Purples
        ("Lavender", Color::from_rgba(230, 230, 250, 255)),
        ("Purple", Color::from_rgba(128, 0, 128, 255)),
        ("Indigo", Color::from_rgba(75, 0, 130, 255)),
        ("Dark Purple", Color::from_rgba(70, 0, 100, 255)),

        // Browns and Earth Tones
        ("Tan", Color::from_rgba(210, 180, 140, 255)),
        ("Brown", Color::from_rgba(165, 42, 42, 255)),
        ("Saddle Brown", Color::from_rgba(139, 69, 19, 255)),
        ("Maroon", Color::from_rgba(128, 0, 0, 255)),

        // Pastels
        ("Peach", Color::from_rgba(255, 218, 185, 255)),
        ("Mint", Color::from_rgba(189, 252, 201, 255)),
        ("Baby Blue", Color::from_rgba(137, 207, 240, 255)),
        ("Lilac", Color::from_rgba(200, 162, 200, 255)),
    ]
}

fn draw_color_palette(state: &mut ApplicationState) {
    let palette_x = state.palette_position.x;
    let palette_y = state.palette_position.y;

    // Larger palette to accommodate more colors
    let palette_width = 360.0;
    let palette_height = 390.0;
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

    // Draw title bar with gradient for visual feedback
    let title_color = if title_bar_rect.contains(mouse_pos) || state.palette_dragging {
        Color::from_rgba(100, 100, 180, 255)
    } else {
        Color::from_rgba(80, 80, 150, 255)
    };

    draw_rectangle(palette_x, palette_y, palette_width, title_bar_height, title_color);
    draw_rectangle_lines(palette_x, palette_y, palette_width, title_bar_height, 2.0, BLACK);

    // Draw title text
    let title = "Game Boy Color Palette - Drag Me!";
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

    // Get colors
    let colors = get_gbc_palette();

    let color_size = 28.0;
    let padding = 6.0;
    let start_x = palette_x + padding;
    let cols_per_row = 8;

    // Draw current color indicator at the top
    let indicator_x = palette_x + padding;
    let indicator_y = content_y + padding;
    let indicator_size = 60.0;

    draw_rectangle(indicator_x, indicator_y, indicator_size, indicator_size, state.current_color);
    draw_rectangle_lines(indicator_x, indicator_y, indicator_size, indicator_size, 3.0, BLACK);

    // Label for current color
    draw_text("Current", indicator_x, indicator_y - 5.0, 16.0, BLACK);

    // Draw all colors in a grid (starting below the indicator)
    let grid_start_y = indicator_y + indicator_size + padding * 2.0;

    for (i, (_name, color)) in colors.iter().enumerate() {
        let col = i % cols_per_row;
        let row = i / cols_per_row;
        let x = start_x + col as f32 * (color_size + padding);
        let y = grid_start_y + row as f32 * (color_size + padding);

        // Draw color square
        draw_rectangle(x, y, color_size, color_size, *color);

        // Highlight if this is the current color
        let border_width = if colors_match(state.current_color, *color) { 3.0 } else { 1.5 };
        let border_color = if colors_match(state.current_color, *color) {
            Color::from_rgba(255, 255, 0, 255) // Yellow highlight
        } else {
            BLACK
        };

        draw_rectangle_lines(x, y, color_size, color_size, border_width, border_color);

        // Check if clicked (only if not dragging)
        if !state.palette_dragging {
            let rect = Rect::new(x, y, color_size, color_size);
            if is_mouse_button_pressed(MouseButton::Left) && rect.contains(mouse_pos) {
                state.current_color = *color;
            }
        }
    }
}

/// Helper function to check if two colors are approximately equal
fn colors_match(c1: Color, c2: Color) -> bool {
    (c1.r - c2.r).abs() < 0.01
        && (c1.g - c2.g).abs() < 0.01
        && (c1.b - c2.b).abs() < 0.01
        && (c1.a - c2.a).abs() < 0.01
}
