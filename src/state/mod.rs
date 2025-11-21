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
        }
    }
}
