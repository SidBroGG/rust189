use egui::{Color32, Margin, Response, TextEdit, Ui, Vec2};

/// Renders a single-line text input field styled like Minecraft 1.8.9.
pub fn minecraft_text_input(
    ui: &mut Ui,
    text: &mut String,
    hint_text: &str,
    size: Vec2,
) -> Response {
    let frame = egui::Frame::NONE
        .fill(Color32::from_rgb(0, 0, 0))
        .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(160, 160, 160)))
        .inner_margin(Margin::symmetric(4, 2));

    frame
        .show(ui, |ui| {
            ui.add_sized(
                size,
                TextEdit::singleline(text)
                    .hint_text(hint_text)
                    .text_color(Color32::from_rgb(255, 255, 255))
                    .font(egui::FontId::new(14.0, egui::FontFamily::Proportional))
                    .frame(false),
            )
        })
        .inner
}
