//! Application State Module
//!
//! This module contains the core application state types and structures.
//! It includes the Mode enum for controlling the current editing mode
//! (Paint/Erase) and the ApplicationState struct which holds all the
//! global state for the application including the current mode, color
//! palette visibility, selected color, cell grid, and camera offset.

use macroquad::prelude::*;
use crate::core::*;
use crate::core::camera::Camera as AppCamera;
use crate::core::cell::Cell;
use std::collections::HashMap;

/// Represents the current editing mode of the application
#[derive(PartialEq)]
pub enum Mode {
    /// Paint mode - adds cells with the current color
    Paint,
    /// Erase mode - removes cells from the grid
    Erase,
    /// Pan mode - move the camera
    Pan,
    /// Select mode - select and move cells
    Select,
}

/// Clipboard for storing copied/cut cells
pub struct Clipboard {
    pub width: i32,
    pub height: i32,
    pub cells: HashMap<(i32, i32), Cell>,
    pub has_data: bool,
}

impl Clipboard {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            cells: HashMap::new(),
            has_data: false,
        }
    }
}

/// Represents a change to a single cell for undo/redo
pub struct CellChange {
    pub coord: (i32, i32),
    pub before: Option<Cell>,
    pub after: Option<Cell>,
}

/// Represents a command that can be undone
pub struct Command {
    pub changes: Vec<CellChange>,
}

/// History stack for undo/redo functionality
pub struct History {
    pub stack: Vec<Command>,
    pub max: usize,
}

impl History {
    pub fn new(max: usize) -> Self {
        Self {
            stack: Vec::new(),
            max,
        }
    }

    pub fn push(&mut self, cmd: Command) {
        self.stack.push(cmd);
        if self.stack.len() > self.max {
            self.stack.remove(0);
        }
    }

    pub fn pop(&mut self) -> Option<Command> {
        self.stack.pop()
    }
}

/// Palette display mode
#[derive(Clone, Debug)]
pub enum PaletteMode {
    Basic,
    Extended,
}

/// The main application state containing all global state
pub struct ApplicationState {
    /// Current editing mode (Paint or Erase)
    pub mode: Mode,
    /// Whether the color palette UI is visible
    pub show_palette: bool,
    /// The currently selected color for painting
    pub current_color: Color,
    /// The grid of cells (sparse HashMap-based grid)
    pub cells: CellGrid,
    /// Camera with zoom and pan support
    pub camera: AppCamera,
    /// Position of the color palette window
    pub palette_position: Vec2,
    /// Whether the palette is currently being dragged
    pub palette_dragging: bool,
    /// Offset from palette position to mouse when drag started
    pub palette_drag_offset: Vec2,
    /// Pan tool state: drag start screen position
    pub pan_drag_start_screen: Option<Vec2>,
    /// Pan tool state: drag start camera origin
    pub pan_drag_start_origin: Option<Vec2>,
    /// Selection system state
    pub selection: SelectionState,
    /// Last painted cell coordinates for stroke interpolation
    pub last_painted_cell: Option<(i32, i32)>,
    /// Clipboard for copy/cut/paste operations
    pub clipboard: Clipboard,
    /// Undo/redo history
    pub history: History,
    /// Current palette mode (Basic or Extended)
    pub palette_mode: PaletteMode,
    /// Current palette page index
    pub palette_page: usize,
}

impl ApplicationState {
    /// Creates a new ApplicationState with default values
    pub fn new() -> Self {
        ApplicationState {
            mode: Mode::Paint,
            show_palette: false,
            current_color: BLUE,
            cells: CellGrid::new(),
            camera: AppCamera::new(),
            palette_position: Vec2::new(10.0, 50.0),
            palette_dragging: false,
            palette_drag_offset: Vec2::ZERO,
            pan_drag_start_screen: None,
            pan_drag_start_origin: None,
            selection: SelectionState::new(),
            last_painted_cell: None,
            clipboard: Clipboard::empty(),
            history: History::new(50),
            palette_mode: PaletteMode::Basic,
            palette_page: 0,
        }
    }
}
