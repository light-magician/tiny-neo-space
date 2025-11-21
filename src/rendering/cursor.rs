use macroquad::prelude::*;

use crate::state::Mode;
use crate::core::camera::Camera as AppCamera;

pub fn draw_cursor_based_on_mode(mode: &Mode, camera: &AppCamera, screen_mouse: Vec2) {
    let world_mouse = camera.screen_to_cell(screen_mouse);
    let cell_coords = (world_mouse.x.floor() as i32, world_mouse.y.floor() as i32);
    let cell_screen_pos = camera.cell_to_screen(cell_coords);
    let cell_size = camera.pixel_scale();

    match mode {
        Mode::Paint => {
            // Draw highlight box around the cell
            draw_rectangle_lines(cell_screen_pos.x, cell_screen_pos.y, cell_size, cell_size, 2.0, Color::from_rgba(0, 0, 0, 150));
            // Small cursor dot
            draw_circle(screen_mouse.x, screen_mouse.y, 3.0, BLACK);
        }
        Mode::Erase => {
            // Draw red highlight for eraser
            draw_rectangle_lines(cell_screen_pos.x, cell_screen_pos.y, cell_size, cell_size, 2.0, Color::from_rgba(255, 100, 100, 200));
            // Eraser cursor
            draw_rectangle(screen_mouse.x - 5.0, screen_mouse.y - 5.0, 10.0, 10.0, Color::from_rgba(255, 100, 100, 150));
        }
        Mode::Pan => {
            // Hand cursor for panning
            draw_circle(screen_mouse.x, screen_mouse.y, 4.0, DARKGRAY);
        }
        Mode::Select => {
            // Crosshair cursor for selection
            let size = 8.0;
            draw_line(screen_mouse.x - size, screen_mouse.y, screen_mouse.x + size, screen_mouse.y, 2.0, Color::from_rgba(100, 100, 200, 200));
            draw_line(screen_mouse.x, screen_mouse.y - size, screen_mouse.x, screen_mouse.y + size, 2.0, Color::from_rgba(100, 100, 200, 200));
        }
    }
}
