use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub color: Color,
    pub is_filled: bool,
}

impl Cell {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Cell {
            color: WHITE,
            is_filled: false,
        }
    }

    pub fn with_color(color: Color) -> Self {
        Cell {
            color,
            is_filled: true,
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.color = WHITE;
        self.is_filled = false;
    }
}

pub type CellGrid = HashMap<(i32, i32), Cell>;

pub fn grid_position_to_cell_coords(pos: &Vec2, grid_size: f32) -> (i32, i32) {
    (
        (pos.x / grid_size).floor() as i32,
        (pos.y / grid_size).floor() as i32,
    )
}

#[allow(dead_code)]
pub fn cell_coords_to_screen_position(coords: (i32, i32), grid_size: f32) -> Vec2 {
    Vec2::new(
        coords.0 as f32 * grid_size,
        coords.1 as f32 * grid_size,
    )
}
