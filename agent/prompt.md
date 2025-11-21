````markdown
# Spec for a Rust + Macroquad Pixel Editor Agent

## Scope: pixel canvas, tile palette, selection, groups, infinite canvas, grid, color palette popup

You are an LLM-based coding agent. Your task is to implement the **first slice** of a pixel-art world editor in **Rust** using **macroquad**.

We are **not** doing game runtime, NPCs, physics, or WASM yet. Only:

- Infinite 2D canvas with **pan + zoom**
- **Grid** that adapts to zoom level
- **Cells** (single pixels in world-space)
- **Tiles** (16×16 cells) and **tile groups**
- **Painting / erasing** cells
- **Select / move / delete / group** cells and tiles
- **Color palette popup** (with GBA-like colors) that:
  - opens as a draggable macroquad UI window,
  - captures mouse so we don’t draw underneath,
  - sets the current brush color when clicked.

You should build this in a **modular**, **testable**, and **macroquad-idiomatic** way.

---

## 1. Project layout

Use a single crate with clear module separation. Suggested structure:

```text
src/
  main.rs          // macroquad entrypoint and game loop
  app.rs           // top-level AppState orchestration
  model/
    mod.rs         // re-exports
    color.rs       // Rgba type + GBA palette
    cell.rs        // CellCoord, world cell storage
    tile.rs        // Tile, TilePalette, TileGroup
    selection.rs   // selection data structures
  canvas/
    mod.rs         // coord transforms, camera, visible rect
    grid.rs        // grid rendering based on zoom
    painting.rs    // painting / erasing cells
  ui/
    mod.rs         // top bar, tool buttons
    palette.rs     // color palette popup window
    selection_bar.rs // bar below selected region
  input.rs         // mouse + keyboard state, tool switching
  tools.rs         // Tool enum (Paint, Erase, Select, Pan)
  util.rs          // small helpers (rect, clamp, lerp)
```
````

Keep the **editing logic** in pure-Rust structs (no macroquad calls), and confine drawing to `canvas` and `ui` modules.

---

## 2. Core data model

### 2.1 Basic types: coordinates, rects, color

Use **world coordinates in cell units** as integers. Each grid cell in the world is one unit.

```rust
// src/model/cell.rs
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CellCoord {
    pub x: i32,
    pub y: i32,
}

impl CellCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl Hash for CellCoord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

// Axis-aligned rect in cell coordinates (inclusive bounds)
#[derive(Copy, Clone, Debug)]
pub struct CellRect {
    pub min: CellCoord,
    pub max: CellCoord,
}

impl CellRect {
    pub fn from_two_points(a: CellCoord, b: CellCoord) -> Self {
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = a.x.max(b.x);
        let max_y = a.y.max(b.y);
        Self {
            min: CellCoord::new(min_x, min_y),
            max: CellCoord::new(max_x, max_y),
        }
    }

    pub fn contains(&self, c: CellCoord) -> bool {
        c.x >= self.min.x && c.x <= self.max.x && c.y >= self.min.y && c.y <= self.max.y
    }

    pub fn width(&self) -> i32 {
        self.max.x - self.min.x + 1
    }

    pub fn height(&self) -> i32 {
        self.max.y - self.min.y + 1
    }
}
```

Color type and palette:

```rust
// src/model/color.rs
#[derive(Copy, Clone, Debug)]
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
}
```

Define a **GBA-like palette** as a small 2D array of `Rgba` (you can refine colors later; just pick some pleasant ones now):

```rust
// src/model/color.rs
pub const GBA_PALETTE_ROWS: usize = 4;
pub const GBA_PALETTE_COLS: usize = 8;

pub const GBA_PALETTE: [[Rgba; GBA_PALETTE_COLS]; GBA_PALETTE_ROWS] = [
    [
        Rgba::rgb(15, 56, 15),  // dark green
        Rgba::rgb(48, 98, 48),  // mid green
        Rgba::rgb(139, 172, 15),
        Rgba::rgb(155, 188, 15),
        Rgba::rgb(62, 62, 116),
        Rgba::rgb(92, 92, 168),
        Rgba::rgb(123, 123, 213),
        Rgba::rgb(198, 198, 198),
    ],
    [
        Rgba::rgb(247, 247, 247),
        Rgba::rgb(255, 188, 188),
        Rgba::rgb(255, 119, 119),
        Rgba::rgb(255, 68, 68),
        Rgba::rgb(188, 63, 63),
        Rgba::rgb(120, 0, 0),
        Rgba::rgb(33, 30, 89),
        Rgba::rgb(47, 50, 167),
    ],
    [
        Rgba::rgb(0, 0, 0),
        Rgba::rgb(34, 32, 52),
        Rgba::rgb(69, 40, 60),
        Rgba::rgb(102, 57, 49),
        Rgba::rgb(143, 86, 59),
        Rgba::rgb(223, 113, 38),
        Rgba::rgb(217, 160, 102),
        Rgba::rgb(238, 195, 154),
    ],
    [
        Rgba::rgb(251, 242, 54),
        Rgba::rgb(153, 229, 80),
        Rgba::rgb(106, 190, 48),
        Rgba::rgb(55, 148, 110),
        Rgba::rgb(75, 105, 47),
        Rgba::rgb(82, 75, 36),
        Rgba::rgb(50, 60, 57),
        Rgba::rgb(63, 63, 116),
    ],
];
```

