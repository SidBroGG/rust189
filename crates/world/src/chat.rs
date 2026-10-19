use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSpan {
    pub text: String,
    pub color: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for ChatSpan {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: (255, 255, 255),
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// Strips all § formatting codes from a string.
pub fn strip_formatting(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next(); // skip next formatting character
        } else {
            out.push(c);
        }
    }
    out
}

/// Converts a § color code character to RGB (r, g, b).
pub fn color_code_to_rgb(code: char) -> (u8, u8, u8) {
    match code.to_ascii_lowercase() {
        '0' => (0, 0, 0),         // Black
        '1' => (0, 0, 170),       // Dark Blue
        '2' => (0, 170, 0),       // Dark Green
        '3' => (0, 170, 170),     // Dark Aqua
        '4' => (170, 0, 0),       // Dark Red
        '5' => (170, 0, 170),     // Dark Purple
        '6' => (255, 170, 0),     // Gold
        '7' => (170, 170, 170),   // Gray
        '8' => (85, 85, 85),      // Dark Gray
        '9' => (85, 85, 255),     // Blue
        'a' => (85, 255, 85),     // Green
        'b' => (85, 255, 255),    // Aqua
        'c' => (255, 85, 85),     // Red
        'd' => (255, 85, 255),    // Light Purple
        'e' => (255, 255, 85),    // Yellow
        'f' => (255, 255, 255),   // White
        _ => (255, 255, 255),
    }
}

/// Converts named color strings to § color codes.
pub fn name_to_color_code(name: &str) -> &'static str {
    match name {
        "black" => "§0",
        "dark_blue" => "§1",
        "dark_green" => "§2",
        "dark_aqua" => "§3",
        "dark_red" => "§4",
        "dark_purple" => "§5",
        "gold" => "§6",
        "gray" => "§7",
        "dark_gray" => "§8",
        "blue" => "§9",
        "green" => "§a",
        "aqua" => "§b",
        "red" => "§c",
        "light_purple" => "§d",
        "yellow" => "§e",
        "white" => "§f",
        _ => "§f",
    }
}

/// Parses a JSON chat payload or plain string into a § formatted string.
pub fn parse_chat_message(raw_json_or_text: &str) -> String {
    let trimmed = raw_json_or_text.trim();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') && !trimmed.starts_with('"') {
        return raw_json_or_text.to_string();
    }

    match serde_json::from_str::<Value>(trimmed) {
        Ok(val) => {
            let mut out = String::new();
            extract_text_recursive(&val, &mut out);
            if out.is_empty() {
                raw_json_or_text.to_string()
            } else {
                out
            }
        }
        Err(_) => raw_json_or_text.to_string(),
    }
}

