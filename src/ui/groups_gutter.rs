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
