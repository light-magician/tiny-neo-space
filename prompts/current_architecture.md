# Current Architecture

Tiny Neo Space is a Rust + Macroquad pixel canvas editor. It runs as a desktop app and as WASM. The code is split into state, input, rendering, UI, and core types. This snapshot highlights responsibilities, data flow, and key file locations for quick onboarding.

## Runtime & Build

- Desktop entrypoint: src/main.rs:8-11 (Macroquad runner → app::run).
- WASM entrypoint: src/lib.rs:11-16 (#[wasm_bindgen(start)] → app::run).
- Dependencies: Macroquad 0.4, wasm-bindgen, wasm-bindgen-futures (Cargo.toml). Crate types: cdylib + rlib with a [[bin]] for desktop.

## Module Map

- app: src/app/mod.rs — Frame loop and draw order; orchestrates subsystems.
- state: src/state/mod.rs — Global ApplicationState, modes, clipboard, history, palette; re-exports SelectionState from core.
- core: src/core/* — camera, cell grid, selection geometry/state, color, group model.
- input: src/input/* — input dispatcher, paint/erase tool, selection interactions, clipboard ops, toolbar buttons.
- rendering: src/rendering/* — chunked canvas, LOD grid, selection overlay + action bar, cursor, HUD.
- ui: src/ui/* — palette window (draggable, basic/extended). A groups gutter exists but is not wired (see Gaps).

## Frame Loop & Layers

src/app/mod.rs:10-64 drives a consistent update/draw order:
- L1 Grid: GridRenderer::draw under everything except canvas.
- L2 Canvas: CanvasRenderer::update then draw (dirty‑chunk based).
- L3 Selection overlay: draw_selection_overlay.
- UI hover mask: render_ui_buttons + render_palette_window → over_ui.
- Zoom and input when not over UI: handle_zoom, handle_input.
- L4 Cursor: draw_cursor_based_on_mode.
- L5 Selection action bar: draw_selection_action_bar (Delete button).
- L6 HUD: FPS/zoom/camera info.

## State Model

- Modes: Paint | Erase | Pan | Select (src/state/mod.rs:15-26).
- ApplicationState holds: mode, palette flags/position, current_color, cells (CellGrid), camera, selection (SelectionState), clipboard, history, palette mode/page (src/state/mod.rs:92-130; defaults at 132-155).
- Clipboard and history primitives: CellChange, Command, History (src/state/mod.rs:47-83).
- Selection state and geometry: Selection, SelectionState, SelectionRect, compute_bounding_rect (src/core/selection.rs:47-52, 54-84, 219-229).

## Coordinates & Camera

- World units are integer cells; screen mapping via Camera (src/core/camera.rs:24-39).
- Cursor‑anchored zoom keeps the hovered world point fixed (src/core/camera.rs:56-69).
- Visible world rect for culling: visible_world_rect (src/core/camera.rs:41-49).

## Rendering

- Canvas (chunked): src/rendering/canvas.rs
  - Chunks 64×64 cells → RenderTarget 512×512 px (8 px per cell) (7-13).
  - Mark dirty per edit: mark_dirty (60-67).
  - Rebuild dirty chunks: update + rebuild_chunk (74-86, 88-143).
  - Draw visible chunks with camera culling: 145-206 (optional pixel rounding notes at 173-191 to avoid seams).
- Grid (LOD): compute step and blend from zoom (src/rendering/grid.rs:4-12); draw vertical/horizontal lines with 16‑cell tile emphasis (31-116).
- Selection overlay: drag rect, per‑cell highlight, bounding box, move preview (src/rendering/selection.rs:12-83).
- Selection action bar: Delete button below selection (src/rendering/selection.rs:111-141).
- Cursor: mode‑specific indicator (src/rendering/cursor.rs:6-35).
- HUD: FPS/zoom/camera overlay (src/rendering/hud.rs:19-44).

## Input & Tools

- Dispatcher: handle_input routes by mode; manages temp‑pan, hotkeys, clipboard, delete (src/input/dispatcher.rs:7-79, 33-66).
- Pan: MMB temp‑pan toggles Pan mode; drag updates camera (src/input/dispatcher.rs:136-164).
- Zoom: wheel zooms around cursor (src/input/dispatcher.rs:166-175).
- Paint/Erase: interpolated strokes via Bresenham; unified set_cell marks chunks dirty (src/input/tools.rs:6-37, 69-111).
- Selection: drag to pick only filled cells, Shift‑add union, lift+move with preview, drop to commit (src/input/selection.rs:7-66, 128-198, 200-213).
- Clipboard: copy/cut/paste; paste creates a new selection (src/input/clipboard.rs:7-31, 33-52, 54-92).
- Undo: undo_last replays before state and marks chunks dirty (src/input/dispatcher.rs:111-126).
- Batch mutations: apply_changes_and_record fills before lazily, applies, marks dirty, pushes history (src/input/dispatcher.rs:81-109).

## Hotkeys

- Modes: B Paint, E Erase, V Select (unless Ctrl/Cmd held), H/Space Pan (src/input/dispatcher.rs:50-62).
- Temp Pan: Middle Mouse Button press/release (src/input/dispatcher.rs:12-31).
- Clipboard: Ctrl/Cmd+C/X/V (src/input/dispatcher.rs:33-44).
- Undo: Ctrl/Cmd+Z (src/input/dispatcher.rs:46-48).
- Delete selection: Delete/Backspace (src/input/dispatcher.rs:64-66).

## Data Structures

- Cell and grid: Cell, CellGrid (src/core/cell.rs:4-8, 33).
- Selection: Selection, SelectionState (src/core/selection.rs:47-52, 54-84).
- Group model (present, not wired): Group { id, name, cells } (src/core/group.rs:3-8).

## UI Components

- Toolbar buttons (mode/palette toggle): src/input/ui.rs:5-15, 17-44. Returns hover state to mask input.
- Palette window (draggable; Basic 4×8 GBA; Extended paged 343 colors): src/ui/palette.rs:25-41, 42-167, 169-219; paging at 176-219.

## Known Gaps / Integration Notes

- Groups feature code is present but not integrated:
  - UI panel src/ui/groups_gutter.rs references fields that don’t exist in ApplicationState (e.g., groups_gutter_width, groups, group_index, selected_group_id, rename/context fields). The app loop doesn’t render this panel.
  - Group ops in src/input/groups.rs and Group type in src/core/group.rs exist; membership is not updated during selection moves (input/selection.rs doesn’t call update_membership_on_move).
  - Treat groups as experimental/incomplete until state + rendering integration are added.
- Painting/erasing bypasses history; to make strokes undoable, accumulate CellChange during a stroke and call apply_changes_and_record on release.
- Selection preview texture is rebuilt lazily; code ensures existence before lifting/moving.

## Quick Start Pointers

- New tool: put input in src/input/* and visuals in src/rendering/*; modify cells via apply_changes_and_record and always call CanvasRenderer::mark_dirty per edited cell.
- New overlay/HUD: follow the layer order in src/app/mod.rs:10-64 and gate input with the over_ui pattern.
- Coordinate math: use Camera::cell_to_screen/screen_to_cell and pixel_scale() for consistent sizing.

