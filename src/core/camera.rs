use macroquad::prelude::*;

pub const BASE_CELL_PIXELS: f32 = 24.0;
// At min zoom, 16×16 cells should match one default-zoom cell size
pub const MIN_ZOOM: f32 = 1.0 / 16.0;
pub const MAX_ZOOM: f32 = 4.0;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// World cell coordinates at screen position (0, 0)
    pub origin: Vec2,

    /// Zoom level where 1.0 = BASE_CELL_PIXELS per cell
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            origin: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Get current size in screen pixels of one world cell
    #[inline]
    pub fn pixel_scale(&self) -> f32 {
        BASE_CELL_PIXELS * self.zoom
    }

    /// Convert integer cell coordinates to screen pixels
    pub fn cell_to_screen(&self, cell: (i32, i32)) -> Vec2 {
        let cell_world = Vec2::new(cell.0 as f32, cell.1 as f32);
        (cell_world - self.origin) * self.pixel_scale()
    }

    /// Convert screen pixels to world cell coordinates (float)
    pub fn screen_to_cell(&self, screen: Vec2) -> Vec2 {
        (screen / self.pixel_scale()) + self.origin
    }

    /// Get the world-space rect of the visible canvas area
    pub fn visible_world_rect(&self, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
        let scale = self.pixel_scale();
        let world_min_x = self.origin.x;
        let world_min_y = self.origin.y;
        let world_max_x = self.origin.x + screen_w / scale;
        let world_max_y = self.origin.y + screen_h / scale;
        (world_min_x, world_min_y, world_max_x, world_max_y)
    }

    /// Pan the camera by a delta in world cell units
    pub fn pan_by(&mut self, delta_world: Vec2) {
        self.origin += delta_world;
    }

    /// Zoom around a point on screen (Figma-style zoom)
    pub fn zoom_around_cursor(&mut self, cursor_screen: Vec2, zoom_factor: f32) {
        // Get world position under cursor BEFORE zoom
        let world_before = self.screen_to_cell(cursor_screen);

        // Apply zoom and clamp to valid range
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        // Get world position under cursor AFTER zoom
        let world_after = self.screen_to_cell(cursor_screen);

        // Adjust origin so the world point under cursor stays fixed
        self.origin += world_before - world_after;
    }
}
