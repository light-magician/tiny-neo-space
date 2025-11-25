# Feature Plan: Groups (cell grouping + gutter UI)

This plan implements “Groups,” an abstraction over selected cells, with a left-side gutter (file-tree-like) UI. It follows the project’s architecture (state-driven, renderer/input separation, Macroquad UI drawing) and integrates with existing selection, canvas rendering, and input systems.

Goals:
- Create groups from the current selection (Ctrl/Cmd+G and a “Group” button in the selection action bar).
- Render a left gutter listing groups. Clicking selects the group on the canvas. Double-click renames. Right-click opens an Ungroup/Delete context menu.
- Sync gutter highlight with canvas selection: when a group is selected on canvas, highlight its name in the gutter; if selection includes cells belonging to the group, show a partial highlight.
- Update group membership when selected cells are moved or deleted.

## 1) Data Model

Add group types and state. Keep core data pure; keep UI-editing state in ApplicationState.

- New file: core/group.rs

```rust
// core/group.rs (new)
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Group {
    pub id: u32,
    pub name: String,
    pub cells: HashSet<(i32, i32)>,
}
```

- Wire into core/mod.rs

```rust
// core/mod.rs
pub mod group;         // add
pub use group::*;      // add
```

- Extend ApplicationState (src/state/mod.rs)

Add fields (suggested location below existing palette fields, around lines 118+; final line indices may shift):

```rust
// src/state/mod.rs: after line ~129
// Group system
pub groups: Vec<crate::core::group::Group>,        // ordered groups for gutter
pub group_index: std::collections::HashMap<(i32,i32), u32>, // cell -> group id (single membership)
pub next_group_id: u32,
pub selected_group_id: Option<u32>,

// Gutter UI state (double‑click and context menu)
pub groups_gutter_width: f32,           // e.g., 200.0
pub group_renaming_id: Option<u32>,     // currently renaming
pub group_rename_buffer: String,        // inline text buffer
pub group_last_click_id: Option<u32>,   // for double‑click detection
pub group_last_click_time: f64,         // seconds
pub group_context_target: Option<u32>,  // right‑click target
pub group_context_pos: Vec2,            // popup screen position
```

Initialize in ApplicationState::new (around lines 132‑155):

```rust
// src/state/mod.rs: within ApplicationState::new()
groups: Vec::new(),
group_index: std::collections::HashMap::new(),
next_group_id: 1,
selected_group_id: None,

groups_gutter_width: 200.0,
group_renaming_id: None,
group_rename_buffer: String::new(),
group_last_click_id: None,
group_last_click_time: 0.0,
group_context_target: None,
group_context_pos: Vec2::ZERO,
```

Notes:
- One‑group‑per‑cell simplifies mapping; extensible later to multi‑membership.
- `group_index` enables fast membership checks and update on moves/deletes.

## 2) Input: Group Operations API

Create an input module encapsulating group ops and mapping maintenance.

- New file: src/input/groups.rs