When drawing, you will convert `Rgba` to `macroquad::color::Color`.

---

### 2.2 World cell storage

You need an “infinite” canvas storing only painted cells. Use a `HashMap<CellCoord, Rgba>`.

```rust
// src/model/cell.rs
use std::collections::HashMap;
use super::color::Rgba;

#[derive(Default)]
pub struct CanvasLayer {
    pub pixels: HashMap<CellCoord, Rgba>,
}

impl CanvasLayer {
    pub fn set(&mut self, coord: CellCoord, color: Rgba) {
        self.pixels.insert(coord, color);
    }

    pub fn erase(&mut self, coord: CellCoord) {
        self.pixels.remove(&coord);
    }

    pub fn get(&self, coord: CellCoord) -> Option<&Rgba> {
        self.pixels.get(&coord)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CellCoord, &Rgba)> {
        self.pixels.iter()
    }
}
```

---

### 2.3 Tiles and groups

A **Tile** is a reusable 16×16 sprite. It lives in a **TilePalette**. Later you’ll stamp tiles into the world, but for now, focus on defining them and grouping.

```rust
// src/model/tile.rs
use super::color::Rgba;

pub const TILE_SIZE: usize = 16;

pub type TileId = u32;
pub type TileGroupId = u32;

#[derive(Clone, Debug)]
pub struct Tile {
    pub id: TileId,
    pub name: String,
    // Some cells might be transparent (None) for partial sprites
    pub pixels: [[Option<Rgba>; TILE_SIZE]; TILE_SIZE],
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

#[derive(Default)]
pub struct TilePalette {
    pub tiles: Vec<Tile>,
    next_id: TileId,
}

impl TilePalette {
    pub fn new() -> Self {
        Self { tiles: Vec::new(), next_id: 1 }
    }

    pub fn create_tile(&mut self, name: &str) -> TileId {
        let id = self.next_id;
        self.next_id += 1;
        let tile = Tile::empty(id, name.to_string());
        self.tiles.push(tile);
        id
    }

    pub fn get_mut(&mut self, id: TileId) -> Option<&mut Tile> {
        self.tiles.iter_mut().find(|t| t.id == id)
    }

    pub fn get(&self, id: TileId) -> Option<&Tile> {
        self.tiles.iter().find(|t| t.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter()
    }
}
```

A **TileGroup** is like a Figma component made of multiple tiles (for later world building):

```rust
// src/model/tile.rs
use super::cell::CellCoord;

#[derive(Clone, Debug)]
pub struct TileInstance {
    pub tile_id: TileId,
    // Position in world *tile* coordinates (not used heavily yet)
    pub origin: CellCoord,
}

#[derive(Clone, Debug)]
pub struct TileGroup {
    pub id: TileGroupId,
    pub name: String,
    pub tiles: Vec<TileInstance>, // relative to group's local origin
}
```

---

### 2.4 Selection model

Selections can cover:

- Arbitrary **cells** (painted on the canvas)
- Later also tile instances and groups

You need to track both the **selection rectangle** and the **contents**.

```rust
// src/model/selection.rs
use super::cell::{CellCoord, CellRect};

#[derive(Clone, Debug)]
pub enum SelectionKind {
    Cells(Vec<CellCoord>),
    // Later: Tiles(Vec<TileInstanceId>), Groups(Vec<TileGroupId>), etc.
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub rect: CellRect,
    pub kind: SelectionKind,
}

#[derive(Default)]
pub struct SelectionState {
    pub active_drag: bool,
    pub drag_start: Option<CellCoord>,
    pub drag_end: Option<CellCoord>,
    // Current committed selection
    pub current: Option<Selection>,
    // During move-drag, track offset in world units (float for smoothness)
    pub move_offset: (f32, f32),
    pub is_moving: bool,
}
```

---

### 2.5 Project and global state

One top-level structure (`Project`) for core data, and `AppState` for transient UI/camera state.

```rust
// src/app.rs
use crate::model::{
    cell::CanvasLayer,
    tile::TilePalette,
    selection::SelectionState,
    color::Rgba,
};

pub struct Project {
    pub canvas: CanvasLayer,
    pub tiles: TilePalette,
}

pub struct Brush {
    pub color: Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Paint,
    Erase,
    Select,
    Pan, // hand tool
}

// Camera & zoom are in canvas module
use crate::canvas::camera::Camera;

pub struct UiState {
    pub palette_open: bool,
    pub palette_pos: (f32, f32),
}

pub struct AppState {
    pub project: Project,
    pub brush: Brush,
    pub active_tool: Tool,
    pub camera: Camera,
    pub selection: SelectionState,
    pub ui: UiState,
    pub input: crate::input::InputState,
}
```

