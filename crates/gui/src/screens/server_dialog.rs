use crate::clipboard::text_input_with_paste;
use crate::i18n::{t, Language};
use egui::{Button, Context, Vec2, Window};

pub struct ServerDialogState {
    pub name: String,
    pub ip: String,
    pub edit_index: Option<usize>,
}

impl Default for ServerDialogState {
    fn default() -> Self {
        Self {
            name: "Server".to_string(),
            ip: "localhost".to_string(),
            edit_index: None,
        }
    }
}

pub enum ServerDialogResult {
    Save {
        name: String,
        ip: String,
        edit_index: Option<usize>,
    },
    Cancel,
    None,
}

pub fn render_server_dialog(
    ctx: &Context,
    state: &mut ServerDialogState,
    is_direct_connect: bool,
    lang: Language,
) -> ServerDialogResult {
    let title = if is_direct_connect {
        t(lang, "dialog.direct_title")
    } else if state.edit_index.is_some() {
        t(lang, "dialog.edit_title")
    } else {
        t(lang, "dialog.add_title")
    };

    let mut result = ServerDialogResult::None;
    let mut is_open = true;

    Window::new(title)
        .collapsible(false)
        .resizable(false)
        .open(&mut is_open)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            if !is_direct_connect {
                ui.label(t(lang, "dialog.name_label"));
                text_input_with_paste(ui, &mut state.name, t(lang, "dialog.name_placeholder"), Some(260.0));
                ui.add_space(8.0);
            }

            ui.label(t(lang, "dialog.ip_label"));
            text_input_with_paste(ui, &mut state.ip, t(lang, "dialog.ip_placeholder"), Some(260.0));
            ui.add_space(14.0);

            let action_btn_text = if is_direct_connect {
                t(lang, "dialog.connect")
            } else {
                t(lang, "dialog.save")
            };

            let can_save = !state.ip.trim().is_empty() && (is_direct_connect || !state.name.trim().is_empty());

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_save, Button::new(action_btn_text).min_size(Vec2::new(120.0, 26.0)))
                    .clicked()
                {
                    result = ServerDialogResult::Save {
                        name: state.name.trim().to_string(),
                        ip: state.ip.trim().to_string(),
                        edit_index: state.edit_index,
                    };
                }

                if ui.add(Button::new(t(lang, "dialog.cancel")).min_size(Vec2::new(90.0, 26.0))).clicked() {
                    result = ServerDialogResult::Cancel;
                }
            });
        });

    if !is_open && matches!(result, ServerDialogResult::None) {
        result = ServerDialogResult::Cancel;
    }

    result
}