```rust
use std::collections::{HashMap, HashSet};
use macroquad::prelude::*;
use crate::state::{ApplicationState, CellChange};
use crate::rendering::CanvasRenderer;
use crate::core::group::Group;

pub fn create_group_from_selection(state: &mut ApplicationState) {
    let sel = match &state.selection.current { Some(s) => s, None => return };
    let cells: HashSet<(i32,i32)> = match &sel.kind {
        crate::core::selection::SelectionKind::Cells(set) => set.clone(),
    };
    if cells.is_empty() { return; }

    let id = state.next_group_id; state.next_group_id += 1;
    let name = format!("Group {}", id);
    state.groups.push(Group { id, name, cells: cells.clone() });
    for &c in &cells { state.group_index.insert(c, id); }
    state.selected_group_id = Some(id);
}

pub fn select_group(state: &mut ApplicationState, id: u32) {
    if let Some(g) = state.groups.iter().find(|g| g.id == id) {
        use crate::core::selection::{Selection, SelectionKind, compute_bounding_rect};
        if let Some(rect) = compute_bounding_rect(&g.cells) {
            state.selection.current = Some(Selection { rect, kind: SelectionKind::Cells(g.cells.clone()), preview: None });
            state.selected_group_id = Some(id);
        }
    }
}

pub fn rename_group(state: &mut ApplicationState, id: u32, new_name: String) {
    if let Some(g) = state.groups.iter_mut().find(|g| g.id == id) { g.name = new_name; }
}

pub fn ungroup(state: &mut ApplicationState, id: u32) {
    if let Some(pos) = state.groups.iter().position(|g| g.id == id) {
        for &c in state.groups[pos].cells.iter() { state.group_index.remove(&c); }
        state.groups.remove(pos);
        if state.selected_group_id == Some(id) { state.selected_group_id = None; }
    }
}

pub fn delete_group_and_cells(state: &mut ApplicationState, canvas: &mut CanvasRenderer, id: u32) {
    if let Some(pos) = state.groups.iter().position(|g| g.id == id) {
        let mut changes: Vec<CellChange> = Vec::new();
        for &c in state.groups[pos].cells.iter() {
            if state.cells.get(&c).is_some() { changes.push(CellChange { coord: c, before: None, after: None }); }
            state.group_index.remove(&c);
        }
        state.groups.remove(pos);
        if !changes.is_empty() {
            crate::input::dispatcher::apply_changes_and_record(state, canvas, changes);
        }
        if state.selected_group_id == Some(id) { state.selected_group_id = None; state.selection.current = None; }
    }
}

// Update membership when cells move (old -> new pairs)
pub fn update_membership_on_move(state: &mut ApplicationState, moved: &[( (i32,i32), (i32,i32) )]) {
    for (old, newc) in moved {
        if let Some(id) = state.group_index.remove(old) {
            state.group_index.insert(*newc, id);
            if let Some(g) = state.groups.iter_mut().find(|g| g.id == id) {
                g.cells.remove(old);
                g.cells.insert(*newc);
            }
        }
    }
}

// Remove membership for deleted cells
pub fn remove_cells_from_groups(state: &mut ApplicationState, cells: &[(i32,i32)]) {
    for &c in cells {
        if let Some(id) = state.group_index.remove(&c) {
            if let Some(g) = state.groups.iter_mut().find(|g| g.id == id) { g.cells.remove(&c); }
        }
    }
}

// Helper: set selected_group_id based on current selection (exact match)
pub fn sync_selected_group_from_selection(state: &mut ApplicationState) {
    let sel = match &state.selection.current { Some(s) => s, None => { state.selected_group_id = None; return; } };
    let selected: &std::collections::HashSet<(i32,i32)> = match &sel.kind { crate::core::selection::SelectionKind::Cells(set) => set };
    for g in &state.groups {
        if g.cells.len() == selected.len() && g.cells.iter().all(|c| selected.contains(c)) {
            state.selected_group_id = Some(g.id); return;
        }
    }
    state.selected_group_id = None;
}
```

- Export from input/mod.rs

```rust
// src/input/mod.rs
pub mod groups;                // add
pub use groups::*;             // add
```

- Hotkey in dispatcher (near other Ctrl/Cmd hotkeys, around src/input/dispatcher.rs:33‑48):

```rust
// src/input/dispatcher.rs (add after Ctrl/Cmd+Z block)
if ctrl_or_cmd() && is_key_pressed(KeyCode::G) {
    crate::input::groups::create_group_from_selection(state);
}
```

- Update selection moves and deletes to maintain membership:
  - In `drop_lifted` (src/input/selection.rs:167‑198), after reinsertion, build `moved_pairs` from each lifted cell’s old->new coord and call:

```rust
crate::input::groups::update_membership_on_move(state, &moved_pairs);
```

  - In `delete_selection` (src/input/selection.rs:200‑213), collect removed coords and call:

```rust
crate::input::groups::remove_cells_from_groups(state, &removed_coords);
```

  - In `cut_selection` (src/input/clipboard.rs:33‑52), also remove memberships for deleted coords.

## 3) UI: Gutter (Groups Panel)

