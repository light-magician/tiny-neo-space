use macroquad::texture::Texture2D;



pub struct RenderState {
    pub ui_texture: Texture2D,
    pub strokes_texture: Texture2D,
    pub needs_ui_update: bool,
    pub needs_strokes_update: bool,
}

impl RenderState {
    pub fn default() -> Self {
        RenderState {
            ui_texture: Texture2D::empty(),
            strokes_texture: Texture2D::empty(),
            needs_ui_update: true,
            needs_strokes_update: true,
        }
    }
}