Initialize defaults in an `impl AppState::new()`.

---

## 3. Camera and infinite canvas

You need a **2D camera** with **pan + zoom**, mapping between world cell coordinates and screen pixels. Infinite canvas implementations in HTML/WebGL use the same pattern: store camera position + zoom, and reproject points each frame. ([Sandro Maglione][1])

### 3.1 Camera definition

Decide:

- `camera.origin` = world-space coordinates of **screen (0,0)** in **cell units**.
- `zoom` is a float; base cell size is a constant in pixels.

```rust
// src/canvas/camera.rs
use macroquad::prelude::*;

pub const BASE_CELL_PIXELS: f32 = 24.0; // default size of one cell at zoom = 1.0

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub origin: Vec2, // world cell coords at screen (0,0)
    pub zoom: f32,    // 1.0 = base scale
}

impl Camera {
    pub fn new() -> Self {
        Self {
            origin: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }

    pub fn cell_to_screen(&self, cell: (i32, i32)) -> Vec2 {
        let world = Vec2::new(cell.0 as f32, cell.1 as f32);
        (world - self.origin) * self.pixel_scale()
    }

    pub fn screen_to_cell(&self, screen: Vec2) -> Vec2 {
        screen / self.pixel_scale() + self.origin
    }

    pub fn pixel_scale(&self) -> f32 {
        BASE_CELL_PIXELS * self.zoom
    }

    pub fn visible_world_rect(&self, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
        let scale = self.pixel_scale();
        let world_min_x = self.origin.x;
        let world_min_y = self.origin.y;
        let world_max_x = self.origin.x + screen_w / scale;
        let world_max_y = self.origin.y + screen_h / scale;
        (world_min_x, world_min_y, world_max_x, world_max_y)
    }
}
```

**Pan** (hand/drag tool):

- On mouse down, record `drag_start_screen` and `drag_start_origin`.
- While dragging, compute `delta_screen = current_screen - drag_start_screen`.
- Convert `delta_world = delta_screen / pixel_scale`.
- Set `camera.origin = drag_start_origin - delta_world`.

You get linear, zoom-correct pan.

**Zoom**:

- On scroll wheel:
  - Compute `world_under_cursor_before = screen_to_cell(mouse_pos)`.
  - Adjust `zoom` (clamp between e.g. 0.2 and 4.0).
  - Compute `world_under_cursor_after = screen_to_cell(mouse_pos)` with new zoom.
  - Adjust origin so that the world point under cursor stays fixed:
    `origin += world_under_cursor_before - world_under_cursor_after`.

This gives Figma-like zoom around cursor.

---

## 4. Input handling and mouse state

Macroquad exposes per-frame input query functions (`mouse_position`, `is_mouse_button_*`, `mouse_wheel`, keyboard checks, etc.). ([Docs.rs][2])

Wrap them into your own struct each frame:

```rust
// src/input.rs
use macroquad::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct MouseButtons {
    pub left_down: bool,
    pub left_pressed: bool,
    pub left_released: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct MouseState {
    pub pos_screen: Vec2,
    pub wheel_delta: f32,
    pub buttons: MouseButtons,
    pub over_ui: bool, // set by UI system
}

#[derive(Debug)]
pub struct InputState {
    pub mouse: MouseState,
    pub last_mouse_pos: Vec2,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse: MouseState {
                pos_screen: vec2(0.0, 0.0),
                wheel_delta: 0.0,
                buttons: MouseButtons {
                    left_down: false,
                    left_pressed: false,
                    left_released: false,
                },
                over_ui: false,
            },
            last_mouse_pos: vec2(0.0, 0.0),
        }
    }

    pub fn update(&mut self) {
        let (x, y) = mouse_position();
        let pos = vec2(x, y);
        let (scroll_x, scroll_y) = mouse_wheel();

        self.mouse.wheel_delta = scroll_y; // vertical scroll only
        self.mouse.pos_screen = pos;

        let left_down = is_mouse_button_down(MouseButton::Left);
        let left_pressed = is_mouse_button_pressed(MouseButton::Left);
        let left_released = is_mouse_button_released(MouseButton::Left);

        self.mouse.buttons = MouseButtons {
            left_down,
            left_pressed,
            left_released,
        };

        self.last_mouse_pos = pos;
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse.pos_screen - self.last_mouse_pos
    }
}
```

Workflow per frame:

1. `state.input.update()`
2. Build UI via `ui::render_ui(&mut state)` (this sets `state.input.mouse.over_ui = true` when inside popups).
3. If `!state.input.mouse.over_ui`, handle canvas tools.

---

## 5. Tools and modes

Define tools:

```rust
// src/tools.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Paint,
    Erase,
    Select,
    Pan,
}
```

