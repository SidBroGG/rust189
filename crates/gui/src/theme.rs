use egui::{Color32, CornerRadius, Visuals};

/// Standard 16 Minecraft chat/formatting colors.
pub const COLOR_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
pub const COLOR_DARK_BLUE: Color32 = Color32::from_rgb(0, 0, 170);
pub const COLOR_DARK_GREEN: Color32 = Color32::from_rgb(0, 170, 0);
pub const COLOR_DARK_AQUA: Color32 = Color32::from_rgb(0, 170, 170);
pub const COLOR_DARK_RED: Color32 = Color32::from_rgb(170, 0, 0);
pub const COLOR_DARK_PURPLE: Color32 = Color32::from_rgb(170, 0, 170);
pub const COLOR_GOLD: Color32 = Color32::from_rgb(255, 170, 0);
pub const COLOR_GRAY: Color32 = Color32::from_rgb(170, 170, 170);
pub const COLOR_DARK_GRAY: Color32 = Color32::from_rgb(85, 85, 85);
pub const COLOR_BLUE: Color32 = Color32::from_rgb(85, 85, 255);
pub const COLOR_GREEN: Color32 = Color32::from_rgb(85, 255, 85);
pub const COLOR_AQUA: Color32 = Color32::from_rgb(85, 255, 255);
pub const COLOR_RED: Color32 = Color32::from_rgb(255, 85, 85);
pub const COLOR_LIGHT_PURPLE: Color32 = Color32::from_rgb(255, 85, 255);
pub const COLOR_YELLOW: Color32 = Color32::from_rgb(255, 255, 85);
pub const COLOR_WHITE: Color32 = Color32::from_rgb(255, 255, 255);

/// Returns the egui Color32 for a Minecraft section color code character.
pub fn color_from_code(c: char) -> Option<Color32> {
    match c {
        '0' => Some(COLOR_BLACK),
        '1' => Some(COLOR_DARK_BLUE),
        '2' => Some(COLOR_DARK_GREEN),
        '3' => Some(COLOR_DARK_AQUA),
        '4' => Some(COLOR_DARK_RED),
        '5' => Some(COLOR_DARK_PURPLE),
        '6' => Some(COLOR_GOLD),
        '7' => Some(COLOR_GRAY),
        '8' => Some(COLOR_DARK_GRAY),
        '9' => Some(COLOR_BLUE),
        'a' | 'A' => Some(COLOR_GREEN),
        'b' | 'B' => Some(COLOR_AQUA),
        'c' | 'C' => Some(COLOR_RED),
        'd' | 'D' => Some(COLOR_LIGHT_PURPLE),
        'e' | 'E' => Some(COLOR_YELLOW),
        'f' | 'F' => Some(COLOR_WHITE),
        _ => None,
    }
}

/// Text styling flags matching standard section symbol formatting codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormattedStyle {
    pub color: Color32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub obfuscated: bool,
}

impl Default for FormattedStyle {
    fn default() -> Self {
        Self {
            color: COLOR_WHITE,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            obfuscated: false,
        }
    }
}

/// A parsed text segment containing styled text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedSegment {
    pub style: FormattedStyle,
    pub text: String,
}

/// Parses formatted text into detailed styled segments (colors, bold, strikethrough, underline, italic, reset).
pub fn parse_formatted_segments(text: &str) -> Vec<FormattedSegment> {
    let mut segments = Vec::new();
    let mut current_style = FormattedStyle::default();
    let mut current_text = String::new();

    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '§' || ch == '&' {
            if let Some(&next) = chars.peek() {
                let lower = next.to_ascii_lowercase();
                if let Some(col) = color_from_code(lower) {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    // Color code resets fancy styling as per standard behavior
                    current_style.color = col;
                    current_style.bold = false;
                    current_style.italic = false;
                    current_style.underline = false;
                    current_style.strikethrough = false;
                    current_style.obfuscated = false;
                    continue;
                } else if lower == 'r' {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    current_style = FormattedStyle::default();
                    continue;
                } else if lower == 'k' {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    current_style.obfuscated = true;
                    continue;
                } else if lower == 'l' {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    current_style.bold = true;
                    continue;
                } else if lower == 'm' {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    current_style.strikethrough = true;
                    continue;
                } else if lower == 'n' {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    current_style.underline = true;
                    continue;
                } else if lower == 'o' {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(FormattedSegment {
                            style: current_style,
                            text: std::mem::take(&mut current_text),
                        });
                    }
                    current_style.italic = true;
                    continue;
                }
            }
        }
        current_text.push(ch);
    }

    if !current_text.is_empty() {
        segments.push(FormattedSegment {
            style: current_style,
            text: current_text,
        });
    }

    if segments.is_empty() {
        vec![FormattedSegment {
            style: FormattedStyle::default(),
            text: String::new(),
        }]
    } else {
        segments
    }
}

