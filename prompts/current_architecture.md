# Current Architecture

This document explains the architecture of `tiny-neo-space` as implemented in the `src/` tree. It is designed so that a junior engineer can navigate the codebase and implement new features with minimal guesswork.

The application is a Macroquad-based pixel canvas editor with modes (paint, erase, pan, select), a chunked canvas renderer, a grid renderer with LOD, a selection system with lift/move/preview, a GBA-inspired palette window, and a simple undo stack.

Sections include file paths and precise line references. Snippets show the exact code lines related to each concept.

## Runtime Entrypoints

- Desktop entry: `src/main.rs:8-11`

```rust
// src/main.rs:8-11
#[macroquad::main("tiny-neo-space")]
async fn main() {
    app::run().await;
}
```

- WASM entry: `src/lib.rs:11-16`

```rust
// src/lib.rs:11-16
#[wasm_bindgen(start)]
pub fn start() {
    spawn_local(async {
        app::run().await;
    });
}
```

Both entries delegate to `app::run()`, which hosts the frame loop.

## Frame Loop and Layering

The main frame loop orchestrates state updates, input, and multi-layer rendering.

- `src/app/mod.rs:10-64` shows the full loop and draw order:

```rust
// src/app/mod.rs:10-64
pub async fn run() {
    let mut state = ApplicationState::new();
    let mut hud = Hud::new();
    let mut grid_renderer = GridRenderer::new();
    let mut canvas_renderer = CanvasRenderer::new();

    loop {
        let dt = get_frame_time();
        hud.update(dt);

        // White background
        clear_background(WHITE);

        // LAYER 1: Grid (behind everything except canvas)
        grid_renderer.update_if_needed();
        grid_renderer.draw(&state.camera);

        // LAYER 2: Canvas
        canvas_renderer.update_if_screen_resized();
        canvas_renderer.update(&state.cells);
        canvas_renderer.draw(&state.cells, &state.camera);

        // LAYER 3: Selection overlay
        draw_selection_overlay(&state);

        // Check if mouse is over UI
        let over_buttons = render_ui_buttons(&mut state);
        let over_palette = render_palette_window(&mut state);
        let over_ui = over_buttons || over_palette;

        // Handle zoom (scroll wheel) - only if not over UI
        if !over_ui {
            handle_zoom(&mut state);
        }

        // Handle user input (painting/erasing/panning) - only if not over UI
        if !over_ui {
            handle_input(&mut state, &mut canvas_renderer);
        }

        // LAYER 4: Cursor (only if not over UI)
        if !over_ui {
            let screen_mouse_pos = Vec2::from(mouse_position());
            draw_cursor_based_on_mode(&state.mode, &state.camera, screen_mouse_pos);
        }

        // LAYER 5: Selection action bar (on top of everything)
        draw_selection_action_bar(&mut state, &mut canvas_renderer);

        // LAYER 6: HUD (with camera info)
        hud.draw(&state.camera);

        next_frame().await
    }
}
```

Notes:
- Input is disabled while cursor is over UI to avoid conflicts.
- Rendering order ensures: grid → canvas → selection overlay → cursor → action bar → HUD.

## Global Application State

All UI, tool, camera, selection, clipboard and history state lives in `ApplicationState`.

- Modes: `src/state/mod.rs:15-26`

```rust
// src/state/mod.rs:15-26
#[derive(PartialEq, Clone)]
pub enum Mode {
    Paint,
    Erase,
    Pan,
    Select,
}
```

- State structure: `src/state/mod.rs:92-130`

