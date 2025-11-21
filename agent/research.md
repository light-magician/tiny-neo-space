# Implementation Research: Rust + Macroquad Pixel Editor

> Comprehensive analysis for transforming the current codebase to match agent/prompt.md specification

---

## Table of Contents

1. [Overview & Architecture](#1-overview--architecture)
2. [Camera System & Coordinate Transformations](#2-camera-system--coordinate-transformations)
3. [Tile System & Palette](#3-tile-system--palette)
4. [Selection System & Operations](#4-selection-system--operations)
5. [UI System & Macroquad Widgets](#5-ui-system--macroquad-widgets)
6. [Zoom-Aware Grid Rendering](#6-zoom-aware-grid-rendering)
7. [Implementation Roadmap](#7-implementation-roadmap)

---

## 1. Overview & Architecture

### Current State Analysis

**File Structure:**
```
src/
  main.rs                    # Macroquad entry point
  lib.rs                     # WASM entry point
  app/mod.rs                 # Main game loop
  core/
    mod.rs                   # Core exports
    cell.rs                  # Cell, CellGrid (HashMap<(i32,i32), Cell>)
    constants.rs             # GRID_SIZE = 10.0, grid_color()
  state/mod.rs               # ApplicationState, Mode enum
  rendering/
    mod.rs                   # Rendering exports
    canvas.rs                # CanvasRenderer with dirty tracking
    grid.rs                  # GridRenderer with render target
    cursor.rs                # draw_cursor_based_on_mode
    hud.rs                   # Hud (FPS display)
  input/
    mod.rs                   # Input exports
    tools.rs                 # perform_drawing (paint/erase)
    ui.rs                    # render_ui_buttons, custom palette
    dispatcher.rs            # handle_input routing
```

**Key Abstractions:**
- **CellGrid**: `HashMap<(i32, i32), Cell>` for sparse pixel storage
- **Cell**: `{ color: Color, is_filled: bool }`
- **ApplicationState**: Global state with mode, cells, camera_offset, palette state
- **CanvasRenderer**: Uses RenderTarget with dirty cell tracking
- **GridRenderer**: Infinite grid via modulo offset

**Current Capabilities:**
- ✅ Sparse pixel canvas with paint/erase
- ✅ Simple pan (camera_offset: Vec2)
- ✅ Grid rendering (fixed 10px spacing)
- ✅ Custom color palette (draggable window)
- ✅ Dirty cell optimization for canvas
- ❌ No zoom functionality
- ❌ No tile system
- ❌ No selection tool
- ❌ No macroquad::ui widgets

---

## 2. Camera System & Coordinate Transformations

### 2.1 Current State Problems

**Current Camera:**
```rust
// In ApplicationState:
pub camera_offset: Vec2  // Simple pan offset
```

**Coordinate Conversion:**
```rust
// In dispatcher.rs:
let world_mouse_pos = screen_mouse_pos + state.camera_offset;

// In cell.rs:
pub fn grid_position_to_cell_coords(pos: &Vec2, grid_size: f32) -> (i32, i32) {
    ((pos.x / grid_size).floor() as i32, (pos.y / grid_size).floor() as i32)
}
```

**Problems:**
1. No zoom capability—`GRID_SIZE` is hardcoded at 10.0 pixels
2. Implicit 1:1 relationship between world and screen pixels
3. Pan logic doesn't account for zoom scaling
4. Grid rendering uses fixed spacing regardless of view scale
5. Render targets have fixed resolution (incompatible with zoom)

### 2.2 Target Camera Architecture

**New Camera Struct:**
```rust
// src/core/camera.rs (NEW FILE)
use macroquad::prelude::*;

pub const BASE_CELL_PIXELS: f32 = 24.0;  // Changed from 10.0 to match spec
pub const MIN_ZOOM: f32 = 0.2;
pub const MAX_ZOOM: f32 = 4.0;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// World cell coordinates at screen position (0, 0)
    pub origin: Vec2,

    /// Zoom level where 1.0 = BASE_CELL_PIXELS per cell
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            origin: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Get current size in screen pixels of one world cell
    #[inline]
    pub fn pixel_scale(&self) -> f32 {
        BASE_CELL_PIXELS * self.zoom
    }

    /// Convert integer cell coordinates to screen pixels
    /// Example: cell (10, 5) with zoom 2.0 → screen (480, 240)
    pub fn cell_to_screen(&self, cell: (i32, i32)) -> Vec2 {
        let cell_world = Vec2::new(cell.0 as f32, cell.1 as f32);
        (cell_world - self.origin) * self.pixel_scale()
    }

    /// Convert screen pixels to world cell coordinates (float)
    /// Floor the result to get cell indices
    pub fn screen_to_cell(&self, screen: Vec2) -> Vec2 {
        (screen / self.pixel_scale()) + self.origin
    }

    /// Get the world-space rect of the visible canvas area
    /// Returns: (min_x, min_y, max_x, max_y) in cell units
    pub fn visible_world_rect(&self, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
        let scale = self.pixel_scale();
        let world_min_x = self.origin.x;
        let world_min_y = self.origin.y;
        let world_max_x = self.origin.x + screen_w / scale;
        let world_max_y = self.origin.y + screen_h / scale;
        (world_min_x, world_min_y, world_max_x, world_max_y)
    }

    /// Pan the camera by a delta in world cell units
    pub fn pan_by(&mut self, delta_world: Vec2) {
        self.origin += delta_world;
    }

    /// Zoom around a point on screen (Figma-style zoom)
    /// cursor_screen: mouse position in screen pixels
    /// zoom_factor: multiply current zoom by this (e.g., 1.1 or 0.9)
    pub fn zoom_around_cursor(&mut self, cursor_screen: Vec2, zoom_factor: f32) {
        // Get world position under cursor BEFORE zoom
        let world_before = self.screen_to_cell(cursor_screen);

        // Apply zoom and clamp to valid range
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        // Get world position under cursor AFTER zoom
        let world_after = self.screen_to_cell(cursor_screen);

        // Adjust origin so the world point under cursor stays fixed
        self.origin += world_before - world_after;
    }
}
```

### 2.3 Transformation Math

**Cell to Screen:**
```
screen_pos = (cell_coords - camera.origin) * pixel_scale
           = (cell_coords - camera.origin) * (BASE_CELL_PIXELS * zoom)

Example:
  Cell (10, 10), origin (0, 0), zoom 2.0, BASE_CELL_PIXELS 24.0
  → screen_pos = (10, 10) * 48.0 = (480, 480)
```

**Screen to Cell:**
```
cell_pos = screen_pos / pixel_scale + camera.origin
         = screen_pos / (BASE_CELL_PIXELS * zoom) + camera.origin

Example:
  Screen (480, 480), origin (0, 0), zoom 2.0
  → cell_pos = (480, 480) / 48.0 = (10.0, 10.0)
  → cell_indices = floor((10.0, 10.0)) = (10, 10)
```

**Pan with Zoom Correction:**
```rust
// On mouse down: record start state
drag_start_screen = mouse_position();
drag_start_origin = camera.origin;

// While dragging:
delta_screen = current_mouse - drag_start_screen;
delta_world = delta_screen / camera.pixel_scale();
camera.origin = drag_start_origin - delta_world;
```

**Zoom Around Cursor:**
```rust
// User scrolls wheel
let world_before = camera.screen_to_cell(cursor_screen);
camera.zoom *= zoom_factor;
camera.zoom = camera.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
let world_after = camera.screen_to_cell(cursor_screen);
camera.origin += world_before - world_after;
```

### 2.4 Integration Changes

**Update ApplicationState:**
```rust
// In src/state/mod.rs
use crate::core::camera::Camera;

pub struct ApplicationState {
    // CHANGE: Replace camera_offset with full Camera
    // pub camera_offset: Vec2,
    pub camera: Camera,

    // ADD: Pan tool state
    pub pan_drag_start_screen: Option<Vec2>,
    pub pan_drag_start_origin: Option<Vec2>,

    // ... rest of fields
}
```

**Update Mode enum:**
```rust
#[derive(PartialEq)]
pub enum Mode {
    Paint,
    Erase,
    Pan,  // NEW
}
```

**Update Input Handlers:**
```rust
// In src/input/dispatcher.rs
pub fn handle_input(state: &mut ApplicationState, canvas_renderer: &mut CanvasRenderer) {
    let screen_mouse_pos = Vec2::from(mouse_position());

    // CHANGED: Use camera.screen_to_cell instead of simple addition
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);
    let cell_coords = (world_mouse_pos.x.floor() as i32, world_mouse_pos.y.floor() as i32);

    match state.mode {
        Mode::Paint => perform_drawing(state, cell_coords, false, canvas_renderer),
        Mode::Erase => perform_drawing(state, cell_coords, true, canvas_renderer),
        Mode::Pan => handle_pan_tool(state, screen_mouse_pos),
    }
}

fn handle_pan_tool(state: &mut ApplicationState, screen_mouse: Vec2) {
    if is_mouse_button_pressed(MouseButton::Left) {
        state.pan_drag_start_screen = Some(screen_mouse);
        state.pan_drag_start_origin = Some(state.camera.origin);
    }

    if is_mouse_button_down(MouseButton::Left) {
        if let (Some(start_screen), Some(start_origin)) =
            (state.pan_drag_start_screen, state.pan_drag_start_origin)
        {
            let delta_screen = screen_mouse - start_screen;
            let delta_world = delta_screen / state.camera.pixel_scale();
            state.camera.origin = start_origin - delta_world;
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        state.pan_drag_start_screen = None;
        state.pan_drag_start_origin = None;
    }
}

pub fn handle_zoom(state: &mut ApplicationState) {
    let (_scroll_x, scroll_y) = mouse_wheel();

    if scroll_y != 0.0 {
        let cursor_screen = Vec2::from(mouse_position());
        let zoom_factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        state.camera.zoom_around_cursor(cursor_screen, zoom_factor);
    }
}
```

**Update Renderers:**
```rust
// GridRenderer.draw() signature:
pub fn draw(&self, camera: &Camera) { /* ... */ }

// CanvasRenderer.draw() signature:
pub fn draw(&self, cells: &CellGrid, camera: &Camera) {
    let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(
        screen_width(), screen_height()
    );
    let pixel_scale = camera.pixel_scale();

    for (coords, cell) in cells.iter() {
        // Frustum culling
        if coords.0 as f32 >= min_x && coords.0 as f32 <= max_x &&
           coords.1 as f32 >= min_y && coords.1 as f32 <= max_y
        {
            let screen_pos = camera.cell_to_screen(*coords);
            draw_rectangle(screen_pos.x, screen_pos.y, pixel_scale, pixel_scale, cell.color);
        }
    }
}
```

**Update Cursor Rendering:**
```rust
// In src/rendering/cursor.rs
pub fn draw_cursor_based_on_mode(mode: &Mode, camera: &Camera, screen_mouse: Vec2) {
    let world_mouse = camera.screen_to_cell(screen_mouse);
    let cell_coords = (world_mouse.x.floor() as i32, world_mouse.y.floor() as i32);
    let cell_screen_pos = camera.cell_to_screen(cell_coords);
    let cell_size = camera.pixel_scale();

    match mode {
        Mode::Paint => {
            draw_rectangle_lines(cell_screen_pos.x, cell_screen_pos.y,
                cell_size, cell_size, 2.0, Color::from_rgba(0, 0, 0, 150));
        }
        Mode::Erase => {
            draw_rectangle_lines(cell_screen_pos.x, cell_screen_pos.y,
                cell_size, cell_size, 2.0, Color::from_rgba(255, 100, 100, 200));
        }
        Mode::Pan => {
            draw_circle(screen_mouse.x, screen_mouse.y, 4.0, DARKGRAY);
        }
    }
}
```

**Main Loop Integration:**
```rust
// In src/app/mod.rs
pub async fn run() {
    let mut state = ApplicationState::new();

    loop {
        clear_background(WHITE);

        // 1. Handle input
        handle_input(&mut state, &mut canvas_renderer);

        // 2. Handle zoom (scroll wheel)
        handle_zoom(&mut state);

        // 3. Draw grid
        grid_renderer.draw(&state.camera);

        // 4. Draw canvas
        canvas_renderer.draw(&state.cells, &state.camera);

        // 5. Draw cursor
        let screen_mouse = Vec2::from(mouse_position());
        draw_cursor_based_on_mode(&state.mode, &state.camera, screen_mouse);

        // 6. UI
        render_ui_buttons(&mut state);

        next_frame().await;
    }
}
```

### 2.5 Critical Considerations

**Render Target Limitation:**
- Current `CanvasRenderer` uses fixed-resolution `RenderTarget`
- With zoom, cannot pre-render entire world to texture
- **Solution:** Render cells directly to screen each frame with frustum culling
- Use dirty cell tracking to optimize what needs redrawing

**Coordinate Space Summary:**

| Space | Unit | Origin | Direction | Usage |
|-------|------|--------|-----------|-------|
| **Screen** | Pixels | (0,0) top-left | X→, Y↓ | Mouse input, drawing |
| **World (float)** | Cells | Arbitrary (camera.origin) | X→, Y↓ | After screen_to_cell |
| **Cell indices** | Integer | Arbitrary | X→, Y↓ | HashMap keys |

**Macroquad Gotchas:**
- `RenderTarget` has Y-axis flipped (OpenGL convention)
- `FilterMode::Nearest` prevents anti-aliasing artifacts at fractional zoom
- Built-in `Camera2D` unsuitable for infinite canvas; use custom transforms

---

## 3. Tile System & Palette

### 3.1 Data Structure Design

**Core Types:**
```rust
// src/core/tile.rs (NEW FILE)
use macroquad::prelude::Color;
use std::collections::HashMap;

pub const TILE_SIZE: usize = 16;
pub type TileId = u32;
pub type TileGroupId = u32;

#[derive(Clone, Debug)]
pub struct Tile {
    pub id: TileId,
    pub name: String,
    /// Tile pixels: [row][col], None = transparent
    pub pixels: [[Option<Color>; TILE_SIZE]; TILE_SIZE],
}

impl Tile {
    pub fn empty(id: TileId, name: String) -> Self {
        Self {
            id,
            name,
            pixels: [[None; TILE_SIZE]; TILE_SIZE],
        }
    }
}

/// TilePalette: library of reusable tile definitions
pub struct TilePalette {
    tiles: HashMap<TileId, Tile>,
    next_id: TileId,
}

impl TilePalette {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_tile(&mut self, name: String) -> TileId {
        let id = self.next_id;
        self.next_id += 1;
        self.tiles.insert(id, Tile::empty(id, name));
        id
    }

    pub fn get(&self, id: TileId) -> Option<&Tile> {
        self.tiles.get(&id)
    }

    pub fn get_mut(&mut self, id: TileId) -> Option<&mut Tile> {
        self.tiles.get_mut(&id)
    }

    pub fn remove(&mut self, id: TileId) -> Option<Tile> {
        self.tiles.remove(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.values()
    }

    pub fn count(&self) -> usize {
        self.tiles.len()
    }
}

/// TileInstance: placement of a tile in the world
#[derive(Clone, Copy, Debug)]
pub struct TileInstance {
    pub tile_id: TileId,
    pub origin: (i32, i32),  // Top-left in world cell coords
}

/// TileGroup: composition of multiple tiles
#[derive(Clone, Debug)]
pub struct TileGroup {
    pub id: TileGroupId,
    pub name: String,
    pub tiles: Vec<TileInstance>,  // Relative to group origin
}

pub struct TileGroupLibrary {
    groups: HashMap<TileGroupId, TileGroup>,
    next_id: TileGroupId,
}

impl TileGroupLibrary {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_group(&mut self, name: String) -> TileGroupId {
        let id = self.next_id;
        self.next_id += 1;
        self.groups.insert(id, TileGroup {
            id,
            name,
            tiles: Vec::new(),
        });
        id
    }

    pub fn get_group(&self, id: TileGroupId) -> Option<&TileGroup> {
        self.groups.get(&id)
    }

    pub fn get_group_mut(&mut self, id: TileGroupId) -> Option<&mut TileGroup> {
        self.groups.get_mut(&id)
    }
}
```

### 3.2 Relationship to Existing Cell System

**Three Independent Layers:**

```
┌─────────────────────────────────────────────────────────┐
│                   ApplicationState                       │
├─────────────────────────────────────────────────────────┤
│ cells: CellGrid            ← Freehand painted pixels    │
│ tiles: TilePalette         ← Reusable 16×16 sprites     │
│ tile_instances: Vec<...>   ← Placed tiles in world      │
│ tile_groups: TileGroupLib  ← Tile compositions          │
└─────────────────────────────────────────────────────────┘
```

**Rendering Order:**
1. Grid (background)
2. Tile instances (stamped sprites)
3. Painted cells (freehand layer on top)
4. Selection overlays

**Memory Usage:**
- Per tile: ~2KB (16×16×8 bytes for Option<Color> + overhead)
- 1000 tiles = ~2MB (acceptable)
- 10,000 tiles = ~20MB (manageable)

### 3.3 Selection to Tile Conversion

**Algorithm:**
```rust
// src/core/tile.rs
#[derive(Clone, Copy, Debug)]
pub struct SelectionRect {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl SelectionRect {
    pub fn width(&self) -> i32 { self.max_x - self.min_x + 1 }
    pub fn height(&self) -> i32 { self.max_y - self.min_y + 1 }

    pub fn is_valid_tile_size(&self) -> bool {
        self.width() <= TILE_SIZE as i32 && self.height() <= TILE_SIZE as i32
    }
}

/// Convert selected cells to a tile
/// Uses closure to fetch cell colors from state
pub fn cells_to_tile<F>(
    palette: &mut TilePalette,
    selected_cells: &[(i32, i32)],
    selection_rect: &SelectionRect,
    tile_name: String,
    get_color: F,
) -> Result<TileId, String>
where
    F: Fn(i32, i32) -> Option<Color>,
{
    if !selection_rect.is_valid_tile_size() {
        return Err(format!(
            "Selection too large: {}×{} (max 16×16)",
            selection_rect.width(),
            selection_rect.height()
        ));
    }

    if selected_cells.is_empty() {
        return Err("No cells selected".to_string());
    }

    let tile_id = palette.create_tile(tile_name);
    let tile = palette.get_mut(tile_id).expect("just created");

    for &(cell_x, cell_y) in selected_cells {
        let tile_x = (cell_x - selection_rect.min_x) as usize;
        let tile_y = (cell_y - selection_rect.min_y) as usize;

        if tile_x < TILE_SIZE && tile_y < TILE_SIZE {
            tile.pixels[tile_y][tile_x] = get_color(cell_x, cell_y);
        }
    }

    Ok(tile_id)
}
```

**Usage Example:**
```rust
// In event handler for "Group to Tile" button:
pub fn handle_group_to_tile(state: &mut ApplicationState) {
    let Some(selection) = &state.selection.current else { return; };

    let selected_cells: Vec<(i32, i32)> = /* ... from selection ... */;

    let result = cells_to_tile(
        &mut state.tiles,
        &selected_cells,
        &selection.rect,
        format!("tile_{}", state.tiles.count()),
        |x, y| {
            state.cells.get(&(x, y))
                .filter(|cell| cell.is_filled)
                .map(|cell| cell.color)
        },
    );

    if let Ok(tile_id) = result {
        println!("Created tile {}", tile_id);
    }
}
```

### 3.4 Tile Rendering

```rust
// src/rendering/tile_renderer.rs (NEW FILE)
use macroquad::prelude::*;
use crate::core::tile::*;
use crate::core::camera::Camera;

pub struct TileRenderer {
    // Could add texture cache later for performance
}

impl TileRenderer {
    pub fn new() -> Self {
        Self {}
    }

    /// Draw a single tile at world coordinates
    pub fn draw_tile_at_world(
        &self,
        tile: &Tile,
        world_x: i32,
        world_y: i32,
        camera: &Camera,
    ) {
        let screen_origin = camera.cell_to_screen((world_x, world_y));
        let cell_px = camera.pixel_scale();

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                if let Some(color) = tile.pixels[y][x] {
                    let px = screen_origin.x + (x as f32 * cell_px);
                    let py = screen_origin.y + (y as f32 * cell_px);
                    draw_rectangle(px, py, cell_px, cell_px, color);
                }
            }
        }
    }

    /// Draw all tile instances in the world
    pub fn draw_all_tiles(&self, state: &ApplicationState) {
        for instance in &state.tile_instances {
            if let Some(tile) = state.tiles.get(instance.tile_id) {
                self.draw_tile_at_world(
                    tile,
                    instance.origin.0,
                    instance.origin.1,
                    &state.camera,
                );
            }
        }
    }
}
```

### 3.5 ApplicationState Integration

```rust
// In src/state/mod.rs
use crate::core::tile::{TilePalette, TileInstance, TileGroupLibrary};

pub struct ApplicationState {
    // Existing fields...
    pub mode: Mode,
    pub show_palette: bool,
    pub current_color: Color,
    pub cells: CellGrid,
    pub camera: Camera,

    // NEW: Tile system
    pub tiles: TilePalette,
    pub tile_instances: Vec<TileInstance>,
    pub tile_groups: TileGroupLibrary,

    // ... rest of fields
}

impl ApplicationState {
    pub fn new() -> Self {
        ApplicationState {
            // ... existing initialization ...
            tiles: TilePalette::new(),
            tile_instances: Vec::new(),
            tile_groups: TileGroupLibrary::new(),
        }
    }
}
```

---

## 4. Selection System & Operations

### 4.1 SelectionState Design

```rust
// src/core/selection.rs (NEW FILE)
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
    // Future: Tiles(Vec<TileInstanceId>), Groups(Vec<TileGroupId>)
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
```

### 4.2 Selection Tool Handler

```rust
// In src/input/dispatcher.rs (or new selection.rs)
pub fn handle_select_tool(state: &mut ApplicationState) {
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);
    let cell_coords = (world_mouse_pos.x.floor() as i32, world_mouse_pos.y.floor() as i32);

    // Mouse pressed: start drag or move
    if is_mouse_button_pressed(MouseButton::Left) {
        if state.selection.contains_point(cell_coords.0, cell_coords.1) {
            // Click inside selection → start move
            state.selection.start_move((world_mouse_pos.x, world_mouse_pos.y));
        } else {
            // Click outside → start new selection drag
            state.selection.start_drag(cell_coords);
        }
    }

    // During drag: update end point
    if state.selection.active_drag && is_mouse_button_down(MouseButton::Left) {
        state.selection.update_drag_end(cell_coords);
    }

    // During move: accumulate delta
    if state.selection.is_moving && is_mouse_button_down(MouseButton::Left) {
        if let Some((prev_x, prev_y)) = state.selection.last_move_mouse {
            let delta_x = world_mouse_pos.x - prev_x;
            let delta_y = world_mouse_pos.y - prev_y;
            state.selection.update_move(delta_x, delta_y);
        }
        state.selection.last_move_mouse = Some((world_mouse_pos.x, world_mouse_pos.y));
    }

    // Mouse released: finalize
    if is_mouse_button_released(MouseButton::Left) {
        if state.selection.is_moving {
            if let Some((offset_x, offset_y)) = state.selection.finalize_move() {
                apply_selection_move(state, offset_x, offset_y);
            }
        } else if state.selection.active_drag {
            state.selection.finalize_drag(&state.cells);
        }
    }
}

fn apply_selection_move(state: &mut ApplicationState, offset_x: i32, offset_y: i32) {
    if let Some(sel) = &state.selection.current {
        if let SelectionKind::Cells(coords) = &sel.kind {
            // Collect old cell data
            let mut cell_data = Vec::new();
            for &(x, y) in coords.iter() {
                let old_coord = (x - offset_x, y - offset_y);
                if let Some(cell) = state.cells.remove(&old_coord) {
                    cell_data.push(((x, y), cell));
                }
            }

            // Place at new coordinates
            for (new_coord, cell) in cell_data {
                state.cells.insert(new_coord, cell);
            }
        }
    }
}
```

### 4.3 Selection Rendering

```rust
// src/rendering/selection.rs (NEW FILE)
use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::core::camera::Camera;

pub fn draw_selection_overlay(state: &ApplicationState) {
    let camera = &state.camera;

    // Draw active drag rectangle (translucent)
    if state.selection.active_drag {
        if let (Some(start), Some(end)) = (state.selection.drag_start, state.selection.drag_end) {
            draw_selection_rect(camera, start, end, Color::new(0.3, 0.6, 1.0, 0.15), 2.0);
        }
    }

    // Draw finalized selection
    if let Some(sel) = &state.selection.current {
        let rect = &sel.rect;
        let min_screen = camera.cell_to_screen((rect.min_x, rect.min_y));
        let max_screen = camera.cell_to_screen((rect.max_x + 1, rect.max_y + 1));
        let w = max_screen.x - min_screen.x;
        let h = max_screen.y - min_screen.y;

        // Fill
        draw_rectangle(min_screen.x, min_screen.y, w, h, Color::new(0.3, 0.6, 1.0, 0.1));

        // Outline
        draw_rectangle_lines(min_screen.x, min_screen.y, w, h, 2.0,
            Color::new(0.5, 0.8, 1.0, 0.8));

        // During move: show preview offset
        if state.selection.is_moving {
            let pixel_scale = camera.pixel_scale();
            let offset_px_x = state.selection.move_offset_x * pixel_scale;
            let offset_px_y = state.selection.move_offset_y * pixel_scale;

            draw_rectangle_lines(
                min_screen.x + offset_px_x,
                min_screen.y + offset_px_y,
                w, h, 1.0,
                Color::new(1.0, 1.0, 0.3, 0.6), // Yellow preview
            );
        }
    }
}

fn draw_selection_rect(
    camera: &Camera,
    p1: (i32, i32),
    p2: (i32, i32),
    fill_color: Color,
    border_width: f32,
) {
    let min_x = p1.0.min(p2.0);
    let max_x = p1.0.max(p2.0);
    let min_y = p1.1.min(p2.1);
    let max_y = p1.1.max(p2.1);

    let min_screen = camera.cell_to_screen((min_x, min_y));
    let max_screen = camera.cell_to_screen((max_x + 1, max_y + 1));

    let w = max_screen.x - min_screen.x;
    let h = max_screen.y - min_screen.y;

    draw_rectangle(min_screen.x, min_screen.y, w, h, fill_color);
    draw_rectangle_lines(min_screen.x, min_screen.y, w, h, border_width,
        Color::new(fill_color.r, fill_color.g, fill_color.b, 0.9));
}
```

### 4.4 Selection Action Bar

```rust
// src/rendering/selection_bar.rs (NEW FILE)
use macroquad::prelude::*;
use crate::state::ApplicationState;

pub fn draw_selection_action_bar(state: &mut ApplicationState) {
    if let Some(sel) = &state.selection.current {
        let rect = &sel.rect;
        let camera = &state.camera;

        // Position bar below selection
        let min_screen = camera.cell_to_screen((rect.min_x, rect.min_y));
        let max_screen = camera.cell_to_screen((rect.max_x + 1, rect.max_y + 1));

        let bar_y = max_screen.y + 4.0;
        let bar_x = min_screen.x;
        let bar_width = max_screen.x - min_screen.x;
        let bar_height = 28.0;

        // Don't draw if off-screen
        if bar_y > screen_height() || bar_y + bar_height < 0.0 {
            return;
        }

        // Background
        draw_rectangle(bar_x, bar_y, bar_width, bar_height,
            Color::from_rgba(80, 80, 120, 200));
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.0, BLACK);

        // Delete button
        if draw_action_button("Delete", bar_x + 4.0, bar_y + 2.0, 60.0, 24.0) {
            delete_selection(state);
        }

        // Group to Tile button
        if draw_action_button("Group→Tile", bar_x + 68.0, bar_y + 2.0, 90.0, 24.0) {
            group_selection_to_tile(state);
        }
    }
}

fn draw_action_button(label: &str, x: f32, y: f32, w: f32, h: f32) -> bool {
    let mouse_pos = Vec2::from(mouse_position());
    let rect = Rect::new(x, y, w, h);
    let is_hovered = rect.contains(mouse_pos);

    let color = if is_hovered {
        Color::from_rgba(100, 100, 150, 255)
    } else {
        Color::from_rgba(70, 70, 110, 255)
    };

    draw_rectangle(x, y, w, h, color);
    draw_rectangle_lines(x, y, w, h, 1.0, BLACK);

    let text_size = measure_text(label, None, 14, 1.0);
    let text_x = x + (w - text_size.width) / 2.0;
    let text_y = y + (h + text_size.height) / 2.0;
    draw_text(label, text_x, text_y, 14.0, WHITE);

    is_mouse_button_pressed(MouseButton::Left) && is_hovered
}

fn delete_selection(state: &mut ApplicationState) {
    if let Some(sel) = &state.selection.current {
        if let crate::core::selection::SelectionKind::Cells(coords) = &sel.kind {
            for &coord in coords {
                state.cells.remove(&coord);
            }
        }
        state.selection.current = None;
    }
}

fn group_selection_to_tile(state: &mut ApplicationState) {
    // Implementation requires tile system (Section 3.3)
}
```

### 4.5 Integration into ApplicationState

```rust
// In src/state/mod.rs
use crate::core::selection::SelectionState;

#[derive(PartialEq)]
pub enum Mode {
    Paint,
    Erase,
    Select,  // NEW
    Pan,
}

pub struct ApplicationState {
    // ... existing fields ...
    pub selection: SelectionState,  // NEW
}
```

---

## 5. UI System & Macroquad Widgets

### 5.1 Current vs Target Approach

**Current UI (Custom):**
- Manual button drawing with `draw_rectangle` + `draw_text`
- Custom drag tracking (`palette_dragging`, `palette_drag_offset`)
- No UI state management library
- Direct color selection from vec of named colors

**Target UI (Macroquad::ui):**
- Use `root_ui()` for centralized UI context
- `widgets::Window` for draggable palette
- Automatic position tracking
- Mouse capture detection with `over_ui` flag
- Structured `GBA_PALETTE` constant

### 5.2 Color Palette Structure

```rust
// src/core/color.rs (NEW FILE)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to macroquad Color (f32 0.0-1.0)
    pub fn to_mq_color(self) -> macroquad::color::Color {
        macroquad::color::Color::from_rgba(self.r, self.g, self.b, self.a)
    }

    /// Convert from macroquad Color
    pub fn from_mq_color(c: macroquad::color::Color) -> Self {
        Self {
            r: (c.r * 255.0) as u8,
            g: (c.g * 255.0) as u8,
            b: (c.b * 255.0) as u8,
            a: (c.a * 255.0) as u8,
        }
    }
}

pub const GBA_PALETTE_ROWS: usize = 4;
pub const GBA_PALETTE_COLS: usize = 8;

pub const GBA_PALETTE: [[Rgba; GBA_PALETTE_COLS]; GBA_PALETTE_ROWS] = [
    [
        Rgba::rgb(15, 56, 15),    // dark green
        Rgba::rgb(48, 98, 48),    // mid green
        Rgba::rgb(139, 172, 15),  // yellow-green
        Rgba::rgb(155, 188, 15),  // bright yellow-green
        Rgba::rgb(62, 62, 116),   // dark blue
        Rgba::rgb(92, 92, 168),   // medium blue
        Rgba::rgb(123, 123, 213), // bright blue
        Rgba::rgb(198, 198, 198), // light gray
    ],
    [
        Rgba::rgb(247, 247, 247), // white
        Rgba::rgb(255, 188, 188), // light pink
        Rgba::rgb(255, 119, 119), // pink
        Rgba::rgb(255, 68, 68),   // hot pink/red
        Rgba::rgb(188, 63, 63),   // dark red
        Rgba::rgb(120, 0, 0),     // darker red
        Rgba::rgb(33, 30, 89),    // dark purple-blue
        Rgba::rgb(47, 50, 167),   // indigo
    ],
    [
        Rgba::rgb(0, 0, 0),       // black
        Rgba::rgb(34, 32, 52),    // very dark gray
        Rgba::rgb(69, 40, 60),    // dark brown
        Rgba::rgb(102, 57, 49),   // brown
        Rgba::rgb(143, 86, 59),   // tan
        Rgba::rgb(223, 113, 38),  // orange
        Rgba::rgb(217, 160, 102), // light tan
        Rgba::rgb(238, 195, 154), // peach
    ],
    [
        Rgba::rgb(251, 242, 54),  // bright yellow
        Rgba::rgb(153, 229, 80),  // light green
        Rgba::rgb(106, 190, 48),  // medium green
        Rgba::rgb(55, 148, 110),  // teal-green
        Rgba::rgb(75, 105, 47),   // dark green
        Rgba::rgb(82, 75, 36),    // olive
        Rgba::rgb(50, 60, 57),    // dark teal
        Rgba::rgb(63, 63, 116),   // steel blue
    ],
];
```

### 5.3 Palette Window with Macroquad::ui

```rust
// src/ui/palette.rs (NEW FILE)
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use crate::core::color::{Rgba, GBA_PALETTE, GBA_PALETTE_ROWS, GBA_PALETTE_COLS};
use crate::state::ApplicationState;

pub fn render_palette_window(state: &mut ApplicationState) {
    if !state.show_palette {
        return;
    }

    let ui = &mut root_ui();

    // Stable window ID (critical for maintaining state)
    let window_id = hash!("color_palette_window");

    let window_size = vec2(220.0, 180.0);
    let window_pos = state.palette_position;

    widgets::Window::new(window_id, window_pos, window_size)
        .label("Color Palette")
        .titlebar(true)
        .movable(true)
        .ui(ui, |ui| {
            // Draw 4 rows × 8 columns of color buttons
            for row in 0..GBA_PALETTE_ROWS {
                ui.separator();

                ui.layout_horizontal(|ui| {
                    for col in 0..GBA_PALETTE_COLS {
                        let rgba = GBA_PALETTE[row][col];
                        let mq_color = rgba.to_mq_color();

                        // Create button (20×20 px)
                        let clicked = ui.button(Some(vec2(20.0, 20.0)), "");

                        // Overlay colored rectangle
                        if let Some(rect) = ui.last_widget() {
                            draw_rectangle(rect.x, rect.y, rect.w, rect.h, mq_color);

                            // Highlight current color
                            if colors_match(state.current_color, mq_color) {
                                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, YELLOW);
                            }
                        }

                        // Set color on click
                        if clicked {
                            state.current_color = mq_color;
                        }
                    }
                });
            }
        });

    // Query window position and check mouse capture
    if let Some(window_rect) = ui.window_rect(window_id) {
        state.palette_position = Vec2::new(window_rect.x, window_rect.y);

        let mouse_pos = Vec2::from(mouse_position());
        if mouse_pos.x >= window_rect.x
            && mouse_pos.x <= window_rect.x + window_rect.w
            && mouse_pos.y >= window_rect.y
            && mouse_pos.y <= window_rect.y + window_rect.h
        {
            state.input.mouse.over_ui = true;
        }
    }
}

fn colors_match(c1: Color, c2: Color) -> bool {
    (c1.r - c2.r).abs() < 0.01
        && (c1.g - c2.g).abs() < 0.01
        && (c1.b - c2.b).abs() < 0.01
        && (c1.a - c2.a).abs() < 0.01
}
```

### 5.4 Input State Extension

```rust
// In src/state/mod.rs
#[derive(Debug)]
pub struct MouseState {
    pub pos_screen: Vec2,
    pub over_ui: bool,  // NEW: set by UI rendering
}

pub struct InputState {
    pub mouse: MouseState,
    pub last_mouse_pos: Vec2,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse: MouseState {
                pos_screen: vec2(0.0, 0.0),
                over_ui: false,
            },
            last_mouse_pos: vec2(0.0, 0.0),
        }
    }

    pub fn update(&mut self) {
        let (x, y) = mouse_position();
        self.mouse.pos_screen = vec2(x, y);
        self.last_mouse_pos = self.mouse.pos_screen;
    }
}

pub struct ApplicationState {
    // ... existing fields ...
    pub input: InputState,  // NEW
}
```

### 5.5 Main Loop Order for Mouse Capture

```rust
// In src/app/mod.rs
pub async fn run() {
    let mut state = ApplicationState::new();

    loop {
        clear_background(WHITE);

        // 1. Update input state
        state.input.update();

        // 2. UI RENDERING (sets over_ui flag)
        state.input.mouse.over_ui = false;  // Reset each frame
        render_ui_buttons(&mut state);
        render_palette_window(&mut state);  // Sets over_ui if inside

        // 3. TOOLS (check over_ui before canvas input)
        if !state.input.mouse.over_ui {
            handle_input(&mut state, &mut canvas_renderer);
        }

        // 4. Rendering: grid, canvas, overlays
        grid_renderer.draw(&state.camera);
        canvas_renderer.draw(&state.cells, &state.camera);

        next_frame().await;
    }
}
```

### 5.6 Key Macroquad::ui Gotchas

| Issue | Impact | Workaround |
|-------|--------|-----------|
| **Global UI state** | Only one `root_ui()` per frame | Use consistent hash IDs |
| **Window position** | Stored internally | Query with `ui.window_rect()` after rendering |
| **No native color buttons** | Can't tint buttons | Draw rectangle after button with `last_widget()` |
| **Manual mouse capture** | UI doesn't auto-block input | Check window rect bounds, set `over_ui` flag |
| **Hash ID stability** | Must be same each frame | Use `hash!("literal_string")` |

---

## 6. Zoom-Aware Grid Rendering

### 6.1 Current Grid Problems

**Current GridRenderer:**
- Fixed `GRID_SIZE = 10.0` pixels
- Draws ALL grid lines every frame
- No adaptation to zoom level
- ~384 lines at 1920×1080 (constant)

**At extreme zoom (0.1x):**
- Still renders 384 lines
- Grid lines 1 pixel apart (visual noise)
- No performance optimization
- No tile boundary emphasis

### 6.2 Grid Density Algorithm

```rust
// In grid rendering module
pub fn grid_step_cells(cell_px: f32) -> i32 {
    match cell_px {
        cp if cp >= 16.0 => 1,   // Show every cell
        cp if cp >= 8.0 => 2,    // Every 2nd cell
        cp if cp >= 4.0 => 4,    // Every 4th cell
        cp if cp >= 2.0 => 8,    // Every 8th cell
        _ => 16,                  // Only tile boundaries
    }
}

pub fn get_grid_appearance(step_cells: i32) -> (Color, f32) {
    match step_cells {
        1 => (Color::new(0.80, 0.85, 0.95, 0.30), 0.5),    // Fine: light, thin
        2 => (Color::new(0.75, 0.80, 0.90, 0.35), 0.6),
        4 => (Color::new(0.70, 0.75, 0.85, 0.40), 0.7),
        8 => (Color::new(0.65, 0.70, 0.80, 0.45), 0.8),
        _ => (Color::new(0.60, 0.65, 0.75, 0.50), 1.0),    // Tile: darker, thicker
    }
}
```

### 6.3 Updated draw_grid Implementation

```rust
// src/rendering/grid.rs (REFACTORED)
use macroquad::prelude::*;
use crate::core::camera::Camera;

pub struct GridRenderer {
    // Remove: rt, width, height (no render target needed)
}

impl GridRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, camera: &Camera) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(screen_w, screen_h);

        let cell_px = camera.pixel_scale();
        let step_cells = grid_step_cells(cell_px) as i32;

        // Align start/end to grid step
        let start_x = ((min_x.floor() as i32) / step_cells) * step_cells;
        let start_y = ((min_y.floor() as i32) / step_cells) * step_cells;
        let end_x = ((max_x.ceil() as i32) / step_cells + 1) * step_cells;
        let end_y = ((max_y.ceil() as i32) / step_cells + 1) * step_cells;

        let (line_color, line_thickness) = get_grid_appearance(step_cells);

        // Draw vertical lines
        let mut x = start_x;
        while x <= end_x {
            let p0 = camera.cell_to_screen((x, start_y));
            let p1 = camera.cell_to_screen((x, end_y));
            draw_line(p0.x, p0.y, p1.x, p1.y, line_thickness, line_color);
            x += step_cells;
        }

        // Draw horizontal lines
        let mut y = start_y;
        while y <= end_y {
            let p0 = camera.cell_to_screen((start_x, y));
            let p1 = camera.cell_to_screen((end_x, y));
            draw_line(p0.x, p0.y, p1.x, p1.y, line_thickness, line_color);
            y += step_cells;
        }

        // Draw tile boundaries (every 16 cells) if zoomed out
        if step_cells > 1 {
            self.draw_tile_boundaries(camera, start_x, end_x, start_y, end_y);
        }
    }

    fn draw_tile_boundaries(&self, camera: &Camera, min_x: i32, max_x: i32, min_y: i32, max_y: i32) {
        let tile_step = 16;
        let tile_color = Color::new(0.50, 0.60, 0.80, 0.60);
        let tile_thickness = 1.2;

        let tile_start_x = (min_x / tile_step) * tile_step;
        let tile_start_y = (min_y / tile_step) * tile_step;
        let tile_end_x = ((max_x / tile_step) + 1) * tile_step;
        let tile_end_y = ((max_y / tile_step) + 1) * tile_step;

        // Vertical tile boundaries
        let mut x = tile_start_x;
        while x <= tile_end_x {
            let p0 = camera.cell_to_screen((x, tile_start_y));
            let p1 = camera.cell_to_screen((x, tile_end_y));
            draw_line(p0.x, p0.y, p1.x, p1.y, tile_thickness, tile_color);
            x += tile_step;
        }

        // Horizontal tile boundaries
        let mut y = tile_start_y;
        while y <= tile_end_y {
            let p0 = camera.cell_to_screen((tile_start_x, y));
            let p1 = camera.cell_to_screen((tile_end_x, y));
            draw_line(p0.x, p0.y, p1.x, p1.y, tile_thickness, tile_color);
            y += tile_step;
        }
    }
}

fn grid_step_cells(cell_px: f32) -> i32 {
    if cell_px >= 16.0 { 1 }
    else if cell_px >= 8.0 { 2 }
    else if cell_px >= 4.0 { 4 }
    else if cell_px >= 2.0 { 8 }
    else { 16 }
}

fn get_grid_appearance(step_cells: i32) -> (Color, f32) {
    match step_cells {
        1 => (Color::new(0.80, 0.85, 0.95, 0.30), 0.5),
        2 => (Color::new(0.75, 0.80, 0.90, 0.35), 0.6),
        4 => (Color::new(0.70, 0.75, 0.85, 0.40), 0.7),
        8 => (Color::new(0.65, 0.70, 0.80, 0.45), 0.8),
        _ => (Color::new(0.60, 0.65, 0.75, 0.50), 1.0),
    }
}
```

### 6.4 Performance Comparison

Assuming 1920×1080 screen, BASE_CELL_PIXELS=24.0:

| Zoom | cell_px | step | Lines/Frame | Reduction | Visual |
|------|---------|------|-------------|-----------|--------|
| 4.0 | 96px | 1 | ~96 | — | Dense grid |
| 2.0 | 48px | 1 | ~192 | — | Crisp grid |
| 1.0 | 24px | 1 | ~384 | — | Normal |
| 0.5 | 12px | 2 | ~96 | 75% | Every 2nd line |
| 0.25 | 6px | 4 | ~48 | 87.5% | Every 4th line |
| 0.1 | 2.4px | 16 | ~24 | 93.75% | Only tiles |

**Performance Win:** 16× fewer lines at extreme zoom-out while maintaining visual clarity.

---

## 7. Implementation Roadmap

### Phase 1: Camera System Foundation
**Goal:** Replace camera_offset with full Camera struct supporting zoom

**Tasks:**
1. Create `src/core/camera.rs` with Camera struct
2. Update `src/state/mod.rs`: replace `camera_offset` with `camera: Camera`
3. Add `Mode::Pan` variant
4. Implement `handle_pan_tool()` in dispatcher
5. Implement `handle_zoom()` for scroll wheel
6. Update `handle_input()` to use `camera.screen_to_cell()`
7. Update painting/erasing to use cell coordinates from camera transform
8. Test: pan and zoom should work smoothly, cells should stay under cursor during zoom

**Files to Create:**
- `src/core/camera.rs`

**Files to Modify:**
- `src/state/mod.rs`
- `src/input/dispatcher.rs`
- `src/input/tools.rs`
- `src/input/ui.rs` (add Pan button)

**Success Criteria:**
- ✅ Can pan with Pan tool
- ✅ Can zoom with scroll wheel
- ✅ Cursor-point stays fixed during zoom
- ✅ Painting works at any zoom level

---

### Phase 2: Grid Rendering Upgrade
**Goal:** Implement zoom-aware grid with adaptive density

**Tasks:**
1. Refactor `src/rendering/grid.rs`: remove render target
2. Implement `grid_step_cells()` function
3. Implement `get_grid_appearance()` for color/thickness
4. Update `draw()` to accept `&Camera`
5. Implement `visible_world_rect()` culling
6. Add `draw_tile_boundaries()` for 16-cell emphasis
7. Update `src/app/mod.rs` to pass camera to grid renderer
8. Test: grid should adapt smoothly as you zoom

**Files to Modify:**
- `src/rendering/grid.rs`
- `src/app/mod.rs`

**Success Criteria:**
- ✅ Grid density adapts to zoom (1, 2, 4, 8, 16 cell steps)
- ✅ Line color/alpha changes with density
- ✅ Tile boundaries (every 16 cells) visible at low zoom
- ✅ Performance: 16× fewer lines at zoom 0.1

---

### Phase 3: Canvas Rendering Refactor
**Goal:** Update CanvasRenderer for zoom support with frustum culling

**Tasks:**
1. Refactor `src/rendering/canvas.rs`: remove render target
2. Update `draw()` to accept `&Camera`
3. Implement frustum culling using `camera.visible_world_rect()`
4. Use `camera.cell_to_screen()` for cell positioning
5. Use `camera.pixel_scale()` for cell size
6. Update dirty cell tracking (optional: skip if rendering is fast)
7. Test: cells should render correctly at all zoom levels

**Files to Modify:**
- `src/rendering/canvas.rs`
- `src/app/mod.rs`

**Success Criteria:**
- ✅ Cells render with correct size at any zoom
- ✅ Only visible cells are drawn (frustum culling)
- ✅ No visual artifacts during zoom/pan

---

### Phase 4: Color Palette with Macroquad::ui
**Goal:** Replace custom palette with macroquad::ui widgets

**Tasks:**
1. Create `src/core/color.rs` with `Rgba` struct and `GBA_PALETTE`
2. Create `src/ui/palette.rs` with `render_palette_window()`
3. Update `src/state/mod.rs`: add `InputState` with `mouse.over_ui`
4. Update `src/input/ui.rs`: remove custom drag logic
5. Update `src/app/mod.rs`: set `over_ui = false`, call UI, check flag before tools
6. Test: palette should be draggable, clicking shouldn't paint underneath

**Files to Create:**
- `src/core/color.rs`
- `src/ui/palette.rs`

**Files to Modify:**
- `src/state/mod.rs`
- `src/input/ui.rs`
- `src/app/mod.rs`

**Success Criteria:**
- ✅ Palette window draggable via titlebar
- ✅ Color buttons clickable
- ✅ No painting when clicking palette (over_ui blocks tools)
- ✅ Window position persists

---

### Phase 5: Selection System
**Goal:** Implement drag selection, move, and delete

**Tasks:**
1. Create `src/core/selection.rs` with `SelectionState`, `SelectionRect`, `Selection`
2. Update `src/state/mod.rs`: add `Mode::Select` and `selection: SelectionState`
3. Create `src/input/selection.rs` with `handle_select_tool()`
4. Update `src/input/dispatcher.rs` to route `Mode::Select`
5. Create `src/rendering/selection.rs` with `draw_selection_overlay()`
6. Create `src/rendering/selection_bar.rs` with action buttons
7. Implement drag detection (threshold-based)
8. Implement click-inside detection for move
9. Implement move with float offset tracking
10. Implement delete selection
11. Test: drag to select, click inside to move, delete button works

**Files to Create:**
- `src/core/selection.rs`
- `src/input/selection.rs`
- `src/rendering/selection.rs`
- `src/rendering/selection_bar.rs`

**Files to Modify:**
- `src/state/mod.rs`
- `src/input/dispatcher.rs`
- `src/input/ui.rs` (add Select button)
- `src/app/mod.rs` (call selection rendering)

**Success Criteria:**
- ✅ Can drag to create selection
- ✅ Selection shows translucent rect
- ✅ Can click inside and move selection
- ✅ Move preview shown during drag
- ✅ Delete button removes selected cells
- ✅ Selection cleared when clicking outside

---

### Phase 6: Tile System
**Goal:** Implement tile palette and cell-to-tile conversion

**Tasks:**
1. Create `src/core/tile.rs` with `Tile`, `TilePalette`, `TileInstance`, `TileGroup`
2. Update `src/state/mod.rs`: add tile fields
3. Create `src/rendering/tile_renderer.rs`
4. Implement `cells_to_tile()` conversion function
5. Add "Group to Tile" handler in selection bar
6. Implement tile rendering in main loop
7. Test: select cells, group to tile, tile appears in palette

**Files to Create:**
- `src/core/tile.rs`
- `src/rendering/tile_renderer.rs`

**Files to Modify:**
- `src/state/mod.rs`
- `src/rendering/selection_bar.rs`
- `src/app/mod.rs`

**Success Criteria:**
- ✅ Can select cells and convert to tile
- ✅ Tile size validation (max 16×16)
- ✅ Tile stored in TilePalette
- ✅ Tile instances can be placed in world (basic stamping)

---

### Phase 7: Tile UI & Stamping
**Goal:** Add tile palette UI panel and tile stamp tool

**Tasks:**
1. Create `src/ui/tile_panel.rs` for tile library display
2. Add `Mode::TileStamp` with selected tile
3. Implement tile placement on click
4. Add tile preview at cursor
5. Test: select tile from palette, click to stamp

**Files to Create:**
- `src/ui/tile_panel.rs`

**Files to Modify:**
- `src/state/mod.rs`
- `src/input/dispatcher.rs`
- `src/app/mod.rs`

**Success Criteria:**
- ✅ Tile panel shows created tiles
- ✅ Can select tile from panel
- ✅ Can stamp tile into world
- ✅ Preview shown at cursor

---

### Phase 8: Polish & Testing
**Goal:** Refinements, edge case handling, performance tuning

**Tasks:**
1. Add keyboard shortcuts (P=Paint, E=Erase, S=Select, V=Pan)
2. Add visual feedback for active tool
3. Optimize dirty cell tracking
4. Test extreme zoom levels (0.1 to 4.0)
5. Test large canvas (10,000+ cells)
6. Add save/load (optional)
7. Documentation

**Success Criteria:**
- ✅ All tools work smoothly at any zoom
- ✅ No visual artifacts or glitches
- ✅ Performance acceptable (60 FPS with 10k cells)
- ✅ Code documented and organized

---

## Implementation Order Summary

```
Phase 1: Camera System (zoom + pan)
    ↓
Phase 2: Zoom-Aware Grid
    ↓
Phase 3: Canvas Rendering Refactor
    ↓
Phase 4: Macroquad::ui Palette
    ↓
Phase 5: Selection System
    ↓
Phase 6: Tile System
    ↓
Phase 7: Tile UI & Stamping
    ↓
Phase 8: Polish & Testing
```

**Estimated Complexity:**
- Phase 1-3: Foundation (2-3 days)
- Phase 4-5: UI & Selection (2-3 days)
- Phase 6-7: Tile System (3-4 days)
- Phase 8: Polish (1-2 days)

**Total:** ~10-15 days of focused development

---

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Camera** | Custom struct with origin + zoom | Simpler than macroquad Camera2D, better for infinite canvas |
| **Grid** | Direct screen rendering, adaptive step | Render targets incompatible with zoom, adaptive density = 16× perf win |
| **Canvas** | Direct rendering with frustum culling | Zoom makes fixed render target infeasible |
| **Tiles** | HashMap<TileId, Tile> with 16×16 fixed | O(1) lookup, manageable memory (~2MB/1000 tiles) |
| **Selection** | Float offset during drag, integer commit | Smooth visual feedback, clean grid alignment |
| **UI** | Macroquad::ui for palette only | Avoids full UI framework, keeps custom for canvas tools |
| **Color** | Rgba (u8) + GBA_PALETTE const | Compile-time palette, easy conversion to macroquad Color |
| **Coord System** | Cell indices (i32) for storage | Natural grid alignment, HashMap keys |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        ApplicationState                       │
├─────────────────────────────────────────────────────────────┤
│ mode: Mode                                                    │
│ camera: Camera                  ← Zoom + Pan transforms      │
│ cells: CellGrid                 ← Freehand pixels            │
│ tiles: TilePalette              ← Tile definitions           │
│ tile_instances: Vec<...>        ← Placed tiles               │
│ selection: SelectionState       ← Current selection          │
│ input: InputState               ← Mouse capture (over_ui)    │
└─────────────────────────────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    ┌─────┐   ┌─────────┐  ┌──────────┐
    │Input│   │Rendering│  │Core Data │
    └─────┘   └─────────┘  └──────────┘
        │           │            │
    ┌───┴───┐   ┌───┴────┐  ┌───┴────┐
    │Tools  │   │Grid    │  │Camera  │
    │UI     │   │Canvas  │  │Tile    │
    │Select │   │Cursor  │  │Color   │
    └───────┘   │Tile    │  │Cell    │
                │Selection│  └────────┘
                └─────────┘
```

---

## Conclusion

This research document provides:
- ✅ Complete data structure definitions with exact types and methods
- ✅ Transformation formulas with worked examples
- ✅ Integration points between modules
- ✅ Performance analysis and optimization strategies
- ✅ Step-by-step implementation roadmap
- ✅ Code examples for all major components
- ✅ Edge case handling and gotchas
- ✅ Testing criteria for each phase

The architecture is modular, testable, and follows Rust + Macroquad idioms. All abstractions are clearly defined and integrate cleanly with the existing codebase.