/// Parses color formatted text (§0 - §f and &0 - &f) into colored segments, stripping all formatting tokens.
pub fn parse_minecraft_text(text: &str) -> Vec<(Color32, String)> {
    parse_formatted_segments(text)
        .into_iter()
        .map(|seg| (seg.style.color, seg.text))
        .collect()
}

/// Builds an egui LayoutJob from formatted text with custom font size and optional alpha override (monospace by default).
pub fn build_layout_job(text: &str, font_size: f32, alpha_override: Option<u8>) -> egui::text::LayoutJob {
    build_layout_job_styled(text, font_size, true, alpha_override)
}

/// Builds an egui LayoutJob from formatted text with custom font size, selectable font family, and optional alpha override.
pub fn build_layout_job_styled(
    text: &str,
    font_size: f32,
    monospace: bool,
    alpha_override: Option<u8>,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for seg in parse_formatted_segments(text) {
        if seg.text.is_empty() {
            continue;
        }
        let col = if let Some(alpha) = alpha_override {
            Color32::from_rgba_unmultiplied(seg.style.color.r(), seg.style.color.g(), seg.style.color.b(), alpha)
        } else {
            seg.style.color
        };

        let font_id = if monospace {
            if seg.style.bold {
                egui::FontId::monospace(font_size * 1.08)
            } else {
                egui::FontId::monospace(font_size)
            }
        } else if seg.style.bold {
            egui::FontId::proportional(font_size * 1.08)
        } else {
            egui::FontId::proportional(font_size)
        };

        let format = egui::TextFormat {
            font_id,
            color: col,
            italics: seg.style.italic,
            underline: if seg.style.underline {
                egui::Stroke::new(1.0f32, col)
            } else {
                egui::Stroke::NONE
            },
            strikethrough: if seg.style.strikethrough {
                egui::Stroke::new(1.0f32, col)
            } else {
                egui::Stroke::NONE
            },
            ..Default::default()
        };

        job.append(&seg.text, 0.0, format);
    }
    job
}


/// Configures standard clean egui style and default fonts.
pub fn setup_minecraft_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Standard modern egui rounded corners
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);
    visuals.widgets.open.corner_radius = CornerRadius::same(4);
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(4);

    // Modern clean dark theme panels
    visuals.panel_fill = Color32::from_rgb(24, 25, 28);
    visuals.window_fill = Color32::from_rgb(32, 34, 38);

    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minecraft_text() {
        let text = "§aGame §7Network §e[1.8.9]";
        let segments = parse_minecraft_text(text);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], (COLOR_GREEN, "Game ".to_string()));
        assert_eq!(segments[1], (COLOR_GRAY, "Network ".to_string()));
        assert_eq!(segments[2], (COLOR_YELLOW, "[1.8.9]".to_string()));
    }

    #[test]
    fn test_color_from_code() {
        assert_eq!(color_from_code('c'), Some(COLOR_RED));
        assert_eq!(color_from_code('e'), Some(COLOR_YELLOW));
        assert_eq!(color_from_code('z'), None);
    }

    #[test]
    fn test_parse_formatted_segments() {
        let text = "§8§m---§r §c§lImportant!§r §nUnderlined§r §oItalic";
        let segments = parse_formatted_segments(text);
        assert_eq!(segments.len(), 7);

        // Segment 0: dark gray + strikethrough
        assert_eq!(segments[0].text, "---");
        assert_eq!(segments[0].style.color, COLOR_DARK_GRAY);
        assert!(segments[0].style.strikethrough);

        // Segment 1: reset white space
        assert_eq!(segments[1].text, " ");
        assert_eq!(segments[1].style.color, COLOR_WHITE);
        assert!(!segments[1].style.strikethrough);

        // Segment 2: red + bold
        assert_eq!(segments[2].text, "Important!");
        assert_eq!(segments[2].style.color, COLOR_RED);
        assert!(segments[2].style.bold);

        // Segment 3: reset space
        assert_eq!(segments[3].text, " ");

        // Segment 4: underline
        assert_eq!(segments[4].text, "Underlined");
        assert!(segments[4].style.underline);

        // Segment 5: reset space
        assert_eq!(segments[5].text, " ");

        // Segment 6: italic
        assert_eq!(segments[6].text, "Italic");
        assert!(segments[6].style.italic);
    }
}
