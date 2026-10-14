use egui::{Button, Response, TextEdit, Ui};

/// Retrieves plain text from the system clipboard.
pub fn get_clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

/// Sets plain text in the system clipboard.
pub fn set_clipboard_text(text: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text);
    }
}

/// Renders a single-line text input with an integrated clipboard paste button ("📋").
/// Also supports native Ctrl+V keyboard pasting when focused.
pub fn text_input_with_paste(
    ui: &mut Ui,
    text: &mut String,
    hint: &str,
    width: Option<f32>,
) -> Response {
    ui.horizontal(|ui| {
        let mut text_edit = TextEdit::singleline(text).hint_text(hint);
        if let Some(w) = width {
            text_edit = text_edit.desired_width(w);
        }

        let mut response = ui.add(text_edit);

        // Check if user pressed Ctrl+V while the field is focused
        if response.has_focus() {
            ui.input(|i| {
                if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::V) {
                    if let Some(clip) = get_clipboard_text() {
                        text.push_str(&clip);
                        response.mark_changed();
                    }
                }
            });
        }

        // Paste button
        let paste_btn = ui
            .add(Button::new("📋").small())
            .on_hover_text("Вставить из буфера обмена");

        if paste_btn.clicked() {
            if let Some(clip) = get_clipboard_text() {
                *text = clip.trim().to_string();
                response.mark_changed();
            }
        }

        response
    })
    .inner
}