fn extract_text_recursive(val: &Value, out: &mut String) {
    match val {
        Value::String(s) => {
            out.push_str(s);
        }
        Value::Object(map) => {
            if let Some(Value::String(color)) = map.get("color") {
                out.push_str(name_to_color_code(color));
            }
            if let Some(Value::Bool(true)) = map.get("bold") {
                out.push_str("§l");
            }
            if let Some(Value::Bool(true)) = map.get("italic") {
                out.push_str("§o");
            }
            if let Some(Value::Bool(true)) = map.get("underlined") {
                out.push_str("§n");
            }
            if let Some(Value::String(text)) = map.get("text") {
                out.push_str(text);
            }
            if let Some(Value::String(translate_key)) = map.get("translate") {
                let translated = resolve_translate_key(translate_key);
                let template = if !translated.is_empty() {
                    translated
                } else if let Some(stripped) = translate_key.strip_prefix("entity.Villager.") {
                    stripped
                } else if let Some(stripped) = translate_key.strip_prefix("container.") {
                    stripped
                } else {
                    translate_key.as_str()
                };

                if let Some(Value::Array(with_args)) = map.get("with") {
                    let mut res = template.to_string();
                    for (i, arg) in with_args.iter().enumerate() {
                        let mut arg_text = String::new();
                        extract_text_recursive(arg, &mut arg_text);
                        let spec1 = format!("%{}$s", i + 1);
                        let spec2 = format!("%{}s", i + 1);
                        if res.contains(&spec1) {
                            res = res.replace(&spec1, &arg_text);
                        } else if res.contains(&spec2) {
                            res = res.replace(&spec2, &arg_text);
                        } else if let Some(pos) = res.find("%s") {
                            res.replace_range(pos..pos + 2, &arg_text);
                        } else {
                            if !res.is_empty() {
                                res.push(' ');
                            }
                            res.push_str(&arg_text);
                        }
                    }
                    out.push_str(&res);
                } else {
                    out.push_str(template);
                }
            }
            if let Some(Value::Array(extra)) = map.get("extra") {
                for item in extra {
                    extract_text_recursive(item, out);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                extract_text_recursive(item, out);
            }
        }
        _ => {}
    }
}

/// Resolves standard Minecraft translation keys to human-readable Russian text.
pub fn resolve_translate_key(key: &str) -> &'static str {
    match key {
        "entity.Villager.armor" => "Оружейник",
        "entity.Villager.butcher" => "Мясник",
        "entity.Villager.cleric" => "Священник",
        "entity.Villager.farmer" => "Фермер",
        "entity.Villager.fisherman" => "Рыбак",
        "entity.Villager.fletcher" => "Лучник",
        "entity.Villager.leather" => "Кожевник",
        "entity.Villager.librarian" => "Библиотекарь",
        "entity.Villager.name" => "Житель",
        "entity.Villager.shepherd" => "Пастух",
        "entity.Villager.tool" => "Инструментальщик",
        "entity.Villager.weapon" => "Оружейник",
        "container.chest" => "Сундук",
        "container.chestDouble" => "Большой сундук",
        "container.enderchest" => "Сундук Края",
        "container.crafting" => "Верстак",
        "container.furnace" => "Печь",
        "container.enchant" => "Зачаровывание",
        "container.brewing" => "Зельеварка",
        "container.dispenser" => "Раздатчик",
        "container.dropper" => "Выбрасыватель",
        "container.hopper" => "Воронка",
        "container.beacon" => "Маяк",
        "container.repair" => "Наковальня",
        "container.inventory" => "Инвентарь",
        "entity.Horse.name" => "Лошадь",
        "entity.Donkey.name" => "Осёл",
        "entity.Mule.name" => "Мул",
        "gameMode.changed" => "Игровой режим обновлён",
        "chat.type.text" => "<%s> %s",
        "chat.type.emote" => "* %s %s",
        "multiplayer.player.joined" => "%s присоединился к игре",
        "multiplayer.player.left" => "%s покинул игру",
        _ => "",
    }
}

/// Parses a § formatted text string into colored text spans suitable for rendering.
pub fn parse_spans(formatted_text: &str) -> Vec<ChatSpan> {
    let mut spans = Vec::new();
    let mut current_span = ChatSpan::default();
    let mut chars = formatted_text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '§' {
            if let Some(code) = chars.next() {
                let code_lower = code.to_ascii_lowercase();
                if matches!(code_lower, '0'..='9' | 'a'..='f') {
                    if !current_span.text.is_empty() {
                        spans.push(current_span.clone());
                        current_span.text.clear();
                    }
                    current_span.color = color_code_to_rgb(code_lower);
                    current_span.bold = false;
                    current_span.italic = false;
                    current_span.underline = false;
                } else {
                    match code_lower {
                        'l' => current_span.bold = true,
                        'o' => current_span.italic = true,
                        'n' => current_span.underline = true,
                        'r' => {
                            if !current_span.text.is_empty() {
                                spans.push(current_span.clone());
                                current_span.text.clear();
                            }
                            current_span = ChatSpan::default();
                        }
                        _ => {}
                    }
                }
            }
        } else {
            current_span.text.push(c);
        }
    }

    if !current_span.text.is_empty() {
        spans.push(current_span);
    }

    if spans.is_empty() {
        spans.push(ChatSpan {
            text: formatted_text.to_string(),
            ..Default::default()
        });
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_json_parsing() {
        let json = r#"{"text":"Hello, ","color":"yellow","extra":[{"text":"world!","color":"green","bold":true}]}"#;
        let formatted = parse_chat_message(json);
        assert!(formatted.contains("Hello, "));
        assert!(formatted.contains("world!"));

        let clean = strip_formatting(&formatted);
        assert_eq!(clean, "Hello, world!");

        let spans = parse_spans(&formatted);
        assert!(!spans.is_empty());
        assert_eq!(spans[0].color, (255, 255, 85)); // yellow
    }
}
