use macroquad::prelude::*;

pub struct Hud {
    fps: i32,
    frame_time: f32,
}

impl Hud {
    pub fn new() -> Self {
        Hud {
            fps: 0,
            frame_time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_time += dt;
        if self.frame_time >= 1.0 {
            self.fps = (1.0 / dt) as i32;
            self.frame_time = 0.0;
        }
    }

    pub fn draw(&self) {
        let fps_text = format!("FPS: {}", self.fps);
        draw_text(&fps_text, 10.0, screen_height() - 30.0, 20.0, BLACK);
    }
}
