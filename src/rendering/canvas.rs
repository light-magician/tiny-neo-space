use macroquad::prelude::*;
use std::collections::HashMap;

use crate::core::*;
use crate::core::camera::Camera as AppCamera;

const CHUNK_SIZE: i32 = 64; // 64×64 cells per chunk
const CHUNK_TEXTURE_SIZE: u32 = 512; // 512×512 pixels (8px per cell)
const CELL_TEXTURE_SIZE: u32 = 8; // Each cell is 8×8 pixels in chunk texture

struct Chunk {
    render_target: RenderTarget,
    dirty: bool,
}

/// Chunked canvas renderer with cached RenderTargets and dirty rebuilds
/// Partitions the world into 64×64 cell chunks, each rendered into a texture
/// Only rebuilds dirty chunks and only draws visible chunks
pub struct CanvasRenderer {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl CanvasRenderer {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    /// Convert cell coordinates to chunk coordinates
    #[inline]
    fn cell_to_chunk(cell_x: i32, cell_y: i32) -> (i32, i32) {
        (
            cell_x.div_euclid(CHUNK_SIZE),
            cell_y.div_euclid(CHUNK_SIZE)
        )
    }

    /// Convert cell coordinates to local coordinates within chunk (0-63)
    #[inline]
    fn cell_to_local(cell_x: i32, cell_y: i32) -> (i32, i32) {
        (
            cell_x.rem_euclid(CHUNK_SIZE),
            cell_y.rem_euclid(CHUNK_SIZE)
        )
    }

    /// Get or create a chunk at the given chunk coordinates
    fn get_or_create_chunk(&mut self, chunk_coords: (i32, i32)) -> &mut Chunk {
        self.chunks.entry(chunk_coords).or_insert_with(|| {
            let rt = render_target(CHUNK_TEXTURE_SIZE, CHUNK_TEXTURE_SIZE);
            rt.texture.set_filter(FilterMode::Nearest);
            Chunk {
                render_target: rt,
                dirty: true, // New chunks start dirty
            }
        })
    }

    /// Mark a cell as dirty (needs redrawing)
    pub fn mark_dirty(&mut self, cell_coords: (i32, i32)) {
        let chunk_coords = Self::cell_to_chunk(cell_coords.0, cell_coords.1);

        // Get or create the chunk and mark it dirty
        let chunk = self.get_or_create_chunk(chunk_coords);
        chunk.dirty = true;
    }

    /// Check if screen size changed (kept for compatibility)
    pub fn update_if_screen_resized(&mut self) {
        // Not needed with chunked rendering
    }

    /// Rebuild all dirty chunks by rendering their cells into RenderTargets
    pub fn update(&mut self, cells: &CellGrid) {
        // Collect dirty chunk coordinates (can't mutate while iterating)
        let dirty_chunks: Vec<(i32, i32)> = self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.dirty)
            .map(|(coords, _)| *coords)
            .collect();

        for chunk_coords in dirty_chunks {
            self.rebuild_chunk(chunk_coords, cells);
        }
    }

    /// Rebuild a single chunk's RenderTarget
    fn rebuild_chunk(&mut self, chunk_coords: (i32, i32), cells: &CellGrid) {
        let chunk = match self.chunks.get_mut(&chunk_coords) {
            Some(c) => c,
            None => return,
        };

        let rt = chunk.render_target.clone();

        // Use Camera2D with positive Y zoom to avoid coordinate flip
        // This uses screen coordinates: (0,0) at top-left, Y-down
        let camera = Camera2D {
            render_target: Some(rt),
            target: vec2(CHUNK_TEXTURE_SIZE as f32 / 2.0, CHUNK_TEXTURE_SIZE as f32 / 2.0),
            zoom: vec2(
                2.0 / CHUNK_TEXTURE_SIZE as f32,
                2.0 / CHUNK_TEXTURE_SIZE as f32  // Positive Y zoom - no flip!
            ),
            ..Default::default()
        };

        set_camera(&camera);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));

        // Calculate chunk's world cell range
        let chunk_min_x = chunk_coords.0 * CHUNK_SIZE;
        let chunk_min_y = chunk_coords.1 * CHUNK_SIZE;

        // Render all cells in this chunk
        for local_x in 0..CHUNK_SIZE {
            for local_y in 0..CHUNK_SIZE {
                let cell_x = chunk_min_x + local_x;
                let cell_y = chunk_min_y + local_y;

                if let Some(cell) = cells.get(&(cell_x, cell_y)) {
                    if cell.is_filled {
                        // Convert local cell coords to pixel coords in texture
                        let px_x = local_x as f32 * CELL_TEXTURE_SIZE as f32;
                        let px_y = local_y as f32 * CELL_TEXTURE_SIZE as f32;

                        draw_rectangle(
                            px_x,
                            px_y,
                            CELL_TEXTURE_SIZE as f32,
                            CELL_TEXTURE_SIZE as f32,
                            cell.color
                        );
                    }
                }
            }
        }

        set_default_camera();

        chunk.dirty = false;
    }

    /// Draw all visible chunks to screen with frustum culling
    pub fn draw(&self, _cells: &CellGrid, camera: &AppCamera) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Get visible world rect in cell coordinates
        let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(screen_w, screen_h);

        // Convert to chunk coordinates
        let min_chunk_x = (min_x.floor() as i32).div_euclid(CHUNK_SIZE);
        let min_chunk_y = (min_y.floor() as i32).div_euclid(CHUNK_SIZE);
        let max_chunk_x = (max_x.ceil() as i32).div_euclid(CHUNK_SIZE);
        let max_chunk_y = (max_y.ceil() as i32).div_euclid(CHUNK_SIZE);

        // Draw all visible chunks
        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_y in min_chunk_y..=max_chunk_y {
                if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_y)) {
                    // Calculate chunk position in world cells
                    let chunk_world_x = chunk_x * CHUNK_SIZE;
                    let chunk_world_y = chunk_y * CHUNK_SIZE;

                    // Convert to screen space
                    let screen_pos = camera.cell_to_screen((chunk_world_x, chunk_world_y));

                    // Calculate size in screen pixels
                    let chunk_size_px = CHUNK_SIZE as f32 * camera.pixel_scale();

                    // --- Pixel Rounding for Seam Elimination ---
                    // At non-integer zoom levels, floating-point rounding can cause 1-pixel
                    // seams between adjacent chunk textures. Optionally round screen positions
                    // and sizes to whole pixels for crisp, seam-free rendering.
                    //
                    // When to enable:
                    // - If visible seams appear between chunks at certain zoom levels
                    // - When pixel-perfect alignment is more important than sub-pixel smoothness
                    //
                    // Trade-offs:
                    // - Enables: Eliminates seams, crisper rendering at most zoom levels
                    // - Disables: Smoother zoom transitions, sub-pixel positioning maintained
                    //
                    // To enable, uncomment the following lines and use the rounded values below:
                    //
                    // let screen_x = screen_pos.x.round();
                    // let screen_y = screen_pos.y.round();
                    // let size = chunk_size_px.round();

                    // Draw the chunk texture
                    draw_texture_ex(
                        &chunk.render_target.texture,
                        screen_pos.x,  // Replace with screen_x if rounding enabled
                        screen_pos.y,  // Replace with screen_y if rounding enabled
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(chunk_size_px, chunk_size_px)),  // Replace with (size, size) if rounding enabled
                            ..Default::default()
                        }
                    );
                }
            }
        }
    }
}
