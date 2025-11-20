use macroquad::prelude::*;

pub struct GridRenderer {
    rt: RenderTarget,
    width: u32,
    height: u32,
    spacing: f32,
    color: Color,
    thickness: f32,
}

impl GridRenderer {
    pub fn new() -> Self {
        let width = screen_width().max(1.0) as u32;
        let height = screen_height().max(1.0) as u32;
        let rt = render_target(width, height);
        rt.texture.set_filter(FilterMode::Linear);

        let grid = GridRenderer {
            rt,
            width,
            height,
            spacing: 10.0,
            color: Color::from_rgba(210, 225, 255, 255), // light blue lines on white
            thickness: 1.0,
        };

        grid.redraw();
        grid
    }

    fn recreate_target(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.rt = render_target(width, height);
        self.rt.texture.set_filter(FilterMode::Linear);
        self.redraw();
    }

    fn redraw(&self) {
        // Draw the grid into the render target once.
        set_camera(&Camera2D {
            render_target: Some(self.rt.clone()),
            // Map coordinates to pixel space of the render target
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.width as f32, self.height as f32))
        });

        // Transparent background so it overlays the white clear nicely
        clear_background(BLANK);

        let w = self.width as f32;
        let h = self.height as f32;
        let spacing = self.spacing.max(1.0);

        // Vertical lines
        let mut x = 0.0f32;
        while x <= w {
            draw_line(x, 0.0, x, h, self.thickness, self.color);
            x += spacing;
        }

        // Horizontal lines
        let mut y = 0.0f32;
        while y <= h {
            draw_line(0.0, y, w, y, self.thickness, self.color);
            y += spacing;
        }

        set_default_camera();
    }

    pub fn update_if_needed(&mut self) {
        let sw = screen_width().max(1.0) as u32;
        let sh = screen_height().max(1.0) as u32;
        if sw != self.width || sh != self.height {
            self.recreate_target(sw, sh);
        }
    }

    pub fn draw(&self) {
        // Draw the pre-rendered grid texture to fill the screen
        let params = DrawTextureParams {
            dest_size: Some(vec2(self.width as f32, self.height as f32)),
            ..Default::default()
        };
        draw_texture_ex(&self.rt.texture, 0.0, 0.0, WHITE, params);
    }
}
