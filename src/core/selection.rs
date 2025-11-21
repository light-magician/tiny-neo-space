use crate::core::cell::CellGrid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRect {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl SelectionRect {
    pub fn from_points(p1: (i32, i32), p2: (i32, i32)) -> Self {
        Self {
            min_x: p1.0.min(p2.0),
            min_y: p1.1.min(p2.1),
            max_x: p1.0.max(p2.0),
            max_y: p1.1.max(p2.1),
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn width(&self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn height(&self) -> i32 {
        self.max_y - self.min_y + 1
    }
}

#[derive(Clone, Debug)]
pub enum SelectionKind {
    Cells(Vec<(i32, i32)>),
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub rect: SelectionRect,
    pub kind: SelectionKind,
}

/// Main selection state tracking
#[derive(Clone, Debug)]
pub struct SelectionState {
    /// Is user actively dragging to create selection?
    pub active_drag: bool,

    /// Drag start point (cell coordinates)
    pub drag_start: Option<(i32, i32)>,

    /// Current drag end point (updates every frame during drag)
    pub drag_end: Option<(i32, i32)>,

    /// Committed selection (finalized when mouse released)
    pub current: Option<Selection>,

    /// Move mode: is user moving current selection?
    pub is_moving: bool,

    /// During move: accumulated float offset for smooth movement
    pub move_offset_x: f32,
    pub move_offset_y: f32,

    /// Last mouse position in world space (for delta calculation)
    pub last_move_mouse: Option<(f32, f32)>,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            active_drag: false,
            drag_start: None,
            drag_end: None,
            current: None,
            is_moving: false,
            move_offset_x: 0.0,
            move_offset_y: 0.0,
            last_move_mouse: None,
        }
    }
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn start_drag(&mut self, start: (i32, i32)) {
        self.active_drag = true;
        self.drag_start = Some(start);
        self.drag_end = Some(start);
        self.is_moving = false;
    }

    pub fn update_drag_end(&mut self, end: (i32, i32)) {
        if self.active_drag {
            self.drag_end = Some(end);
        }
    }

    pub fn finalize_drag(&mut self, cells: &CellGrid) -> bool {
        self.active_drag = false;

        if let (Some(start), Some(end)) = (self.drag_start, self.drag_end) {
            let rect = SelectionRect::from_points(start, end);

            let selected_cells: Vec<(i32, i32)> = cells
                .iter()
                .filter_map(|(coord, cell)| {
                    if cell.is_filled && rect.contains(coord.0, coord.1) {
                        Some(*coord)
                    } else {
                        None
                    }
                })
                .collect();

            if selected_cells.is_empty() {
                self.current = None;
                return false;
            }

            self.current = Some(Selection {
                rect,
                kind: SelectionKind::Cells(selected_cells),
            });
            return true;
        }

        self.current = None;
        false
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        if let Some(sel) = &self.current {
            sel.rect.contains(x, y)
        } else {
            false
        }
    }

    pub fn start_move(&mut self, mouse_world: (f32, f32)) {
        if self.current.is_some() {
            self.is_moving = true;
            self.move_offset_x = 0.0;
            self.move_offset_y = 0.0;
            self.last_move_mouse = Some(mouse_world);
        }
    }

    pub fn update_move(&mut self, delta_x: f32, delta_y: f32) {
        if self.is_moving {
            self.move_offset_x += delta_x;
            self.move_offset_y += delta_y;
        }
    }

    pub fn finalize_move(&mut self) -> Option<(i32, i32)> {
        if !self.is_moving {
            return None;
        }

        self.is_moving = false;

        let offset_x = self.move_offset_x.round() as i32;
        let offset_y = self.move_offset_y.round() as i32;

        if offset_x == 0 && offset_y == 0 {
            self.move_offset_x = 0.0;
            self.move_offset_y = 0.0;
            return None;
        }

        if let Some(sel) = &mut self.current {
            if let SelectionKind::Cells(coords) = &mut sel.kind {
                // Update cell coordinates
                *coords = coords.iter().map(|(x, y)| (x + offset_x, y + offset_y)).collect();

                // Update rect
                sel.rect.min_x += offset_x;
                sel.rect.max_x += offset_x;
                sel.rect.min_y += offset_y;
                sel.rect.max_y += offset_y;
            }
        }

        self.move_offset_x = 0.0;
        self.move_offset_y = 0.0;
        Some((offset_x, offset_y))
    }
}