Add UI buttons to switch tools (e.g. a toolbar at the top or left).

In the main loop, after input and UI:

```rust
fn update_tools(state: &mut AppState) {
    if state.input.mouse.over_ui {
        return; // ignore canvas if over palette or UI window
    }

    match state.active_tool {
        Tool::Paint => canvas::painting::handle_paint_tool(state),
        Tool::Erase => canvas::painting::handle_erase_tool(state),
        Tool::Select => canvas::selection::handle_select_tool(state),
        Tool::Pan   => canvas::camera::handle_pan_tool(state),
    }
}
```

Each handler reads:

- `state.input.mouse`
- `state.camera`
- writes to `state.project.canvas`, `state.selection`, or `state.camera`.

---

## 6. Painting & erasing

### 6.1 Painting tool

The painting tool:

- Converts screen mouse position → world cell coords.
- On left button down / pressed:
  - For each new cell the mouse has traversed since last frame, set the brush color in `CanvasLayer`.

Implement as:

```rust
// src/canvas/painting.rs
use macroquad::prelude::*;
use crate::{
    app::AppState,
    model::cell::CellCoord,
};

fn screen_to_cell(camera: &crate::canvas::camera::Camera, pos: Vec2) -> CellCoord {
    let world = camera.screen_to_cell(pos);
    CellCoord::new(world.x.floor() as i32, world.y.floor() as i32)
}

pub fn handle_paint_tool(state: &mut AppState) {
    let input = &state.input.mouse;
    if !input.buttons.left_down {
        return;
    }

    let cell = screen_to_cell(&state.camera, input.pos_screen);
    state.project.canvas.set(cell, state.brush.color);
}
```

For smoother strokes, you can interpolate between last and current cell and fill intermediate cells.

### 6.2 Erase tool

Same logic but calling `erase`:

```rust
pub fn handle_erase_tool(state: &mut AppState) {
    let input = &state.input.mouse;
    if !input.buttons.left_down {
        return;
    }

    let cell = screen_to_cell(&state.camera, input.pos_screen);
    state.project.canvas.erase(cell);
}
```

---

## 7. Selection mode

Selection mode should behave like Figma / design tools:

- Click-drag: shows translucent rectangle while dragging.
- On release: compute selection rectangle and populate `SelectionState.current`.
- Selected content:
  - For now: find all painted cells inside rect.

- Visual feedback:
  - Draw the selection rect (semi-transparent fill).
  - Draw bounding box outline around selected cells.
  - Draw a **toolbar** just under the bounding box with buttons:
    - Delete
    - Group Cells → Tile
    - (Later) Group Tiles

### 7.1 Detecting drag vs click

Implementation sketch:

```rust
// src/canvas/selection.rs
use macroquad::prelude::*;
use crate::{
    app::AppState,
    model::cell::{CellCoord, CellRect},
    model::selection::{SelectionState, Selection, SelectionKind},
};

fn screen_to_cell(camera: &crate::canvas::camera::Camera, pos: Vec2) -> CellCoord {
    let world = camera.screen_to_cell(pos);
    CellCoord::new(world.x.floor() as i32, world.y.floor() as i32)
}

pub fn handle_select_tool(state: &mut AppState) {
    let mouse = &state.input.mouse;

    // Start drag selection
    if mouse.buttons.left_pressed {
        let start = screen_to_cell(&state.camera, mouse.pos_screen);
        state.selection.active_drag = true;
        state.selection.drag_start = Some(start);
        state.selection.drag_end = Some(start);
        state.selection.is_moving = false;
    }

    // Update drag selection
    if state.selection.active_drag && mouse.buttons.left_down {
        let end = screen_to_cell(&state.camera, mouse.pos_screen);
        state.selection.drag_end = Some(end);
        // You could implement a threshold before committing as "selection rectangle"
    }

    // End drag selection
    if state.selection.active_drag && mouse.buttons.left_released {
        state.selection.active_drag = false;
        if let (Some(start), Some(end)) =
            (state.selection.drag_start, state.selection.drag_end)
        {
            let rect = CellRect::from_two_points(start, end);
            let selected_cells: Vec<CellCoord> = state
                .project
                .canvas
                .pixels
                .keys()
                .filter(|c| rect.contains(**c))
                .cloned()
                .collect();

            if !selected_cells.is_empty() {
                state.selection.current = Some(Selection {
                    rect,
                    kind: SelectionKind::Cells(selected_cells),
                });
            } else {
                state.selection.current = None;
            }
        }
    }

    // Clear selection when clicking outside without dragging
    if mouse.buttons.left_pressed && !state.selection.active_drag {
        if let Some(sel) = &state.selection.current {
            let clicked_cell = screen_to_cell(&state.camera, mouse.pos_screen);
            if !sel.rect.contains(clicked_cell) {
                state.selection.current = None;
            }
        }
    }

    // Move selection (click-drag on selection rectangle)
    // This is “move” rather than “new selection”
    // You can detect if mouse-down happens inside current rect and treat it differently
}
```

