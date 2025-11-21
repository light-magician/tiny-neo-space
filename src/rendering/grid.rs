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
        let start_x = ((min_x.floor() as i32) / step_cells) * step_cells;
        let start_y = ((min_y.floor() as i32) / step_cells) * step_cells;
        let end_x = ((max_x.ceil() as i32) / step_cells + 1) * step_cells;
        let end_y = ((max_y.ceil() as i32) / step_cells + 1) * step_cells;

        let (line_color, line_thickness) = get_grid_appearance(step_cells);

        // Draw vertical lines
        let mut x = start_x;
        while x <= end_x {
            let p0 = camera.cell_to_screen((x, start_y));
            let p1 = camera.cell_to_screen((x, end_y));
            draw_line(p0.x, p0.y, p1.x, p1.y, line_thickness, line_color);
            x += step_cells;
        }

        // Draw horizontal lines
        let mut y = start_y;
        while y <= end_y {
            let p0 = camera.cell_to_screen((start_x, y));
            let p1 = camera.cell_to_screen((end_x, y));
            draw_line(p0.x, p0.y, p1.x, p1.y, line_thickness, line_color);
            y += step_cells;
        }

        // Draw tile boundaries (every 16 cells) if zoomed out
        if step_cells > 1 {
            self.draw_tile_boundaries(camera, start_x, end_x, start_y, end_y);
        }
    }

    fn draw_tile_boundaries(&self, camera: &AppCamera, min_x: i32, max_x: i32, min_y: i32, max_y: i32) {
        let tile_step = 16;
        let tile_color = Color::new(0.50, 0.60, 0.80, 0.60);
        let tile_thickness = 1.2;

        let tile_start_x = (min_x / tile_step) * tile_step;
        let tile_start_y = (min_y / tile_step) * tile_step;
        let tile_end_x = ((max_x / tile_step) + 1) * tile_step;
        let tile_end_y = ((max_y / tile_step) + 1) * tile_step;

        // Vertical tile boundaries
        let mut x = tile_start_x;
        while x <= tile_end_x {
            let p0 = camera.cell_to_screen((x, tile_start_y));
            let p1 = camera.cell_to_screen((x, tile_end_y));
            draw_line(p0.x, p0.y, p1.x, p1.y, tile_thickness, tile_color);
            x += tile_step;
        }

        // Horizontal tile boundaries
        let mut y = tile_start_y;
        while y <= tile_end_y {
            let p0 = camera.cell_to_screen((tile_start_x, y));
            let p1 = camera.cell_to_screen((tile_end_x, y));
            draw_line(p0.x, p0.y, p1.x, p1.y, tile_thickness, tile_color);
            y += tile_step;
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