```rust
// src/state/mod.rs:92-130
pub struct ApplicationState {
    pub mode: Mode,
    pub show_palette: bool,
    pub current_color: Color,
    pub cells: CellGrid,
    pub camera: AppCamera,
    pub palette_position: Vec2,
    pub palette_dragging: bool,
    pub palette_drag_offset: Vec2,
    pub pan_drag_start_screen: Option<Vec2>,
    pub pan_drag_start_origin: Option<Vec2>,
    pub temp_pan_active: bool,
    pub temp_pan_previous_mode: Option<Mode>,
    pub selection: SelectionState,
    pub last_painted_cell: Option<(i32, i32)>,
    pub clipboard: Clipboard,
    pub history: History,
    pub palette_mode: PaletteMode,
    pub palette_page: usize,
}
```

- Defaults: `src/state/mod.rs:132-155`

```rust
// src/state/mod.rs:132-155
impl ApplicationState {
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
            temp_pan_active: false,
            temp_pan_previous_mode: None,
            selection: SelectionState::new(),
            last_painted_cell: None,
            clipboard: Clipboard::empty(),
            history: History::new(50),
            palette_mode: PaletteMode::Basic,
            palette_page: 0,
        }
    }
}
```

### Clipboard and History

- Clipboard data is relative to the top-left of the selection rect.
- History is a simple stack of commands with `CellChange` entries.
  - Types: `src/state/mod.rs:47-63`
  - Stack ops: `src/state/mod.rs:65-83`

## Core Types and Coordinates

### Camera and world <-> screen mapping

- `src/core/camera.rs` defines a world-in-cells coordinate system, a zoom, and helpers:

```rust
// src/core/camera.rs:24-39
pub fn pixel_scale(&self) -> f32 { BASE_CELL_PIXELS * self.zoom }
pub fn cell_to_screen(&self, cell: (i32, i32)) -> Vec2 {
    let cell_world = Vec2::new(cell.0 as f32, cell.1 as f32);
    (cell_world - self.origin) * self.pixel_scale()
}
pub fn screen_to_cell(&self, screen: Vec2) -> Vec2 {
    (screen / self.pixel_scale()) + self.origin
}
```

- Zoom keeping cursor anchored: `src/core/camera.rs:56-69`

```rust
// src/core/camera.rs:56-69
pub fn zoom_around_cursor(&mut self, cursor_screen: Vec2, zoom_factor: f32) {
    let world_before = self.screen_to_cell(cursor_screen);
    self.zoom *= zoom_factor;
    self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    let world_after = self.screen_to_cell(cursor_screen);
    self.origin += world_before - world_after;
}
```

### Cell and Grid

- `src/core/cell.rs:4-8,19-24,33` define a basic `Cell` and `CellGrid` map:

```rust
// src/core/cell.rs:4-8,19-24,33
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell { pub color: Color, pub is_filled: bool }
impl Cell { pub fn with_color(color: Color) -> Self { Cell { color, is_filled: true } } }
pub type CellGrid = HashMap<(i32, i32), Cell>;
```

### Selection model

`src/core/selection.rs` separates geometry/state from input and rendering:

```rust
// src/core/selection.rs:5-11
pub struct SelectionRect { pub min_x: i32, pub min_y: i32, pub max_x: i32, pub max_y: i32 }
// src/core/selection.rs:23-33
impl SelectionRect { pub fn contains(&self, x: i32, y: i32) -> bool { ... } }
// src/core/selection.rs:47-52
pub struct Selection { pub rect: SelectionRect, pub kind: SelectionKind, pub preview: Option<RenderTarget> }
// src/core/selection.rs:54-84
pub struct SelectionState { /* drag/move/lift/offsets */ }
```

## Rendering Subsystem

Rendering is split by concern: canvas (chunked), grid (LOD), selection overlay, cursor, and HUD.

### CanvasRenderer (chunked rendering with dirty rebuilds)

- Chunks and dirty-flag: `src/rendering/canvas.rs:11-21`
- Mark dirty on cell change: `src/rendering/canvas.rs:60-67`
- Rebuild dirty chunks: `src/rendering/canvas.rs:74-86` calls `rebuild_chunk`
- Rebuild chunk texture: `src/rendering/canvas.rs:95-107,112-139`

