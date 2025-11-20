use macroquad::prelude::*;

use super::{mode_selector::Mode, cell::CellGrid};

pub struct ApplicationState {
    pub mode: Mode,
    pub show_palette: bool,
    pub current_color: Color,
    pub cells: CellGrid,
}

impl ApplicationState {
    pub fn new() -> Self {
        ApplicationState {
            mode: Mode::Paint,
            show_palette: false,
            current_color: BLUE,
            cells: CellGrid::new(),
        }
    }
}