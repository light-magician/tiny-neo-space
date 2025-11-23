use macroquad::prelude::*;
use crate::core::camera::Camera as AppCamera;

/// Compute LOD level and blend factor from zoom
fn compute_lod(zoom: f32) -> (i32, f32) {
    let step_smooth = 1.0 / zoom;        // doubles when zoom halves
    let lod_f = step_smooth.log2();      // 0 @ 1.0x, 1 @ 0.5x, 2 @ 0.25x
    let lod = lod_f.floor().max(0.0) as i32;
    let blend = (lod_f - lod as f32).clamp(0.0, 1.0);
    let step = 1 << lod;                 // 2^lod
    (step, blend)
}

/// Snap to pixel center for crisp lines
#[inline]
fn snap_px(v: f32) -> f32 {
    v.round() + 0.5
}

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

        // Compute LOD step and fade factor
        let (step, blend) = compute_lod(camera.zoom);

        // Compute start/end aligned to step using Euclidean division (correct for negatives)
        let start_x = (min_x.floor() as i32).div_euclid(step) * step;
        let start_y = (min_y.floor() as i32).div_euclid(step) * step;
        let end_x   = (max_x.ceil()  as i32).div_euclid(step) * step + step;
        let end_y   = (max_y.ceil()  as i32).div_euclid(step) * step + step;

        let thickness = 1.0; // screen-space pixels
        let base = Color::new(0.70, 0.75, 0.85, 0.45);

        // Draw vertical lines
        let mut x = start_x;
        while x <= end_x {
            if x % step != 0 {
                x += 1;
                continue;
            }

            // Determine if this line survives to the next LOD level
            let survives_next = (x % (step * 2)) == 0;
            let mut alpha_mul = if survives_next {
                1.0
            } else {
                1.0 - blend
            };

            // Emphasize tile boundaries (every 16 cells)
            let is_tile = (x % 16) == 0;
            if is_tile {
                alpha_mul = (alpha_mul * 1.15).min(1.0);
            }

            // Only draw if visible
            if alpha_mul > 0.001 {
                let p0 = camera.cell_to_screen((x, start_y));
                let p1 = camera.cell_to_screen((x, end_y));
                let col = Color::new(base.r, base.g, base.b, base.a * alpha_mul);
                draw_line(
                    snap_px(p0.x), snap_px(p0.y),
                    snap_px(p1.x), snap_px(p1.y),
                    thickness, col
                );
            }
            x += step;
        }

        // Draw horizontal lines (mirror of vertical)
        let mut y = start_y;
        while y <= end_y {
            if y % step != 0 {
                y += 1;
                continue;
            }

            let survives_next = (y % (step * 2)) == 0;
            let mut alpha_mul = if survives_next {
                1.0
            } else {
                1.0 - blend
            };

            let is_tile = (y % 16) == 0;
            if is_tile {
                alpha_mul = (alpha_mul * 1.15).min(1.0);
            }

            if alpha_mul > 0.001 {
                let p0 = camera.cell_to_screen((start_x, y));
                let p1 = camera.cell_to_screen((end_x, y));
                let col = Color::new(base.r, base.g, base.b, base.a * alpha_mul);
                draw_line(
                    snap_px(p0.x), snap_px(p0.y),
                    snap_px(p1.x), snap_px(p1.y),
                    thickness, col
                );
            }
            y += step;
        }
    }

}