```rust
// src/rendering/canvas.rs:145-206 (draw visible chunks)
pub fn draw(&self, _cells: &CellGrid, camera: &AppCamera) {
    let (min_x, min_y, max_x, max_y) = camera.visible_world_rect(screen_w, screen_h);
    let min_chunk_x = (min_x.floor() as i32).div_euclid(CHUNK_SIZE);
    // ...
    for chunk_x in min_chunk_x..=max_chunk_x {
        for chunk_y in min_chunk_y..=max_chunk_y {
            if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_y)) {
                let chunk_world_x = chunk_x * CHUNK_SIZE;
                let screen_pos = camera.cell_to_screen((chunk_world_x, chunk_world_y));
                let chunk_size_px = CHUNK_SIZE as f32 * camera.pixel_scale();
                draw_texture_ex(&chunk.render_target.texture, screen_pos.x, screen_pos.y, WHITE,
                    DrawTextureParams { dest_size: Some(vec2(chunk_size_px, chunk_size_px)), ..Default::default() });
            }
        }
    }
}
```

Implementation details:
- Each chunk is a `RenderTarget` of fixed pixel size (8 px per cell).
- Screen draw uses camera to compute screen position/size; optional rounding hints are present to avoid seams at fractional zoom.

### GridRenderer (LOD grid with alpha blend across zoom)

`src/rendering/grid.rs` computes a power-of-two step based on zoom with alpha blending between steps.

```rust
// src/rendering/grid.rs:4-12
fn compute_lod(zoom: f32) -> (i32, f32) {
    let step_smooth = 1.0 / zoom;
    let lod_f = step_smooth.log2();
    let lod = lod_f.floor().max(0.0) as i32;
    let blend = (lod_f - lod as f32).clamp(0.0, 1.0);
    let step = 1 << lod;
    (step, blend)
}
```

The draw function (lines `31-116`) renders vertical/horizontal lines at the current step with emphasis for tile boundaries every 16 cells and smooth fading when transitioning LOD.

### Selection overlay and action bar

`src/rendering/selection.rs` draws the selection during drag, after commit, and during move:

```rust
// src/rendering/selection.rs:9-17 (drag rectangle)
if state.selection.active_drag { if let (Some(start), Some(end)) = (...) {
    draw_selection_rect(camera, start, end, Color::new(0.3, 0.6, 1.0, 0.15), 2.0);
}}

// src/rendering/selection.rs:19-35 (highlight selected cells when not moving)
if let SelectionKind::Cells(cell_set) = &sel.kind { for &(x, y) in cell_set { ... } }

// src/rendering/selection.rs:37-50 (bounding rect)
draw_rectangle_lines(min_screen.x, min_screen.y, w, h, 1.0, ...);

// src/rendering/selection.rs:52-83 (move preview + yellow target outline)
if state.selection.is_moving { if let Some(preview) = &sel.preview { draw_texture_ex(...); } }
```

An action bar with a Delete button is drawn below the selection rect: `src/rendering/selection.rs:111-141`.

### Cursor and HUD

- Mode-based cursor: `src/rendering/cursor.rs:6-35` draws different indicators for each mode.
- HUD with FPS/zoom/camera origin: `src/rendering/hud.rs:29-44`. Call site is in the frame loop.

## Input System

### Dispatcher and Hotkeys

`src/input/dispatcher.rs` centralizes input handling and routes to mode-specific handlers.

```rust
// src/input/dispatcher.rs:7-79
pub fn handle_input(state: &mut ApplicationState, canvas_renderer: &mut CanvasRenderer) {
    // Temp pan with MMB; restore previous mode on release
    // Clipboard: Ctrl/Cmd+C/X/V; Undo: Ctrl/Cmd+Z
    // Mode hotkeys: B (paint), E (erase), V (select) [no Ctrl], H/Space (pan)
    // Delete selected: Delete/Backspace
    let screen_mouse_pos = Vec2::from(mouse_position());
    let world_mouse_pos = state.camera.screen_to_cell(screen_mouse_pos);
    match state.mode {
        Mode::Paint => perform_drawing(state, &world_mouse_pos, false, canvas_renderer),
        Mode::Erase => perform_drawing(state, &world_mouse_pos, true, canvas_renderer),
        Mode::Pan => handle_pan_tool(state, screen_mouse_pos),
        Mode::Select => handle_select_tool(state, canvas_renderer),
    }
}
```