### 7.2 Moving selection

For move behavior:

- When a selection exists and the user `left_pressed` inside `selection.rect`, treat that as “start move”.
- Record `move_origin_cell` and `start_mouse_screen`.
- While dragging:
  - Compute `delta_screen = current_screen - start_mouse_screen`.
  - `delta_world = delta_screen / pixel_scale`.
  - Store `selection.move_offset = (delta_world.x, delta_world.y)`.

- On release:
  - Apply move: for `SelectionKind::Cells`, rebuild `canvas.pixels` with new cell coords = old + offset (rounded).

Implementation sketch:

```rust
pub fn handle_select_tool(state: &mut AppState) {
    let mouse = &state.input.mouse;

    // Movement start
    if mouse.buttons.left_pressed {
        if let Some(sel) = &state.selection.current {
            let clicked = screen_to_cell(&state.camera, mouse.pos_screen);
            if sel.rect.contains(clicked) {
                state.selection.is_moving = true;
                state.selection.move_offset = (0.0, 0.0);
                // store any other needed starting info (e.g. start rect) in SelectionState
                return;
            }
        }
    }

    // Movement ongoing
    if state.selection.is_moving && mouse.buttons.left_down {
        let scale = state.camera.pixel_scale();
        let delta_screen = state.input.mouse.pos_screen - state.input.last_mouse_pos;
        let dx_world = delta_screen.x / scale;
        let dy_world = delta_screen.y / scale;
        state.selection.move_offset.0 += dx_world;
        state.selection.move_offset.1 += dy_world;
    }

    // Movement end
    if state.selection.is_moving && mouse.buttons.left_released {
        commit_selection_move(state);
        state.selection.is_moving = false;
        state.selection.move_offset = (0.0, 0.0);
    }

    // If not moving, handle rectangle drag as above
    // (drag_start/drag_end + commit selection)
}

fn commit_selection_move(state: &mut AppState) {
    let offset_x = state.selection.move_offset.0.round() as i32;
    let offset_y = state.selection.move_offset.1.round() as i32;

    if offset_x == 0 && offset_y == 0 {
        return;
    }

    if let Some(sel) = &mut state.selection.current {
        match &mut sel.kind {
            SelectionKind::Cells(coords) => {
                // Remove old cells and reinsert
                let mut new_pixels = Vec::with_capacity(coords.len());
                for &old_coord in coords.iter() {
                    if let Some(color) = state.project.canvas.get(old_coord).cloned() {
                        state.project.canvas.erase(old_coord);
                        let new_coord = CellCoord::new(
                            old_coord.x + offset_x,
                            old_coord.y + offset_y,
                        );
                        state.project.canvas.set(new_coord, color);
                        new_pixels.push(new_coord);
                    }
                }
                coords.clear();
                coords.extend(new_pixels);

                sel.rect.min.x += offset_x;
                sel.rect.max.x += offset_x;
                sel.rect.min.y += offset_y;
                sel.rect.max.y += offset_y;
            }
        }
    }
}
```

### 7.3 Grouping selection into a Tile

When the toolbar “Group Cells → Tile” is clicked:

1. Find current `Selection`.

2. Ensure `rect.width() <= TILE_SIZE` and `rect.height() <= TILE_SIZE` (16).

3. Create new `Tile` in `TilePalette`.

4. Fill tile’s `pixels` from the selected world cells, mapping:

   ```text
   tile_x = cell.x - rect.min.x
   tile_y = cell.y - rect.min.y
   ```

5. Optional: delete the original cells or leave them.

```rust
pub fn group_selection_into_tile(state: &mut AppState, tile_name: &str) {
    let Some(sel) = &state.selection.current else { return; };

    let rect = sel.rect;
    if rect.width() > TILE_SIZE as i32 || rect.height() > TILE_SIZE as i32 {
        // for now, silently ignore or log error
        return;
    }

    let mut palette = &mut state.project.tiles;
    let tile_id = palette.create_tile(tile_name);
    let tile = palette.get_mut(tile_id).unwrap();

    if let SelectionKind::Cells(coords) = &sel.kind {
        for coord in coords {
            if let Some(color) = state.project.canvas.get(*coord) {
                let tx = (coord.x - rect.min.x) as usize;
                let ty = (coord.y - rect.min.y) as usize;
                tile.pixels[ty][tx] = Some(*color);
            }
        }
    }

    // Optionally: leave the canvas as-is or delete selected cells
}
```

---

## 8. Color palette popup (macroquad UI)

Use `macroquad::ui` to implement a draggable window with color buttons. Macroquad’s `root_ui` and `widgets::Window` provide a small immediate-mode GUI. ([Docs.rs][3])

### 8.1 UI state

You already have:

```rust
pub struct UiState {
    pub palette_open: bool,
    pub palette_pos: (f32, f32),
}
```

Initialize:

- `palette_open = false`
- `palette_pos = (50.0, 50.0)`

