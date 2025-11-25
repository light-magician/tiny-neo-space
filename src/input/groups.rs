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
