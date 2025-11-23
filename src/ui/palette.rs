use macroquad::prelude::*;
use crate::core::color::{GBA_PALETTE, GBA_PALETTE_ROWS, GBA_PALETTE_COLS, generate_gba_extended_palette};
use crate::state::{ApplicationState, PaletteMode};

pub fn render_palette_window(state: &mut ApplicationState) -> bool {
    if !state.show_palette {
        return false;
    }

    let palette_x = state.palette_position.x;
    let palette_y = state.palette_position.y;

    // Palette dimensions (adjusted for new UI elements)
    let palette_width = 200.0;
    let base_height = match state.palette_mode {
        PaletteMode::Basic => 160.0,
        PaletteMode::Extended => 320.0, // Taller for extended mode
    };
    let palette_height = base_height;
    let title_bar_height = 25.0;
    let page_controls_height = 30.0;

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

    // Mode toggle buttons
    let button_y = content_y + 5.0;
    let button_width = 90.0;
    let button_height = 25.0;
    let button_spacing = 5.0;
    let basic_button_x = palette_x + 5.0;
    let extended_button_x = basic_button_x + button_width + button_spacing;

    let basic_button_rect = Rect::new(basic_button_x, button_y, button_width, button_height);
    let extended_button_rect = Rect::new(extended_button_x, button_y, button_width, button_height);

    // Draw Basic button
    let basic_color = match state.palette_mode {
        PaletteMode::Basic => Color::from_rgba(100, 150, 100, 255), // Active green
        PaletteMode::Extended => Color::from_rgba(180, 180, 180, 255), // Inactive gray
    };
    draw_rectangle(basic_button_x, button_y, button_width, button_height, basic_color);
    draw_rectangle_lines(basic_button_x, button_y, button_width, button_height, 2.0, BLACK);
    let basic_text = "Basic";
    let basic_text_size = measure_text(basic_text, None, 16, 1.0);
    draw_text(
        basic_text,
        basic_button_x + (button_width - basic_text_size.width) / 2.0,
        button_y + (button_height + basic_text_size.height) / 2.0,
        16.0,
        BLACK,
    );

    // Draw Extended button
    let extended_color = match state.palette_mode {
        PaletteMode::Extended => Color::from_rgba(100, 150, 100, 255), // Active green
        PaletteMode::Basic => Color::from_rgba(180, 180, 180, 255), // Inactive gray
    };
    draw_rectangle(extended_button_x, button_y, button_width, button_height, extended_color);
    draw_rectangle_lines(extended_button_x, button_y, button_width, button_height, 2.0, BLACK);
    let extended_text = "Extended";
    let extended_text_size = measure_text(extended_text, None, 16, 1.0);
    draw_text(
        extended_text,
        extended_button_x + (button_width - extended_text_size.width) / 2.0,
        button_y + (button_height + extended_text_size.height) / 2.0,
        16.0,
        BLACK,
    );

    // Handle mode button clicks
    if !state.palette_dragging {
        if is_mouse_button_pressed(MouseButton::Left) {
            if basic_button_rect.contains(mouse_pos) {
                state.palette_mode = PaletteMode::Basic;
                state.palette_page = 0; // Reset page when switching modes
            } else if extended_button_rect.contains(mouse_pos) {
                state.palette_mode = PaletteMode::Extended;
                state.palette_page = 0; // Reset page when switching modes
            }
        }
    }

    // Draw color swatches based on mode
    let swatch_start_y = button_y + button_height + 5.0;

    match state.palette_mode {
        PaletteMode::Basic => {
            // Original basic palette layout
            let color_size = 20.0;
            let padding = 4.0;
            let start_x = palette_x + padding;
            let start_y = swatch_start_y;

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
        }
        PaletteMode::Extended => {
            // Extended palette with paging
            let extended_palette = generate_gba_extended_palette();
            let total_colors = extended_palette.len(); // 343 colors
            let colors_per_page = 200;
            let total_pages = (total_colors + colors_per_page - 1) / colors_per_page; // Ceiling division

            // Ensure page is within bounds
            if state.palette_page >= total_pages {
                state.palette_page = total_pages - 1;
            }

            // Calculate which colors to show
            let start_idx = state.palette_page * colors_per_page;
            let end_idx = (start_idx + colors_per_page).min(total_colors);
            let page_colors = &extended_palette[start_idx..end_idx];

            // Layout: 20 columns x 10 rows = 200 colors per page
            let cols = 20;
            let color_size = 8.0;
            let padding = 1.0;
            let start_x = palette_x + 5.0;
            let start_y = swatch_start_y;

            for (idx, rgba) in page_colors.iter().enumerate() {
                let row = idx / cols;
                let col = idx % cols;

                let mq_color = rgba.to_mq_color();
                let x = start_x + col as f32 * (color_size + padding);
                let y = start_y + row as f32 * (color_size + padding);

                // Draw color square
                draw_rectangle(x, y, color_size, color_size, mq_color);

                // Highlight if this is the current color
                let border_width = if colors_match(state.current_color, mq_color) { 2.0 } else { 1.0 };
                let border_color = if colors_match(state.current_color, mq_color) {
                    Color::from_rgba(255, 255, 0, 255) // Yellow highlight
                } else {
                    Color::from_rgba(100, 100, 100, 255) // Gray border
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

            // Page controls at bottom
            let page_controls_y = palette_y + palette_height - page_controls_height - 5.0;
            let prev_button_x = palette_x + 5.0;
            let prev_button_width = 50.0;
            let prev_button_height = 25.0;

            let next_button_x = palette_x + palette_width - 55.0;
            let next_button_width = 50.0;
            let next_button_height = 25.0;

            let prev_button_rect = Rect::new(prev_button_x, page_controls_y, prev_button_width, prev_button_height);
            let next_button_rect = Rect::new(next_button_x, page_controls_y, next_button_width, next_button_height);

            // Draw Prev button
            let prev_active = state.palette_page > 0;
            let prev_color = if prev_active {
                Color::from_rgba(100, 100, 200, 255)
            } else {
                Color::from_rgba(150, 150, 150, 255)
            };
            draw_rectangle(prev_button_x, page_controls_y, prev_button_width, prev_button_height, prev_color);
            draw_rectangle_lines(prev_button_x, page_controls_y, prev_button_width, prev_button_height, 2.0, BLACK);
            let prev_text = "< Prev";
            let prev_text_size = measure_text(prev_text, None, 14, 1.0);
            draw_text(
                prev_text,
                prev_button_x + (prev_button_width - prev_text_size.width) / 2.0,
                page_controls_y + (prev_button_height + prev_text_size.height) / 2.0,
                14.0,
                BLACK,
            );

            // Draw Next button
            let next_active = state.palette_page < total_pages - 1;
            let next_color = if next_active {
                Color::from_rgba(100, 100, 200, 255)
            } else {
                Color::from_rgba(150, 150, 150, 255)
            };
            draw_rectangle(next_button_x, page_controls_y, next_button_width, next_button_height, next_color);
            draw_rectangle_lines(next_button_x, page_controls_y, next_button_width, next_button_height, 2.0, BLACK);
            let next_text = "Next >";
            let next_text_size = measure_text(next_text, None, 14, 1.0);
            draw_text(
                next_text,
                next_button_x + (next_button_width - next_text_size.width) / 2.0,
                page_controls_y + (next_button_height + next_text_size.height) / 2.0,
                14.0,
                BLACK,
            );

            // Draw page indicator
            let page_text = format!("Page {}/{}", state.palette_page + 1, total_pages);
            let page_text_size = measure_text(&page_text, None, 14, 1.0);
            draw_text(
                &page_text,
                palette_x + (palette_width - page_text_size.width) / 2.0,
                page_controls_y + (prev_button_height + page_text_size.height) / 2.0,
                14.0,
                BLACK,
            );

            // Handle page button clicks
            if !state.palette_dragging {
                if is_mouse_button_pressed(MouseButton::Left) {
                    if prev_button_rect.contains(mouse_pos) && prev_active {
                        state.palette_page = state.palette_page.saturating_sub(1);
                    } else if next_button_rect.contains(mouse_pos) && next_active {
                        state.palette_page = (state.palette_page + 1).min(total_pages - 1);
                    }
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