### 8.2 Opening / closing palette

On the main toolbar, add a “Color” button. When clicked:

- If `palette_open == false`, set `true`.
- If `true`, set `false`.

Example macroquad UI toolbar:

```rust
// src/ui/mod.rs
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use crate::app::{AppState, Tool};

pub fn render_top_bar(state: &mut AppState) {
    let ui = &mut root_ui();

    widgets::Window::new(hash!("top_bar"), vec2(0.0, 0.0), vec2(screen_width(), 30.0))
        .movable(false)
        .titlebar(false)
        .ui(ui, |ui| {
            ui.label(None, "Tools:");

            if ui.button(None, "Paint") {
                state.active_tool = Tool::Paint;
            }
            if ui.button(None, "Erase") {
                state.active_tool = Tool::Erase;
            }
            if ui.button(None, "Select") {
                state.active_tool = Tool::Select;
            }
            if ui.button(None, "Pan") {
                state.active_tool = Tool::Pan;
            }

            ui.separator();

            if ui.button(None, "Color") {
                state.ui.palette_open = !state.ui.palette_open;
            }
        });
}
```

### 8.3 Palette window implementation

Render this after the top bar but before canvas input, so you can mark mouse as “over UI” when inside the palette.

```rust
// src/ui/palette.rs
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use crate::app::AppState;
use crate::model::color::{GBA_PALETTE, GBA_PALETTE_ROWS, GBA_PALETTE_COLS, Rgba};

fn to_mq_color(c: Rgba) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

pub fn render_palette_window(state: &mut AppState) {
    if !state.ui.palette_open {
        return;
    }

    let ui = &mut root_ui();
    let pos = vec2(state.ui.palette_pos.0, state.ui.palette_pos.1);
    let size = vec2(220.0, 160.0);

    let window_id = hash!("color_palette");

    let response = widgets::Window::new(window_id, pos, size)
        .label("Palette")
        .titlebar(true)
        .movable(true)
        .ui(ui, |ui| {
            for row in 0..GBA_PALETTE_ROWS {
                ui.separator();
                ui.layout_horizontal(|ui| {
                    for col in 0..GBA_PALETTE_COLS {
                        let color = GBA_PALETTE[row][col];
                        let colormq = to_mq_color(color);

                        // Draw small colored button
                        let clicked = ui.button(vec2(20.0, 20.0), "");
                        if clicked {
                            state.brush.color = color;
                        }

                        // Overdraw the button rect with solid color
                        let rect = ui.last_widget();
                        if let Some(rect) = rect {
                            draw_rectangle(
                                rect.x,
                                rect.y,
                                rect.w,
                                rect.h,
                                colormq,
                            );
                        }
                    }
                });
            }
        });

    // Update palette position from window rect if needed
    if let Some(rect) = ui.window_rect(window_id) {
        state.ui.palette_pos = (rect.x, rect.y);
        // Mark mouse as over UI if inside
        let mouse = state.input.mouse.pos_screen;
        if mouse.x >= rect.x && mouse.x <= rect.x + rect.w &&
           mouse.y >= rect.y && mouse.y <= rect.y + rect.h {
            // Prevent canvas from drawing underneath
            state.input.mouse.over_ui = true;
        }
    }

    // NOTE: macroquad's widgets::Window doesn't have a built-in close button
    // by default; you can add a "Close" button inside the UI if desired.
}
```

If the actual macroquad API differs slightly, adjust, but the pattern is:

- Use `root_ui()`.
- Create a `Window` with a stable `hash!` id.
- Inside, create buttons for colors.
- Convert `Rgba` → `Color` and overlay the colored rect.
- Detect mouse inside window bounds and set `over_ui = true`.

---

## 9. Infinite grid rendering with zoom-aware density

You need a grid that:

- Aligns to world cell coordinates.
- Changes visual density as you zoom:
  - When zoomed in: all cell lines.
  - Zooming out: drop some lines (e.g. show only every 2nd, 4th, 16th) to avoid overdraw and noise.

- Looks roughly “similar” across zoom levels (grid lines appear consistent). This is how design tools like Figma/Miro do it. ([Infinite Canvas Tutorial][4])

### 9.1 Decide grid levels

Let:

- `cell_px = camera.pixel_scale()` = size of one world cell in screen pixels.

Define:

```rust
fn grid_step_cells(cell_px: f32) -> i32 {
    if cell_px >= 16.0 {
        1  // every cell line
    } else if cell_px >= 8.0 {
        2  // every 2 cells
    } else if cell_px >= 4.0 {
        4
    } else if cell_px >= 2.0 {
        8
    } else {
        16 // only tile-level grid
    }
}
```

As zoom changes, you smoothly move from fine grid to coarse grid.

### 9.2 Compute visible grid lines

Given `camera.visible_world_rect(screen_w, screen_h)`:

