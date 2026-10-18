use super::data::{HudBossBarInfo, HudData};
use super::layout::HudLayoutConfig;
use super::modules::{
    render_armor_status, render_boss_bar, render_coords_direction, render_crosshair,
    render_fps_ping, render_keystrokes, render_potion_effects, render_unified_hotbar,
};
use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Id, LayerId, Order, Painter, Pos2,
    Rect, Slider, Stroke, Vec2, Window,
};

/// Persistent interaction state for the Snap-to-Grid HUD Editor.
#[derive(Debug, Clone)]
pub struct HudEditorState {
    pub selected_id: Option<String>,
    pub dragging_id: Option<String>,
    pub resizing_id: Option<String>,
    pub drag_start_mouse: Pos2,
    pub drag_start_rect: Rect,
    pub snap_guide_x: Option<f32>,
    pub snap_guide_y: Option<f32>,
    pub new_profile_name: String,
    pub show_new_profile_modal: bool,
}

impl Default for HudEditorState {
    fn default() -> Self {
        Self {
            selected_id: None,
            dragging_id: None,
            resizing_id: None,
            drag_start_mouse: Pos2::ZERO,
            drag_start_rect: Rect::ZERO,
            snap_guide_x: None,
            snap_guide_y: None,
            new_profile_name: String::new(),
            show_new_profile_modal: false,
        }
    }
}

