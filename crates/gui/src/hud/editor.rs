use super::data::HudData;
use super::element::HudAnchor;
use super::layout::HudLayoutConfig;
use super::modules::{
    get_module_content_size, render_armor_status, render_boss_bar, render_coords_direction,
    render_crosshair, render_exp_bar, render_fps_ping, render_hotbar, render_keystrokes,
    render_potion_effects, render_vitals,
};
use egui::{
    Align2, Color32, CornerRadius, DragValue, FontId, Frame, Id, LayerId, Order, Painter, Pos2, Rect,
    Slider, Stroke, Vec2, Window,
};

/// Persistent interaction state for the Snap-to-Grid HUD Editor.
#[derive(Debug, Clone)]
pub struct HudEditorState {
    pub selected_id: Option<String>,
    pub dragging_id: Option<String>,
    pub drag_start_mouse: Pos2,
    pub drag_start_rect: Rect,
    pub new_profile_name: String,
    pub show_new_profile_modal: bool,
}

impl Default for HudEditorState {
    fn default() -> Self {
        Self {
            selected_id: None,
            dragging_id: None,
            drag_start_mouse: Pos2::ZERO,
            drag_start_rect: Rect::ZERO,
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

    // 1. Semi-transparent backdrop overlay
    let overlay_painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("hud_editor_overlay")));
    overlay_painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(90));

    // 2. Grid lines overlay (if grid snapping is enabled)
    let grid_step = layout.grid_snap_size;
    if grid_step > 0 {
        let grid_col = Color32::from_rgba_unmultiplied(255, 255, 255, 14);
        let mut x = grid_step as f32;
        while x < screen_size.x {
            overlay_painter.line_segment(
                [Pos2::new(x, 0.0), Pos2::new(x, screen_size.y)],
                Stroke::new(1.0_f32, grid_col),
            );
            x += grid_step as f32;
        }

        let mut y = grid_step as f32;
        while y < screen_size.y {
            overlay_painter.line_segment(
                [Pos2::new(0.0, y), Pos2::new(screen_size.x, y)],
                Stroke::new(1.0_f32, grid_col),
            );
            y += grid_step as f32;
        }
    }

    // 3. Collect element bounding boxes and render HUD modules with editor frames
    let mouse_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
    let mouse_down = ctx.input(|i| i.pointer.primary_down());
    let mouse_clicked = ctx.input(|i| i.pointer.primary_clicked());

    // Stop dragging if mouse button is released
    if !mouse_down && state.dragging_id.is_some() {
        state.dragging_id = None;
    }

    let element_ids: Vec<String> = layout.active_elements().keys().cloned().collect();

    for id in &element_ids {
        let content_size = get_module_content_size(id, data);
        let elem_config = layout.get_element(id).unwrap().clone();

        let rect = elem_config.screen_rect(screen_size, content_size);
        let is_selected = state.selected_id.as_deref() == Some(id.as_str());
        let is_hovered = rect.contains(mouse_pos);

        // Handle element selection on click
        if mouse_clicked && is_hovered {
            state.selected_id = Some(id.clone());
            state.dragging_id = Some(id.clone());
            state.drag_start_mouse = mouse_pos;
            state.drag_start_rect = rect;
        }

        // Handle dragging with live snap-to-grid and screen bounds clamping
        if state.dragging_id.as_deref() == Some(id.as_str()) && mouse_down {
            let delta = mouse_pos - state.drag_start_mouse;
            let target_min = state.drag_start_rect.min + delta;

            let clamped_x = target_min.x.clamp(0.0, (screen_size.x - rect.width()).max(0.0));
            let clamped_y = target_min.y.clamp(0.0, (screen_size.y - rect.height()).max(0.0));

            if let Some(elem) = layout.get_element_mut(id) {
                elem.set_from_screen_pos(
                    Pos2::new(clamped_x, clamped_y),
                    screen_size,
                    content_size,
                    grid_step,
                );
            }
        }

        // Draw HUD module background & border if configured
        if elem_config.bg_color[3] > 0 {
            overlay_painter.rect_filled(
                rect,
                CornerRadius::same(elem_config.corner_radius as u8),
                elem_config.bg_color32(),
            );
        }
        if elem_config.border_width > 0.0 && elem_config.border_color[3] > 0 {
            overlay_painter.rect_stroke(
                rect,
                CornerRadius::same(elem_config.corner_radius as u8),
                Stroke::new(elem_config.border_width, elem_config.border_color32()),
                egui::StrokeKind::Inside,
            );
        }

        // Render actual module contents inside the rect
        render_module_by_id(&overlay_painter, id, rect, data, tick_counter);

        // Draw editor drag frame & corner handles around element
        let (frame_stroke, tag_col) = if is_selected {
            (Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0)), Color32::from_rgb(255, 215, 0))
        } else if is_hovered {
            (Stroke::new(1.0_f32, Color32::from_rgb(85, 255, 255)), Color32::from_rgb(85, 255, 255))
        } else {
            (Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(120, 130, 150, 90)), Color32::from_rgb(160, 165, 180))
        };

        overlay_painter.rect_stroke(rect, CornerRadius::same(2), frame_stroke, egui::StrokeKind::Outside);

        // Name tag at top-left of box
        overlay_painter.text(
            Pos2::new(rect.left(), rect.top() - 2.0),
            Align2::LEFT_BOTTOM,
            &elem_config.name,
            FontId::proportional(9.0),
            tag_col,
        );

        // Corner handles when selected
        if is_selected {
            let hs = 3.0;
            for corner in [rect.left_top(), rect.right_top(), rect.left_bottom(), rect.right_bottom()] {
                overlay_painter.rect_filled(
                    Rect::from_center_size(corner, Vec2::splat(hs * 2.0)),
                    CornerRadius::ZERO,
                    Color32::from_rgb(255, 215, 0),
                );
            }
        }
    }

    // 4. Floating Top Toolbar (Profile switcher, Grid Snapping, Reset, Exit)
    Window::new("HudEditorToolbar")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 14.0))
        .frame(
            Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(16, 18, 24, 245))
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

                // Grid Snapping
                ui.label("Grid:");
                for &(label, step) in &[("Free", 0), ("4px", 4), ("8px", 8), ("16px", 16), ("32px", 32)] {
                    if ui.selectable_label(layout.grid_snap_size == step, label).clicked() {
                        layout.grid_snap_size = step;
                    }
                }

                ui.separator();

                // Reset & Done buttons
                if ui.button("Reset Positions").clicked() {
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

    // 5. Property Inspector Window for the Selected Element
    if let Some(sel_id) = state.selected_id.clone() {
        if let Some(elem) = layout.get_element_mut(&sel_id) {
            Window::new(format!("Element: {}", elem.name))
                .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-16.0, -16.0))
                .resizable(false)
                .collapsible(true)
                .frame(
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(18, 22, 30, 245))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(70, 78, 95)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(14, 10)),
                )
                .show(ctx, |ui| {
                    ui.set_width(220.0);
                    ui.vertical(|ui| {
                        // Visibility Checkbox
                        ui.checkbox(&mut elem.visible, "Visible in HUD");

                        ui.add_space(4.0);

                        // Scale Slider (0.5 .. 2.0)
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(Slider::new(&mut elem.scale, 0.5..=2.0).step_by(0.05));
                        });

                        ui.add_space(4.0);

                        // Background Opacity
                        let mut bg_alpha = elem.bg_color[3];
                        ui.horizontal(|ui| {
                            ui.label("BG Alpha:");
                            if ui.add(Slider::new(&mut bg_alpha, 0..=255)).changed() {
                                elem.bg_color[3] = bg_alpha;
                            }
                        });

                        // Border Opacity
                        let mut border_alpha = elem.border_color[3];
                        ui.horizontal(|ui| {
                            ui.label("Border Alpha:");
                            if ui.add(Slider::new(&mut border_alpha, 0..=255)).changed() {
                                elem.border_color[3] = border_alpha;
                            }
                        });

                        ui.add_space(4.0);

                        // Anchor Selector
                        ui.horizontal(|ui| {
                            ui.label("Anchor:");
                            egui::ComboBox::from_id_salt("anchor_combo")
                                .selected_text(format!("{:?}", elem.anchor))
                                .show_ui(ui, |ui| {
                                    for a in &[
                                        HudAnchor::TopLeft,
                                        HudAnchor::TopCenter,
                                        HudAnchor::TopRight,
                                        HudAnchor::CenterLeft,
                                        HudAnchor::Center,
                                        HudAnchor::CenterRight,
                                        HudAnchor::BottomLeft,
                                        HudAnchor::BottomCenter,
                                        HudAnchor::BottomRight,
                                    ] {
                                        ui.selectable_value(&mut elem.anchor, *a, format!("{:?}", a));
                                    }
                                });
                        });

                        // Offset display
                        ui.horizontal(|ui| {
                            ui.label("Offset:");
                            ui.add(DragValue::new(&mut elem.offset[0]).prefix("X: "));
                            ui.add(DragValue::new(&mut elem.offset[1]).prefix("Y: "));
                        });

                        ui.add_space(6.0);
                        if ui.button("Deselect").clicked() {
                            state.selected_id = None;
                        }
                    });
                });
        }
    }

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
        "hotbar" => render_hotbar(painter, rect, data),
        "vitals" => render_vitals(painter, rect, data, tick_counter),
        "exp_bar" => render_exp_bar(painter, rect, data),
        "boss_bar" => render_boss_bar(painter, rect, data),
        "keystrokes" => render_keystrokes(painter, rect, data),
        "fps_ping" => render_fps_ping(painter, rect, data),
        "armor_status" => render_armor_status(painter, rect, data),
        "potion_effects" => render_potion_effects(painter, rect, data),
        "coords_direction" => render_coords_direction(painter, rect, data),
        _ => {}
    }
}
