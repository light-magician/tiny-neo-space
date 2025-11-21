use macroquad::prelude::*;
use std::collections::HashSet;

use crate::core::*;
use crate::core::camera::Camera as AppCamera;

/// Optimized canvas renderer with frustum culling
/// Renders cells directly to screen using camera transforms
pub struct CanvasRenderer {
    dirty_cells: HashSet<(i32, i32)>,
}

impl CanvasRenderer {
    pub fn new() -> Self {
        CanvasRenderer {
            dirty_cells: HashSet::new(),
        }
    }

    /// Mark a cell as dirty (needs redrawing)
    pub fn mark_dirty(&mut self, cell_coords: (i32, i32)) {
        self.dirty_cells.insert(cell_coords);
    }

    /// Check if screen size changed (no longer needed but kept for compatibility)
    pub fn update_if_screen_resized(&mut self) {
        // No longer needed since we're rendering directly
    }

    /// Update (no longer needed but kept for compatibility)
    pub fn update(&mut self, _cells: &CellGrid) {
        self.dirty_cells.clear();
    }

    /// Draw all visible cells to screen with frustum culling
    pub fn draw(&self, cells: &CellGrid, camera: &AppCamera) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(screen_w, screen_h);
        let pixel_scale = camera.pixel_scale();

        // Only draw cells within visible area
        for (coords, cell) in cells.iter() {
            if cell.is_filled {
                let cell_x = coords.0 as f32;
                let cell_y = coords.1 as f32;

                // Frustum culling
                if cell_x >= min_x && cell_x <= max_x && cell_y >= min_y && cell_y <= max_y {
                    let screen_pos = camera.cell_to_screen(*coords);
                    draw_rectangle(screen_pos.x, screen_pos.y, pixel_scale, pixel_scale, cell.color);
                }
            }
        }
    }
}