/// Renders the interactive HUD layout editor. Returns `true` if the editor was closed.
pub fn render_hud_editor(
    ctx: &egui::Context,
    state: &mut HudEditorState,
    layout: &mut HudLayoutConfig,
    data: &HudData,
    tick_counter: u64,
) -> bool {
    let mut close_editor = false;
    let screen_rect = ctx.screen_rect();
    let screen_size = screen_rect.size();

    // Compute interface-scaled grid step
    let gui_scale = ctx.zoom_factor().max(1.0);
    let effective_grid_step = (16.0_f32 * gui_scale).max(8.0);

    // 1. Semi-transparent backdrop and grid overlay on Background layer
    let bg_painter = ctx.layer_painter(LayerId::new(Order::Background, Id::new("hud_editor_grid_layer")));
    bg_painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(100));

    let grid_col = Color32::from_rgba_unmultiplied(255, 255, 255, 14);
    let mut x = effective_grid_step;
    while x < screen_size.x {
        bg_painter.line_segment(
            [Pos2::new(x, 0.0), Pos2::new(x, screen_size.y)],
            Stroke::new(1.0_f32, grid_col),
        );
        x += effective_grid_step;
    }

    let mut y = effective_grid_step;
    while y < screen_size.y {
        bg_painter.line_segment(
            [Pos2::new(0.0, y), Pos2::new(screen_size.x, y)],
            Stroke::new(1.0_f32, grid_col),
        );
        y += effective_grid_step;
    }

    // Rigid center crosshair on background layer as center reference
    let center = Pos2::new(screen_size.x * 0.5, screen_size.y * 0.5);
    render_crosshair(&bg_painter, Rect::from_center_size(center, Vec2::splat(16.0)));

    // 2. Mouse Input Handling for dragging and resizing
    let pointer_over_ui = ctx.is_pointer_over_area();
    let mouse_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
    let mouse_pressed = ctx.input(|i| i.pointer.primary_pressed());
    let mouse_down = ctx.input(|i| i.pointer.primary_down());

    if !mouse_down {
        state.dragging_id = None;
        state.resizing_id = None;
        state.snap_guide_x = None;
        state.snap_guide_y = None;
    }

    let element_ids: Vec<String> = layout
        .active_elements()
        .keys()
        .filter(|&k| k != "crosshair")
        .cloned()
        .collect();

    // 3. Process mouse interactions (Resizing and Moving)
    let resize_handle_size = 12.0_f32;

    // Check if mouse pressed to initiate drag or resize ONLY when not interacting with windows
    if mouse_pressed && !pointer_over_ui {
        let mut hit_element = false;

        // Check selected element's resize handle first
        if let Some(sel_id) = &state.selected_id {
            if let Some(elem) = layout.get_element(sel_id) {
                let rect = elem.screen_rect_with_grid(screen_size, effective_grid_step);
                let handle_rect = Rect::from_min_max(
                    Pos2::new(rect.right() - resize_handle_size, rect.bottom() - resize_handle_size),
                    Pos2::new(rect.right() + 2.0, rect.bottom() + 2.0),
                );
                if handle_rect.contains(mouse_pos) {
                    state.resizing_id = Some(sel_id.clone());
                    state.drag_start_mouse = mouse_pos;
                    state.drag_start_rect = rect;
                    hit_element = true;
                }
            }
        }

        // Check if any element body was clicked
        if !hit_element {
            for id in element_ids.iter().rev() {
                if let Some(elem) = layout.get_element(id) {
                    let rect = elem.screen_rect_with_grid(screen_size, effective_grid_step);
                    let handle_rect = Rect::from_min_max(
                        Pos2::new(rect.right() - resize_handle_size, rect.bottom() - resize_handle_size),
                        Pos2::new(rect.right() + 2.0, rect.bottom() + 2.0),
                    );

                    if handle_rect.contains(mouse_pos) {
                        state.selected_id = Some(id.clone());
                        state.resizing_id = Some(id.clone());
                        state.drag_start_mouse = mouse_pos;
                        state.drag_start_rect = rect;
                        hit_element = true;
                        break;
                    } else if rect.contains(mouse_pos) {
                        state.selected_id = Some(id.clone());
                        state.dragging_id = Some(id.clone());
                        state.drag_start_mouse = mouse_pos;
                        state.drag_start_rect = rect;
                        hit_element = true;
                        break;
                    }
                }
            }
        }

        // If clicked on empty space on the background canvas, deselect
        if !hit_element {
            state.selected_id = None;
        }
    }

    // Execute active dragging or resizing
    if mouse_down {
        if let Some(resizing_id) = &state.resizing_id {
            let delta = mouse_pos - state.drag_start_mouse;
            let target_w = (state.drag_start_rect.width() + delta.x).max(16.0);
            let target_h = (state.drag_start_rect.height() + delta.y).max(16.0);

            if let Some(elem) = layout.get_element_mut(resizing_id) {
                elem.set_from_screen_size(
                    Vec2::new(target_w, target_h),
                    screen_size,
                    effective_grid_step,
                );
            }
            ctx.set_cursor_icon(CursorIcon::ResizeNwSe);
        } else if let Some(dragging_id) = &state.dragging_id {
            let delta = mouse_pos - state.drag_start_mouse;
            let elem_w = state.drag_start_rect.width();
            let elem_h = state.drag_start_rect.height();
            let mut target_min = state.drag_start_rect.min + delta;

            // Vertical center magnetization (centering along X axis: element center X == screen center X)
            let screen_center_x = screen_size.x * 0.5;
            let elem_center_x = target_min.x + elem_w * 0.5;
            if (elem_center_x - screen_center_x).abs() <= 14.0 {
                target_min.x = screen_center_x - elem_w * 0.5;
                state.snap_guide_x = Some(screen_center_x);
            } else {
                state.snap_guide_x = None;
            }

            // Horizontal center magnetization (centering along Y axis: element center Y == screen center Y)
            let screen_center_y = screen_size.y * 0.5;
            let elem_center_y = target_min.y + elem_h * 0.5;
            if (elem_center_y - screen_center_y).abs() <= 14.0 {
                target_min.y = screen_center_y - elem_h * 0.5;
                state.snap_guide_y = Some(screen_center_y);
            } else {
                state.snap_guide_y = None;
            }

            if let Some(elem) = layout.get_element_mut(dragging_id) {
                elem.set_from_screen_pos(target_min, screen_size, effective_grid_step);
            }
            ctx.set_cursor_icon(CursorIcon::Grabbing);
        }
    }

    // 4. Render HUD elements on Middle layer (strictly underneath Foreground Toolbar window)
    let elem_painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("hud_editor_elements_layer")));
    let fg_painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("hud_editor_fg_layer")));

    // Active center guide lines
    if let Some(cx) = state.snap_guide_x {
        fg_painter.line_segment(
            [Pos2::new(cx, 0.0), Pos2::new(cx, screen_size.y)],
            Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 255)),
        );
    }
    if let Some(cy) = state.snap_guide_y {
        fg_painter.line_segment(
            [Pos2::new(0.0, cy), Pos2::new(screen_size.x, cy)],
            Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 255)),
        );
    }

    let bg_alpha = (255.0 * layout.global_opacity).clamp(0.0, 255.0) as u8;
    let border_alpha = (220.0 * layout.global_opacity).clamp(0.0, 255.0) as u8;
    let elem_bg_col = Color32::from_rgba_unmultiplied(16, 18, 24, bg_alpha);
    let elem_border_col = Color32::from_rgba_unmultiplied(65, 72, 88, border_alpha);

    for id in &element_ids {
        let elem_config = match layout.get_element(id) {
            Some(e) => e.clone(),
            None => continue,
        };

        let rect = elem_config.screen_rect_with_grid(screen_size, effective_grid_step);
        let is_selected = state.selected_id.as_deref() == Some(id.as_str());
        let is_hovered = rect.contains(mouse_pos) && !pointer_over_ui;

        // Draw element background and border with global opacity
        if bg_alpha > 0 {
            elem_painter.rect_filled(rect, CornerRadius::same(4), elem_bg_col);
        }
        if border_alpha > 0 {
            elem_painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0_f32, elem_border_col),
                egui::StrokeKind::Inside,
            );
        }

        // Render actual module contents inside the rect
        render_module_by_id(&elem_painter, id, rect, data, tick_counter);

        // Frame and handles on Foreground layer
        let (frame_stroke, tag_col) = if is_selected {
            (Stroke::new(1.8_f32, Color32::from_rgb(255, 215, 0)), Color32::from_rgb(255, 215, 0))
        } else if is_hovered {
            (Stroke::new(1.2_f32, Color32::from_rgb(85, 255, 255)), Color32::from_rgb(85, 255, 255))
        } else {
            (Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(120, 130, 150, 110)), Color32::from_rgb(160, 165, 180))
        };

        fg_painter.rect_stroke(rect, CornerRadius::same(3), frame_stroke, egui::StrokeKind::Outside);

        // Element name tag
        let display_name = if elem_config.visible {
            elem_config.name.clone()
        } else {
            format!("{} (Hidden)", elem_config.name)
        };

        fg_painter.text(
            Pos2::new(rect.left() + 2.0, rect.top() - 3.0),
            Align2::LEFT_BOTTOM,
            display_name,
            FontId::proportional(10.0),
            tag_col,
        );

        // Resize handle at bottom-right corner
        let handle_rect = Rect::from_min_max(
            Pos2::new(rect.right() - resize_handle_size, rect.bottom() - resize_handle_size),
            Pos2::new(rect.right(), rect.bottom()),
        );

        let handle_hovered = handle_rect.contains(mouse_pos) && !pointer_over_ui;
        if handle_hovered && state.dragging_id.is_none() {
            ctx.set_cursor_icon(CursorIcon::ResizeNwSe);
        } else if is_hovered && state.resizing_id.is_none() && state.dragging_id.is_none() {
            ctx.set_cursor_icon(CursorIcon::Grab);
        }

        if is_selected || is_hovered {
            let handle_col = if is_selected {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(85, 255, 255)
            };

            fg_painter.rect_filled(handle_rect, CornerRadius::same(2), handle_col);
            // Diagonal grip lines inside handle
            let p0 = Pos2::new(handle_rect.left() + 3.0, handle_rect.bottom() - 3.0);
            let p1 = Pos2::new(handle_rect.right() - 3.0, handle_rect.top() + 3.0);
            fg_painter.line_segment([p0, p1], Stroke::new(1.0_f32, Color32::from_rgb(20, 24, 32)));
        }
    }

    // 5. Floating Top Toolbar (Profile switcher, Opacity slider, Selected visibility, Save)
    // Placed on Order::Foreground so it is always on top of all HUD elements and canvas handles
    Window::new("HudEditorToolbar")
        .order(Order::Foreground)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 12.0))
        .frame(
            Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(16, 18, 24, 250))
                .stroke(Stroke::new(1.5_f32, Color32::from_rgb(60, 68, 85)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(14, 8)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("HUD Layout Editor")
                        .color(Color32::from_rgb(85, 255, 255))
                        .strong(),
                );
                ui.separator();

                // Profile Selector
                ui.label("Profile:");
                let active_name = layout.active_profile.clone();
                egui::ComboBox::from_id_salt("hud_profile_combo")
                    .selected_text(&active_name)
                    .show_ui(ui, |ui| {
                        let profiles: Vec<String> = layout.profiles.keys().cloned().collect();
                        for p in profiles {
                            if ui.selectable_label(p == active_name, &p).clicked() {
                                layout.switch_profile(&p);
                            }
                        }
                    });

                if ui.button("+ New").clicked() {
                    state.show_new_profile_modal = true;
                    state.new_profile_name = format!("Profile {}", layout.profiles.len() + 1);
                }

                ui.separator();

                // Master Opacity Slider for all HUD elements
                ui.label("Opacity:");
                ui.add(
                    Slider::new(&mut layout.global_opacity, 0.0..=1.0)
                        .show_value(true)
                        .step_by(0.05),
                );

                // Selected Element Controls (Visible button moved into top toolbar)
                if let Some(sel_id) = state.selected_id.clone() {
                    if let Some(elem) = layout.get_element_mut(&sel_id) {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(&elem.name)
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                        );

                        let vis_text = if elem.visible { "👁 Visible" } else { "🚫 Hidden" };
                        let vis_color = if elem.visible {
                            Color32::from_rgb(85, 255, 85)
                        } else {
                            Color32::from_rgb(255, 85, 85)
                        };

                        if ui
                            .button(egui::RichText::new(vis_text).color(vis_color).strong())
                            .clicked()
                        {
                            elem.visible = !elem.visible;
                        }

                        if ui.button("✖").clicked() {
                            state.selected_id = None;
                        }
                    }
                }

                ui.separator();

                // Reset & Save & Close buttons
                if ui.button("Reset").clicked() {
                    layout.reset_active_profile();
                }

                if ui
                    .button(
                        egui::RichText::new("Save & Close (RShift)")
                            .color(Color32::from_rgb(85, 255, 85))
                            .strong(),
                    )
                    .clicked()
                {
                    close_editor = true;
                }
            });
        });

    // 6. Modal dialog for creating a new profile
    if state.show_new_profile_modal {
        Window::new("Create New Profile")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .title_bar(true)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_width(200.0);
                ui.label("Profile name:");
                ui.text_edit_singleline(&mut state.new_profile_name);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        let name = state.new_profile_name.trim();
                        if !name.is_empty() {
                            layout.create_profile(name, Some(&layout.active_profile.clone()));
                        }
                        state.show_new_profile_modal = false;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_new_profile_modal = false;
                    }
                });
            });
    }

    close_editor
}