```rust
pub fn draw_grid(camera: &Camera) {
    let w = screen_width();
    let h = screen_height();
    let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(w, h);

    let cell_px = camera.pixel_scale();
    let step_cells = grid_step_cells(cell_px) as f32;

    // Convert to cell indices
    let start_x = (min_x.floor() as i32 / step_cells as i32 - 1) * step_cells as i32;
    let end_x = (max_x.ceil() as i32 / step_cells as i32 + 1) * step_cells as i32;
    let start_y = (min_y.floor() as i32 / step_cells as i32 - 1) * step_cells as i32;
    let end_y = (max_y.ceil() as i32 / step_cells as i32 + 1) * step_cells as i32;

    // Alpha based on step; fine grid is faint, coarse grid stronger
    let alpha = if step_cells <= 2.0 { 0.25 } else if step_cells <= 4.0 { 0.3 } else { 0.4 };
    let line_color = Color::new(0.3, 0.6, 1.0, alpha);

    // Draw vertical lines
    for x in (start_x..=end_x).step_by(step_cells as usize) {
        let p0 = camera.cell_to_screen((x, start_y));
        let p1 = camera.cell_to_screen((x, end_y));
        draw_line(p0.x, p0.y, p1.x, p1.y, 1.0, line_color);
    }

    // Draw horizontal lines
    for y in (start_y..=end_y).step_by(step_cells as usize) {
        let p0 = camera.cell_to_screen((start_x, y));
        let p1 = camera.cell_to_screen((end_x, y));
        draw_line(p0.x, p0.y, p1.x, p1.y, 1.0, line_color);
    }

    // Optional: emphasize tile boundaries (every 16 cells) with thicker lines
    let tile_step = 16;
    let tile_color = Color::new(0.2, 0.5, 0.9, alpha + 0.1);

    for x in (start_x..=end_x).step_by(tile_step) {
        let p0 = camera.cell_to_screen((x, start_y));
        let p1 = camera.cell_to_screen((x, end_y));
        draw_line(p0.x, p0.y, p1.x, p1.y, 1.5, tile_color);
    }

    for y in (start_y..=end_y).step_by(tile_step) {
        let p0 = camera.cell_to_screen((start_x, y));
        let p1 = camera.cell_to_screen((end_x, y));
        draw_line(p0.x, p0.y, p1.x, p1.y, 1.5, tile_color);
    }
}
```

This ensures:

- Only grid lines within visible world rect are drawn.
- As you zoom out, fewer lines.

---

## 10. Rendering cells and selection

### 10.1 Drawing painted cells

Render only those cells within the visible world rect. Iterate `canvas.pixels` and check if inside bounds.

```rust
// src/canvas/mod.rs
use macroquad::prelude::*;
use crate::{app::AppState, model::cell::CellCoord};

pub fn draw_canvas(state: &AppState) {
    let (min_x, min_y, max_x, max_y) = state
        .camera
        .visible_world_rect(screen_width(), screen_height());

    let mut cell_color = |coord: CellCoord, color: crate::model::color::Rgba| {
        if coord.x as f32 >= min_x && coord.x as f32 <= max_x &&
           coord.y as f32 >= min_y && coord.y as f32 <= max_y
        {
            let p = state.camera.cell_to_screen((coord.x, coord.y));
            let size = state.camera.pixel_scale(); // one cell

            let mq_color = Color::from_rgba(color.r, color.g, color.b, color.a);
            draw_rectangle(p.x, p.y, size, size, mq_color);
        }
    };

    for (coord, color) in state.project.canvas.iter() {
        cell_color(*coord, *color);
    }
}
```

### 10.2 Drawing selection visuals

- If `selection.active_drag`, draw translucent rectangle from `drag_start` to current mouse cell.
- If `selection.current`:
  - Draw a slightly thicker rectangle around `current.rect`.
  - Draw a semi-transparent fill inside.
  - Compute the screen rect for `current.rect` and render a small bar just below with actions.

```rust
pub fn draw_selection_overlay(state: &AppState) {
    // Active drag rect
    if state.selection.active_drag {
        if let (Some(start), Some(end)) =
            (state.selection.drag_start, state.selection.drag_end)
        {
            let rect = CellRect::from_two_points(start, end);
            draw_rect_world(&state.camera, rect, Color::new(0.2, 0.5, 1.0, 0.1), 1.0);
        }
    }

    // Current selection
    if let Some(sel) = &state.selection.current {
        draw_rect_world(
            &state.camera,
            sel.rect,
            Color::new(0.2, 0.5, 1.0, 0.15),
            1.5,
        );
        // Selection action bar below rect
        draw_selection_bar(state, &sel.rect);
    }
}

fn draw_rect_world(camera: &Camera, rect: CellRect, fill: Color, border_thickness: f32) {
    let min_screen = camera.cell_to_screen((rect.min.x, rect.min.y));
    let max_screen = camera.cell_to_screen((rect.max.x + 1, rect.max.y + 1)); // +1 to cover cell size
    let w = max_screen.x - min_screen.x;
    let h = max_screen.y - min_screen.y;

    // fill
    draw_rectangle(min_screen.x, min_screen.y, w, h, fill);
    // border
    let border_color = Color::new(fill.r, fill.g, fill.b, fill.a + 0.1);
    draw_rectangle_lines(min_screen.x, min_screen.y, w, h, border_thickness, border_color);
}
```

