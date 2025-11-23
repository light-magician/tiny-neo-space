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

pub fn render_ui_buttons(state: &mut ApplicationState) -> bool {
    let mut over_ui = false;
    let mouse_pos = Vec2::from(mouse_position());

    // Draw buttons
    if draw_button("Paint", 10.0, 10.0, 80.0, 30.0, state.mode == Mode::Paint) {
        state.mode = Mode::Paint;
    }
    if draw_button("Erase", 100.0, 10.0, 80.0, 30.0, state.mode == Mode::Erase) {
        state.mode = Mode::Erase;
    }
    if draw_button("Pan", 190.0, 10.0, 80.0, 30.0, state.mode == Mode::Pan) {
        state.mode = Mode::Pan;
    }
    if draw_button("Select", 280.0, 10.0, 80.0, 30.0, state.mode == Mode::Select) {
        state.mode = Mode::Select;
    }
    if draw_button("Palette", 370.0, 10.0, 80.0, 30.0, state.show_palette) {
        state.show_palette = !state.show_palette;
    }

    // Check if mouse is over any button
    if mouse_pos.y >= 10.0 && mouse_pos.y <= 40.0 && mouse_pos.x >= 10.0 && mouse_pos.x <= 450.0 {
        over_ui = true;
    }

    over_ui
}

