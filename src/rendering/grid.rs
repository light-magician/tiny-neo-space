use macroquad::prelude::*;
use crate::core::camera::Camera as AppCamera;

pub struct GridRenderer {}

impl GridRenderer {
    pub fn new() -> Self {
        GridRenderer {}
    }

    pub fn update_if_needed(&mut self) {
        // No longer needed since we're rendering directly
    }

    pub fn draw(&self, camera: &AppCamera) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(screen_w, screen_h);

        let cell_px = camera.pixel_scale();
        let step_cells = grid_step_cells(cell_px);

        // Align start/end to grid step
        let start_x = (min_x.floor() as i32 / step_cells) * step_cells;
        let start_y = (min_y.floor() as i32 / step_cells) * step_cells;
        let end_x = (max_x.ceil() as i32 / step_cells + 1) * step_cells;
        let end_y = (max_y.ceil() as i32 / step_cells + 1) * step_cells;

        let (line_color, line_thickness) = get_grid_appearance(step_cells);

        // Draw vertical lines
        let mut x = start_x;
        while x <= end_x {
            let p0 = camera.cell_to_screen((x, start_y));
            let p1 = camera.cell_to_screen((x, end_y));

            // Make tile boundaries (every 16 cells) slightly more prominent
            let is_tile_boundary = x % 16 == 0;
            let thickness = if is_tile_boundary && step_cells > 1 { line_thickness + 0.3 } else { line_thickness };
            let color = if is_tile_boundary && step_cells > 1 {
                Color::new(line_color.r * 0.8, line_color.g * 0.8, line_color.b * 0.8, line_color.a * 1.2)
            } else {
                line_color
            };

            draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
            x += step_cells;
        }

        // Draw horizontal lines
        let mut y = start_y;
        while y <= end_y {
            let p0 = camera.cell_to_screen((start_x, y));
            let p1 = camera.cell_to_screen((end_x, y));

            // Make tile boundaries (every 16 cells) slightly more prominent
            let is_tile_boundary = y % 16 == 0;
            let thickness = if is_tile_boundary && step_cells > 1 { line_thickness + 0.3 } else { line_thickness };
            let color = if is_tile_boundary && step_cells > 1 {
                Color::new(line_color.r * 0.8, line_color.g * 0.8, line_color.b * 0.8, line_color.a * 1.2)
            } else {
                line_color
            };

            draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
            y += step_cells;
        }
    }

}

fn grid_step_cells(cell_px: f32) -> i32 {
    if cell_px >= 16.0 { 1 }
    else if cell_px >= 8.0 { 2 }
    else if cell_px >= 4.0 { 4 }
    else if cell_px >= 2.0 { 8 }
    else { 16 }
}

fn get_grid_appearance(step_cells: i32) -> (Color, f32) {
    match step_cells {
        1 => (Color::new(0.80, 0.85, 0.95, 0.30), 0.5),
        2 => (Color::new(0.75, 0.80, 0.90, 0.35), 0.6),
        4 => (Color::new(0.70, 0.75, 0.85, 0.40), 0.7),
        8 => (Color::new(0.65, 0.70, 0.80, 0.45), 0.8),
        _ => (Color::new(0.60, 0.65, 0.75, 0.50), 1.0),
    }
}
