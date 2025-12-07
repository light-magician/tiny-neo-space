# Feature Plan: Zoom — Max Zoom-Out Targets 16×16 = 1 Cell

Goal
- At the maximum zoomed-out level, a 16×16 block of cells occupies the same on-screen size as a single cell at the default zoom level.
- Preserve the current default zoom behavior and appearance exactly as-is.

Context (Current Architecture)
- Camera zoom model: `pixel_scale = BASE_CELL_PIXELS * zoom` (src/core/camera.rs:24-39). Default `zoom = 1.0` (src/core/camera.rs:16-22). Min/max clamps `MIN_ZOOM`, `MAX_ZOOM` (src/core/camera.rs:3-5) in `zoom_around_cursor` (src/core/camera.rs:56-69).
- Grid and canvas rendering use `pixel_scale()` and cull via `visible_world_rect()`; changing `MIN_ZOOM` only changes how far the user can zoom out, consistent with current design.

Design
- Make minimum zoom exactly 1/16 of default, so 16 cells at min zoom equal 1 cell at default.
  - At default: 1 cell width = `BASE_CELL_PIXELS`.
  - At min: 1 cell width = `BASE_CELL_PIXELS * MIN_ZOOM`.
  - Requirement: `16 * (BASE_CELL_PIXELS * MIN_ZOOM) == BASE_CELL_PIXELS` ⇒ `MIN_ZOOM == 1/16 == 0.0625`.
- Leave `BASE_CELL_PIXELS` and `MAX_ZOOM` unchanged to preserve default behavior and maximum zoom-in.

Change Summary
- Update `MIN_ZOOM` constant from `0.2` to `1.0 / 16.0`.

File Changes
- src/core/camera.rs:3-5

Code Diff (illustrative)
```rust
// src/core/camera.rs:3-5
pub const BASE_CELL_PIXELS: f32 = 24.0;
pub const MIN_ZOOM: f32 = 1.0 / 16.0; // 16×16 cells at min zoom match 1 cell at default
pub const MAX_ZOOM: f32 = 4.0;
```

Rationale & Compatibility
- Grid LOD: `compute_lod` in `src/rendering/grid.rs` uses `1.0/zoom` and blends between power‑of‑two steps. With `MIN_ZOOM = 1/16`, the lowest LOD becomes `step = 16` (expected and desirable for 16‑cell granularity).
- Canvas chunks: Draw size for a 64×64 chunk at min zoom = `64 * pixel_scale = 64 * (BASE_CELL_PIXELS/16)`. With `BASE_CELL_PIXELS = 24`, that’s `64 * 1.5 = 96` px. This is fine; the renderer scales `RenderTarget`s via `draw_texture_ex`.
- Seams: If any sub‑pixel rendering artifacts appear at fractional cell size (1.5 px), optional pixel‑rounding is already documented in `src/rendering/canvas.rs:173-191` and can be toggled without architectural changes.

Acceptance Criteria
- At default zoom (`zoom = 1.0`), a single cell width equals `BASE_CELL_PIXELS` (no change visually).
- At max zoom‑out (`zoom = MIN_ZOOM`), a 16×16 square spans the same on‑screen width and height as a single cell at default.
- Panning, selection, grid, and canvas rendering still function normally at all zoom levels.

Manual Verification Steps
1) Default cell size check
   - Launch app; at default zoom, measure approximately one cell’s on‑screen size; expect ≈ `BASE_CELL_PIXELS` (24 px).
2) Max zoom‑out ratio check
   - Scroll out until zoom no longer decreases (clamped). Draw a 16×16 selection and compare its on‑screen size to one default‑zoom cell (side‑by‑side capture or visual comparison). They should match.
3) Grid LOD sanity
   - While zooming out from default to min, observe grid lines thin/fade smoothly; at min zoom, grid major lines align on 16‑cell boundaries (emphasis already exists at multiples of 16 in `grid.rs`).

Risks & Mitigations
- Fractional cell pixel sizes (1.5 px) can show faint seams on some GPUs at specific zooms. If observed, enable pixel rounding as per comments in `src/rendering/canvas.rs:173-191` for seam‑free visuals.
- Input precision at small sizes remains correct because world↔screen mapping stays consistent via `Camera` helpers.

Implementation Notes
- No other modules require changes; the single constant update satisfies the requirement by design.
- Keep the value as a fraction (`1.0 / 16.0`) for clarity and future adjustments (e.g., if a different tile dimension is desired later).

Open Questions
- Should we expose min/max zoom ratios as configurable settings (e.g., via a debug UI or config file) for future tuning? For now, keep it constant per spec.