/// Helper router to draw a module's content given its ID.
pub fn render_module_by_id(
    painter: &Painter,
    id: &str,
    rect: Rect,
    data: &HudData,
    tick_counter: u64,
) {
    match id {
        "crosshair" => render_crosshair(painter, rect),
        "hotbar" => render_unified_hotbar(painter, rect, data, tick_counter),
        "boss_bar" => {
            if data.boss_bars.is_empty() {
                // In editor preview, show placeholder Wither boss so user can arrange the element
                let preview = HudData {
                    boss_bars: vec![HudBossBarInfo::new("Wither Boss", 1.0)],
                    ..data.clone()
                };
                render_boss_bar(painter, rect, &preview);
            } else {
                render_boss_bar(painter, rect, data);
            }
        }
        "keystrokes" => render_keystrokes(painter, rect, data),
        "fps_ping" => render_fps_ping(painter, rect, data),
        "armor_status" => {
            let has_armor = data.armor_items.iter().any(|a| a.is_some())
                || data.held_item.as_ref().map_or(false, |h| h.max_damage > 0);
            if !has_armor {
                // In editor preview, show placeholder text
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Armor Status (Preview)",
                    FontId::proportional(10.0),
                    Color32::from_rgb(140, 145, 160),
                );
            } else {
                render_armor_status(painter, rect, data);
            }
        }
        "potion_effects" => {
            if data.potion_effects.is_empty() {
                // In editor preview, show placeholder text
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Potion Effects (Preview)",
                    FontId::proportional(10.0),
                    Color32::from_rgb(140, 145, 160),
                );
            } else {
                render_potion_effects(painter, rect, data);
            }
        }
        "coords_direction" => render_coords_direction(painter, rect, data),
        _ => {}
    }
}
