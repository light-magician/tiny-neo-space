use macroquad::prelude::*;
use crate::core::camera::Camera as AppCamera;

pub struct Hud {
    fps: i32,
    accum_time: f32,
    accum_frames: i32,
}

impl Hud {
    pub fn new() -> Self {
        Hud {
            fps: 0,
            accum_time: 0.0,
            accum_frames: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.accum_time += dt;
        self.accum_frames += 1;
        if self.accum_time >= 1.0 {
            self.fps = (self.accum_frames as f32 / self.accum_time).round() as i32;
            self.accum_time = 0.0;
            self.accum_frames = 0;
        }
    }

    pub fn draw(&self, camera: &AppCamera) {
        let y_start = screen_height() - 80.0;
        let line_height = 20.0;

        // FPS
        let fps_text = format!("FPS: {}", self.fps);
        draw_text(&fps_text, 10.0, y_start, 18.0, BLACK);

        // Zoom level (as percentage)
        let zoom_text = format!("Zoom: {:.0}%", camera.zoom * 100.0);
        draw_text(&zoom_text, 10.0, y_start + line_height, 18.0, BLACK);

        // Camera position (origin)
        let pos_text = format!("Position: ({:.1}, {:.1})", camera.origin.x, camera.origin.y);
        draw_text(&pos_text, 10.0, y_start + line_height * 2.0, 18.0, BLACK);
    }
}
