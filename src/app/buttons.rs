use macroquad::prelude::*;

use super::{mode_selector::Mode, ApplicationState};

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

fn draw_color_palette(state: &mut ApplicationState) {
    let palette_x = 10.0;
    let palette_y = 50.0;
    let palette_width = 170.0;
    let palette_height = 120.0;

    // Draw palette background
    draw_rectangle(palette_x, palette_y, palette_width, palette_height, Color::from_rgba(240, 240, 240, 255));
    draw_rectangle_lines(palette_x, palette_y, palette_width, palette_height, 2.0, BLACK);

    // Primary colors
    let colors = [
        ("Red", RED),
        ("Green", GREEN),
        ("Blue", BLUE),
        ("Yellow", YELLOW),
        ("Cyan", Color::from_rgba(0, 255, 255, 255)),
        ("Magenta", MAGENTA),
        ("Black", BLACK),
        ("White", WHITE),
    ];

    let color_size = 30.0;
    let padding = 10.0;
    let start_x = palette_x + padding;
    let start_y = palette_y + padding;

    for (i, (_name, color)) in colors.iter().enumerate() {
        let col = i % 4;
        let row = i / 4;
        let x = start_x + col as f32 * (color_size + padding);
        let y = start_y + row as f32 * (color_size + padding);

        // Draw color square
        draw_rectangle(x, y, color_size, color_size, *color);
        draw_rectangle_lines(x, y, color_size, color_size, 2.0, BLACK);

        // Check if clicked
        let rect = Rect::new(x, y, color_size, color_size);
        if is_mouse_button_pressed(MouseButton::Left) && rect.contains(Vec2::from(mouse_position())) {
            state.current_color = *color;
        }
    }
}