New UI module draws a left-side panel listing groups, supports select, rename (double‑click), and context menu (right‑click). Returns `true` when the mouse is over it, consistent with UI hover pattern.

- New file: src/ui/groups_gutter.rs

```rust
use macroquad::prelude::*;
use crate::state::ApplicationState;
use crate::rendering::CanvasRenderer;

pub fn render_groups_gutter(state: &mut ApplicationState, canvas: &mut CanvasRenderer) -> bool {
    let w = state.groups_gutter_width; let h = screen_height();
    let x = 0.0; let y = 0.0; let mouse = Vec2::from(mouse_position());
    let over = Rect::new(x, y, w, h).contains(mouse);

    // Panel background
    draw_rectangle(x, y, w, h, Color::from_rgba(245,245,250,255));
    draw_rectangle_lines(x, y, w, h, 2.0, BLACK);

    // List items
    let item_h = 22.0; let mut cur_y = y + 6.0;
    let now = get_time();

    for g in &state.groups.clone() { // clone for borrow ease
        let item_rect = Rect::new(x+6.0, cur_y, w-12.0, item_h);
        let is_exact = state.selected_group_id == Some(g.id);
        let selection_cells = state.selection.current.as_ref().and_then(|sel| match &sel.kind { crate::core::selection::SelectionKind::Cells(s) => Some(s), });
        let is_partial = selection_cells.map_or(false, |selset| selset.iter().any(|c| g.cells.contains(c)));

        let bg = if is_exact { Color::from_rgba(180,210,255,255) } else if is_partial { Color::from_rgba(210,225,255,255) } else { Color::from_rgba(230,230,235,255) };
        draw_rectangle(item_rect.x, item_rect.y, item_rect.w, item_rect.h, bg);
        draw_rectangle_lines(item_rect.x, item_rect.y, item_rect.w, item_rect.h, 1.0, BLACK);

        // Name or rename textbox
        let mut label = g.name.clone();
        let renaming = state.group_renaming_id == Some(g.id);
        if renaming { label = state.group_rename_buffer.clone(); }
        draw_text(&label, item_rect.x + 6.0, item_rect.y + 15.0, 16.0, BLACK);

        // Mouse interactions
        let clicked_left = is_mouse_button_pressed(MouseButton::Left) && item_rect.contains(mouse);
        let clicked_right = is_mouse_button_pressed(MouseButton::Right) && item_rect.contains(mouse);

        if clicked_left {
            // Double‑click detection
            if state.group_last_click_id == Some(g.id) && (now - state.group_last_click_time) < 0.35 {
                state.group_renaming_id = Some(g.id);
                state.group_rename_buffer = g.name.clone();
            } else {
                crate::input::groups::select_group(state, g.id);
            }
            state.group_last_click_id = Some(g.id); state.group_last_click_time = now;
        }

        if clicked_right {
            state.group_context_target = Some(g.id);
            state.group_context_pos = mouse;
        }

        cur_y += item_h + 4.0;
    }

    // Handle renaming commit on Enter
    if let Some(id) = state.group_renaming_id {
        if is_key_pressed(KeyCode::Enter) {
            crate::input::groups::rename_group(state, id, state.group_rename_buffer.clone());
            state.group_renaming_id = None; state.group_rename_buffer.clear();
        }
    }

    // Context menu
    if let Some(id) = state.group_context_target {
        let px = state.group_context_pos.x; let py = state.group_context_pos.y;
        let menu_w = 120.0; let menu_h = 48.0; let item_h = 22.0;
        draw_rectangle(px, py, menu_w, menu_h, Color::from_rgba(250,250,250,255));
        draw_rectangle_lines(px, py, menu_w, menu_h, 1.0, BLACK);
        let ungroup_rect = Rect::new(px, py, menu_w, item_h);
        let delete_rect = Rect::new(px, py + item_h, menu_w, item_h);
        draw_text("Ungroup", px + 8.0, py + 15.0, 16.0, BLACK);
        draw_text("Delete", px + 8.0, py + 15.0 + item_h, 16.0, BLACK);
        if is_mouse_button_pressed(MouseButton::Left) {
            if ungroup_rect.contains(mouse) { crate::input::groups::ungroup(state, id); state.group_context_target = None; }
            else if delete_rect.contains(mouse) { crate::input::groups::delete_group_and_cells(state, canvas, id); state.group_context_target = None; }
            else { state.group_context_target = None; }
        }
    }

    over
}
```