Helpers in the same file:
- Wheel zoom around cursor: `src/input/dispatcher.rs:166-175`
- Pan drag with LMB or MMB (temp pan): `src/input/dispatcher.rs:136-164`
- Simple undo: `src/input/dispatcher.rs:111-126` uses history stack.

There is also a general-purpose change applier + recorder intended for grouped edits: `src/input/dispatcher.rs:81-109`.

### Tools: Painting and Erasing

`src/input/tools.rs` contains stroke handling with Bresenham interpolation and a single `set_cell()` write-through abstraction that marks canvas chunks dirty.

```rust
// src/input/tools.rs:69-111
pub fn perform_drawing(state: &mut ApplicationState, mouse_world: &Vec2, is_erasing: bool, canvas_renderer: &mut CanvasRenderer) {
    let cell_coords = (mouse_world.x.floor() as i32, mouse_world.y.floor() as i32);
    if is_mouse_button_pressed(MouseButton::Left) { /* start stroke */ }
    else if is_mouse_button_down(MouseButton::Left) { /* interpolate Bresenham */ }
    else if is_mouse_button_released(MouseButton::Left) { state.last_painted_cell = None; }
}
```

### Selection Interaction (input-side)

`src/input/selection.rs` handles selection creation, additive selection (Shift), lift-and-move with live preview, and drop.

Key flows:
- Start drag or lift-and-move on press: `src/input/selection.rs:13-22`
- Update drag or move deltas: `src/input/selection.rs:24-37`
- Release finishes action:
  - Drop lifted cells: `src/input/selection.rs:39-43` → `drop_lifted`
  - Finalize dragged selection (tight, filled-only): `src/input/selection.rs:43-45` → `finalize_selection_drag_tight`
  - Shift-click to add cell: `src/input/selection.rs:45-65`

Movement uses a “lifted” phase that removes cells from the grid, stores them, shows a preview texture, and re-inserts on drop:
- Start move with lift: `src/input/selection.rs:128-165`
- Drop lifted cells: `src/input/selection.rs:167-198`
- Delete selection: `src/input/selection.rs:200-213`

### Clipboard Operations

`src/input/clipboard.rs` provides copy/cut/paste and selection-at-paste creation.

- Copy selection to relative coords: `src/input/clipboard.rs:7-31`
- Cut = copy then delete: `src/input/clipboard.rs:33-52`
- Paste at cursor + create selection: `src/input/clipboard.rs:54-92`

## UI: Toolbar and Palette Window

### Toolbar buttons

`src/input/ui.rs` draws a row of buttons and toggles mode/palette.
- Draw buttons and hit-test: `src/input/ui.rs:5-15`
- Render toolbar and return hover state: `src/input/ui.rs:17-44`

### Palette window

`src/ui/palette.rs` renders a draggable palette window with two modes:
- Basic (4×8 GBA swatches) and Extended (paged 343-color gamut).
- Drag handling: `src/ui/palette.rs:25-41`
- Window, toggles, and swatches: `src/ui/palette.rs:42-167` (Basic), `169-301` (Extended with paging at `223-294`)

Color model for palette lives in `src/core/color.rs` with helper conversions and constants.

## Selection Rendering (overlay)

Complementing input logic, overlay drawing occurs in `src/rendering/selection.rs`:
- Drag rect: `9-17`
- Cell-level highlight and bounding box: `19-50`
- Move preview texture and target outline: `52-83`
- Action bar with Delete: `111-141`

