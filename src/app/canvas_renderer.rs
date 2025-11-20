use macroquad::prelude::*;
use std::collections::HashSet;

use super::cell::CellGrid;

const GRID_SIZE: f32 = 10.0;

/// Optimized canvas renderer using render target with dirty cell tracking
/// Only redraws cells that have changed since last frame
pub struct CanvasRenderer {
    render_target: RenderTarget,
    width: u32,
    height: u32,
    dirty_cells: HashSet<(i32, i32)>,
    needs_full_redraw: bool,
}

impl CanvasRenderer {
    pub fn new() -> Self {
        let width = screen_width().max(1.0) as u32;
        let height = screen_height().max(1.0) as u32;
        let rt = render_target(width, height);
        rt.texture.set_filter(FilterMode::Nearest); // Crisp pixel-perfect rendering

        CanvasRenderer {
            render_target: rt,
            width,
            height,
            dirty_cells: HashSet::new(),
            needs_full_redraw: true,
        }
    }

    /// Mark a cell as dirty (needs redrawing)
    pub fn mark_dirty(&mut self, cell_coords: (i32, i32)) {
        self.dirty_cells.insert(cell_coords);
    }

    /// Check if screen size changed and recreate render target if needed
    pub fn update_if_screen_resized(&mut self) {
        let sw = screen_width().max(1.0) as u32;
        let sh = screen_height().max(1.0) as u32;

        if sw != self.width || sh != self.height {
            self.width = sw;
            self.height = sh;
            self.render_target = render_target(sw, sh);
            self.render_target.texture.set_filter(FilterMode::Nearest);
            self.needs_full_redraw = true;
        }
    }

    /// Update the render target by redrawing only dirty cells
    pub fn update(&mut self, cells: &CellGrid) {
        if self.needs_full_redraw || !self.dirty_cells.is_empty() {
            // Always do full redraw when there are changes
            // This ensures erased cells are properly cleared
            self.full_redraw(cells);
            self.needs_full_redraw = false;
            self.dirty_cells.clear();
        }
    }

    /// Redraw all cells (used on initialization or screen resize)
    fn full_redraw(&self, cells: &CellGrid) {
        set_camera(&Camera2D {
            render_target: Some(self.render_target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.width as f32, self.height as f32))
        });

        // Clear to transparent (white background from main loop will show through)
        clear_background(BLANK);

        // Draw all cells
        for (coords, cell) in cells {
            if cell.is_filled {
                self.draw_cell_at_coords(*coords, cell.color);
            }
        }

        set_default_camera();
    }

    /// Redraw only dirty cells
    fn partial_redraw(&self, cells: &CellGrid) {
        set_camera(&Camera2D {
            render_target: Some(self.render_target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.width as f32, self.height as f32))
        });

        for coords in &self.dirty_cells {
            let x = coords.0 as f32 * GRID_SIZE;
            let y = coords.1 as f32 * GRID_SIZE;

            // Clear the cell area
            draw_rectangle(x, y, GRID_SIZE, GRID_SIZE, BLANK);

            // Redraw if cell exists and is filled
            if let Some(cell) = cells.get(coords) {
                if cell.is_filled {
                    self.draw_cell_at_coords(*coords, cell.color);
                }
            }
        }

        set_default_camera();
    }

    /// Draw a single cell at the given grid coordinates
    /// Cells are drawn perfectly within grid squares (not overlapping grid lines)
    fn draw_cell_at_coords(&self, coords: (i32, i32), color: Color) {
        let x = coords.0 as f32 * GRID_SIZE;
        let y = coords.1 as f32 * GRID_SIZE;

        // Draw rectangle that fits perfectly within the grid cell
        // No overlap with grid lines
        draw_rectangle(x, y, GRID_SIZE, GRID_SIZE, color);
    }

    /// Draw the render target to the screen
    pub fn draw(&self) {
        let params = DrawTextureParams {
            dest_size: Some(vec2(self.width as f32, self.height as f32)),
            flip_y: true,  // Render targets are Y-flipped in OpenGL
            ..Default::default()
        };
        draw_texture_ex(&self.render_target.texture, 0.0, 0.0, WHITE, params);
    }
}