- Wire into UI module:

```rust
// src/ui/mod.rs
pub mod groups_gutter;                      // add
pub use groups_gutter::render_groups_gutter; // add
```

## 4) App Loop Integration

Integrate gutter rendering and UI hover handling, mirroring palette/buttons.

- `src/app/mod.rs` changes (import and hover):

```rust
// top
use crate::ui::render_palette_window;
use crate::ui::render_groups_gutter;        // add

// inside loop, after draw_selection_overlay and before input
let over_buttons = render_ui_buttons(&mut state);
let over_palette = render_palette_window(&mut state);
let over_gutter = render_groups_gutter(&mut state, &mut canvas_renderer); // add
let over_ui = over_buttons || over_palette || over_gutter;                 // modify
```

Optionally, sync selected_group_id from current canvas selection each frame (before gutter render) to update exact-match highlighting even when selection was made on-canvas:

```rust
crate::input::groups::sync_selected_group_from_selection(&mut state);
```

## 5) Selection Action Bar: “Group” button

Add a second button next to “Delete” in the selection action bar.

- `src/rendering/selection.rs:111-141` near the Delete button, add:

```rust
// After drawing Delete, add Group button to the right
if draw_action_button("Group", bar_x + 80.0, bar_y + 2.0, 70.0, 24.0) {
    crate::input::groups::create_group_from_selection(state);
}
```

This mirrors the existing pattern that calls `delete_selection` from input.

## 6) Membership Maintenance Hooks

Ensure group membership remains correct when cells move or delete.

- On drop after moving selection (src/input/selection.rs:167‑198):

```rust
// Build (old -> new) pairs from lifted cells
let mut moved_pairs = Vec::new();
for lifted in /* previously lifted */ { moved_pairs.push((lifted.coord, /* dest */)); }
crate::input::groups::update_membership_on_move(state, &moved_pairs);
```

- On deletion (src/input/selection.rs delete_selection): collect removed coords into a `Vec<(i32,i32)>` and call `remove_cells_from_groups`.

- On cut (src/input/clipboard.rs cut_selection): likewise remove memberships for deleted coords.

## 7) Hotkeys

Add grouping hotkey to dispatcher:

```rust
// src/input/dispatcher.rs (near other Ctrl/Cmd hotkeys)
if ctrl_or_cmd() && is_key_pressed(KeyCode::G) {
    crate::input::groups::create_group_from_selection(state);
}
```

## 8) Rendering/UX Notes

- Gutter width is fixed (e.g., 200 px) and drawn at x=0.
- Exact‑match highlight (group fully selected) uses a stronger blue than partial highlight (any overlap with current selection).
- Double‑click threshold of ~350ms (`get_time()` delta) is adequate for Macroquad.
- Inline rename uses an in‑memory buffer and commits on Enter; Escape can cancel (optional enhancement).
- Context menu appears at the right‑click position and dismisses on next click outside.

## 9) Minimal Test Cases (manual)

- Create a shape, select cells, Ctrl/Cmd+G → group appears in gutter and is highlighted; clicking name selects group on canvas.
- Double‑click group name → rename mode; type and press Enter → name updates.
- Right‑click group → choose Ungroup → group disappears, cells remain; choose Delete → cells removed, canvas chunks marked dirty, history updated.
- Move a selected group on canvas (drag selection) → after drop, gutter selection persists and group membership updates to new coordinates.
- Selecting any single cell that belongs to a group partially highlights that group in the gutter.

## 10) Future Enhancements (optional)

- Multi‑group membership per cell (change `group_index` to HashMap<Coord, Vec<GroupId>>).
- Persist groups to disk with project save/load.
- Drag‑reorder groups and nested groups/folders.
- Visual icons/badges for groups and hot hover states.