Preview textures are (re)built as needed via `build_selection_preview`: `src/rendering/selection.rs:165-213`.

## Data & Control Flow (per frame)

1) HUD updates FPS every second: `src/rendering/hud.rs:19-27`.
2) Grid draws using camera LOD: `src/rendering/grid.rs:31-116`.
3) Canvas updates dirty chunks and draws visible chunks: `src/rendering/canvas.rs:74-86`, `145-206`.
4) Selection overlay draws drag/selection/preview: `src/rendering/selection.rs:9-83`.
5) UI renders buttons and palette; UI hover disables input: `src/app/mod.rs:35-43,45-54`.
6) Input dispatches based on mode: `src/input/dispatcher.rs:73-78` and helpers.
7) Selection action bar rendered above: `src/app/mod.rs:56-58` → `rendering/selection.rs:111-141`.
8) HUD overlays FPS/zoom/camera: `src/app/mod.rs:59-61`.

## Hotkeys

- Modes: B = Paint, E = Erase, V = Select (unless Ctrl/Cmd held), H/Space = Pan
- Temporary pan (hold): Middle Mouse Button (MMB)
- Zoom: mouse wheel (cursor-anchored)
- Clipboard: Ctrl/Cmd+C (copy), Ctrl/Cmd+X (cut), Ctrl/Cmd+V (paste)
- Undo (basic): Ctrl/Cmd+Z
- Delete selection: Delete or Backspace

References: `src/input/dispatcher.rs:33-66`.

## Performance Considerations

- Canvas is chunked (64×64 cells per chunk) and rendered into `RenderTarget`s.
  - Only dirty chunks rebuild: `src/rendering/canvas.rs:74-86`.
  - Chunk draw is culled to visible region: `src/rendering/canvas.rs:145-206`.
- Grid uses zoom-based LOD with alpha blending to reduce line density: `src/rendering/grid.rs:4-12,31-116`.
- Selection preview renders to a small texture once and reuses it while moving: `src/rendering/selection.rs:165-213`.

## Extending the System: Guidelines

- To modify cells, prefer batching with `apply_changes_and_record` to integrate with undo: `src/input/dispatcher.rs:81-109`.
- When writing tool logic, keep input handling in `src/input/*` and rendering in `src/rendering/*`.
- Use `CanvasRenderer::mark_dirty` for any cell changes so the correct chunk is redrawn.
- Follow the layering in `app::run()` for new overlay or HUD elements.
- Respect camera conversions (`cell_to_screen`, `screen_to_cell`) and `pixel_scale()` for consistent coordinates.
- For new palette/toolbar UI, ensure input hover masks editing input similar to how the toolbar/palette do in `app::run()`.

## File Index and Roles

- Entrypoints: `src/main.rs`, `src/lib.rs`
- App loop: `src/app/mod.rs`
- State: `src/state/mod.rs` (modes, camera state, selection state, history, clipboard, palette)
- Core: `src/core/*` (camera, cell grid, selection types, color, constants)
- Rendering: `src/rendering/*` (canvas chunks, grid LOD, cursor, selection overlay/action bar, HUD)
- Input: `src/input/*` (dispatcher/hotkeys, tools - paint/erase, selection interactions, clipboard ops, toolbar UI)
- UI: `src/ui/palette.rs` (palette window)

## Notes and Observations

- Painting/erasing currently bypasses the history recorder; to make paint strokes undoable as a single action, collect `CellChange`s during a stroke and call `apply_changes_and_record` on release.
- Clipboard ops exist both in `input/clipboard.rs` and partially mirrored near the end of `input/dispatcher.rs` (helper usage) — keep future clipboard enhancements centralized in `input/clipboard.rs`.
- `core/selection.rs` is the single source of truth for selection geometry/state, while `input/selection.rs` handles interaction and `rendering/selection.rs` handles visuals.