`draw_selection_bar` can be implemented in `ui::selection_bar` using macroquad UI, but anchored using the screen-space bounding box derived here.

---

## 11. Main loop wiring

`src/main.rs`:

```rust
use macroquad::prelude::*;
mod app;
mod model;
mod canvas;
mod ui;
mod input;
mod tools;
mod util;

use app::AppState;

#[macroquad::main("Pixel Editor")]
async fn main() {
    let mut state = AppState::new();

    loop {
        clear_background(BLACK);

        // 1. Update input
        state.input.update();

        // 2. Handle zoom (scroll)
        if state.input.mouse.wheel_delta != 0.0 {
            canvas::camera::handle_zoom(&mut state);
        }

        // 3. UI (top bar, palette, selection bar)
        ui::render_top_bar(&mut state);
        ui::palette::render_palette_window(&mut state);
        ui::selection_bar::render_selection_bar(&mut state);

        // 4. Tools logic (ignore if mouse over UI)
        if !state.input.mouse.over_ui {
            crate::tools::update_tools(&mut state);
        }

        // 5. Draw grid, then cells, then selection overlay
        canvas::grid::draw_grid(&state.camera);
        canvas::draw_canvas(&state);
        canvas::selection_overlay::draw_selection_overlay(&state);

        next_frame().await;
    }
}
```

The exact module paths may differ depending on how you split files, but the **order of operations** should be:

1. Input sampling
2. Camera zoom & pan update
3. UI → sets `mouse.over_ui`
4. Tool logic (painting, erasing, selection) only if not over UI
5. Rendering: grid → cells → overlays

---

## 12. Implementation guidelines for the agent

- **Prefer pure logic in model/canvas**:
  - Functions should accept `&mut AppState` or more specific structs and mutate them.
  - Only functions in `canvas`, `ui` and `main.rs` should call macroquad drawing APIs.

- **Don’t assume fixed window size**:
  - Always use `screen_width()` and `screen_height()` at runtime.

- **Be careful with world/screen conversions**:
  - All input starts in **screen space** (`Vec2`).
  - Convert to **world cell space** using `camera.screen_to_cell`.
  - Round down (`floor`) to get integer `CellCoord`.

- **When grouping cells into tiles**, enforce 16×16 max size for now:
  - If selection rectangle exceeds 16 in either dimension, do nothing or log a warning comment.

- **When moving selection**, apply integer rounding when committing:
  - Keep movement stored as `f32` deltas but round when finalizing cell positions.

- **Avoid unnecessary allocations** in tight loops:
  - For grid drawing, compute start/end indices once per frame.

- **Keep tool handlers small** and separated:
  - `handle_paint_tool`, `handle_erase_tool`, `handle_select_tool`, `handle_pan_tool`, `handle_zoom`.

---

## 13. References (for your internal reasoning only)

These concepts mirror patterns from infinite canvas and macroquad docs/tutorials:

- Infinite canvas zoom/pan and world → screen mapping: ([Sandro Maglione][1])
- Zoom-sensitive grid density (drop lines as zoom changes, Figma-style): ([Infinite Canvas Tutorial][4])
- Macroquad input handling (`mouse_position`, `mouse_wheel`, key/button APIs): ([Docs.rs][2])
- Macroquad UI windows/buttons via `root_ui` and `widgets::Window` / `Button`: ([Docs.rs][3])

Use these patterns as underlying logic; don’t copy HTML/JS specifics, but adapt the math and behavior to Rust+macroquad.

---

This document is meant to be **fed directly to an LLM agent**.
The agent’s job is to turn these instructions into concrete Rust code, filling in missing imports, wiring modules, and adjusting small API differences while preserving:

- The **core abstractions** (Camera, CanvasLayer, TilePalette, SelectionState, Tools)
- The **infinite canvas behavior** (pan + zoom, visible rect)
- The **zoom-aware grid**
- The **color palette popup** that blocks painting underneath
- The **select / move / group** cell behavior

```
::contentReference[oaicite:8]{index=8}
```

[1]: https://www.sandromaglione.com/articles/infinite-canvas-html-with-zoom-and-pan?utm_source=chatgpt.com 'Infinite HTML canvas with zoom and pan'
[2]: https://docs.rs/macroquad/latest/macroquad/input/index.html?utm_source=chatgpt.com 'macroquad::input - Rust'
[3]: https://docs.rs/macroquad/latest/macroquad/ui/index.html?utm_source=chatgpt.com 'macroquad::ui - Rust'
[4]: https://infinitecanvas.cc/guide/lesson-005.html?utm_source=chatgpt.com 'Lesson 5 - Grid | An infinite canvas tutorial'
