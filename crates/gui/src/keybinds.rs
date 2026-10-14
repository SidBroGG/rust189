/// Categories for grouping keybindings in the settings UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCategory {
    Movement,
    Gameplay,
    Multiplayer,
    Display,
}

impl KeyCategory {
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Movement => "settings.controls.cat_movement",
            Self::Gameplay => "settings.controls.cat_gameplay",
            Self::Multiplayer => "settings.controls.cat_multiplayer",
            Self::Display => "settings.controls.cat_display",
        }
    }
}

/// Definition of a configurable keybind.
#[derive(Debug, Clone)]
pub struct KeyBindDef {
    pub id: &'static str,
    pub name_key: &'static str,
    pub category: KeyCategory,
    pub default_key: &'static str,
}

/// All registered keybindings in the client.
pub const ALL_KEYBINDS: &[KeyBindDef] = &[
    KeyBindDef {
        id: "forward",
        name_key: "key.forward",
        category: KeyCategory::Movement,
        default_key: "W",
    },
    KeyBindDef {
        id: "back",
        name_key: "key.back",
        category: KeyCategory::Movement,
        default_key: "S",
    },
    KeyBindDef {
        id: "left",
        name_key: "key.left",
        category: KeyCategory::Movement,
        default_key: "A",
    },
    KeyBindDef {
        id: "right",
        name_key: "key.right",
        category: KeyCategory::Movement,
        default_key: "D",
    },
    KeyBindDef {
        id: "jump",
        name_key: "key.jump",
        category: KeyCategory::Movement,
        default_key: "SPACE",
    },
    KeyBindDef {
        id: "sneak",
        name_key: "key.sneak",
        category: KeyCategory::Movement,
        default_key: "LSHIFT",
    },
    KeyBindDef {
        id: "sprint",
        name_key: "key.sprint",
        category: KeyCategory::Movement,
        default_key: "LCONTROL",
    },
    KeyBindDef {
        id: "drop_item",
        name_key: "key.drop_item",
        category: KeyCategory::Gameplay,
        default_key: "Q",
    },
    KeyBindDef {
        id: "inventory",
        name_key: "key.inventory",
        category: KeyCategory::Gameplay,
        default_key: "E",
    },
    KeyBindDef {
        id: "zoom",
        name_key: "key.zoom",
        category: KeyCategory::Gameplay,
        default_key: "C",
    },
    KeyBindDef {
        id: "chat",
        name_key: "key.chat",
        category: KeyCategory::Multiplayer,
        default_key: "T",
    },
    KeyBindDef {
        id: "player_list",
        name_key: "key.player_list",
        category: KeyCategory::Multiplayer,
        default_key: "TAB",
    },
    KeyBindDef {
        id: "toggle_perspective",
        name_key: "key.toggle_perspective",
        category: KeyCategory::Display,
        default_key: "F5",
    },
    KeyBindDef {
        id: "fullscreen_toggle",
        name_key: "key.fullscreen_toggle",
        category: KeyCategory::Display,
        default_key: "F11",
    },
    KeyBindDef {
        id: "toggle_fps",
        name_key: "key.toggle_fps",
        category: KeyCategory::Display,
        default_key: "F3",
    },
];

/// Converts any legacy or raw key name into the standardized uppercase token.
pub fn standardize_key(raw: &str) -> String {
    let trimmed = raw.trim();

    // Map common aliases and legacy format tokens
    match trimmed {
        "KeyW" | "w" | "W" => "W".to_string(),
        "KeyS" | "s" | "S" => "S".to_string(),
        "KeyA" | "a" | "A" => "A".to_string(),
        "KeyD" | "d" | "D" => "D".to_string(),
        "KeyQ" | "q" | "Q" => "Q".to_string(),
        "KeyE" | "e" | "E" => "E".to_string(),
        "KeyT" | "t" | "T" => "T".to_string(),
        "KeyC" | "c" | "C" => "C".to_string(),
        "KeyR" | "r" | "R" => "R".to_string(),
        "KeyF" | "f" | "F" => "F".to_string(),
        "KeyZ" | "z" | "Z" => "Z".to_string(),
        "KeyX" | "x" | "X" => "X".to_string(),
        "KeyV" | "v" | "V" => "V".to_string(),
        "KeyB" | "b" | "B" => "B".to_string(),
        "KeyG" | "g" | "G" => "G".to_string(),
        "KeyH" | "h" | "H" => "H".to_string(),
        "KeyJ" | "j" | "J" => "J".to_string(),
        "KeyK" | "k" | "K" => "K".to_string(),
        "KeyL" | "l" | "L" => "L".to_string(),
        "KeyY" | "y" | "Y" => "Y".to_string(),
        "KeyU" | "u" | "U" => "U".to_string(),
        "KeyI" | "i" | "I" => "I".to_string(),
        "KeyO" | "o" | "O" => "O".to_string(),
        "KeyP" | "p" | "P" => "P".to_string(),
        "KeyM" | "m" | "M" => "M".to_string(),
        "KeyN" | "n" | "N" => "N".to_string(),
        "Space" | "space" | "SPACE" => "SPACE".to_string(),
        "ShiftLeft" | "Shift" | "LShift" | "LSHIFT" | "SHIFT" => "LSHIFT".to_string(),
        "ShiftRight" | "RShift" | "RSHIFT" => "RSHIFT".to_string(),
        "ControlLeft" | "Control" | "LCtrl" | "LCONTROL" | "CTRL" | "CONTROL" => "LCONTROL".to_string(),
        "ControlRight" | "RCtrl" | "RCONTROL" => "RCONTROL".to_string(),
        "AltLeft" | "Alt" | "LAlt" | "LALT" | "ALT" => "LALT".to_string(),
        "AltRight" | "RAlt" | "RALT" => "RALT".to_string(),
        "SuperLeft" | "Super" | "LSuper" | "LSUPER" => "LSUPER".to_string(),
        "SuperRight" | "RSuper" | "RSUPER" => "RSUPER".to_string(),
        "CapsLock" | "Caps" | "CAPSLOCK" => "CAPSLOCK".to_string(),
        "Tab" | "tab" | "TAB" => "TAB".to_string(),
        "Escape" | "Esc" | "esc" | "ESCAPE" => "ESCAPE".to_string(),
        "Enter" | "Return" | "ENTER" => "ENTER".to_string(),
        "Backspace" | "BACKSPACE" => "BACKSPACE".to_string(),
        "Delete" | "Del" | "DELETE" => "DELETE".to_string(),
        "Insert" | "Ins" | "INSERT" => "INSERT".to_string(),
        "Home" | "HOME" => "HOME".to_string(),
        "End" | "END" => "END".to_string(),
        "PageUp" | "PAGEUP" => "PAGEUP".to_string(),
        "PageDown" | "PAGEDOWN" => "PAGEDOWN".to_string(),
        "ArrowUp" | "Up" | "UP" => "UP".to_string(),
        "ArrowDown" | "Down" | "DOWN" => "DOWN".to_string(),
        "ArrowLeft" | "Left" | "LEFT" => "LEFT".to_string(),
        "ArrowRight" | "Right" | "RIGHT" => "RIGHT".to_string(),
        "Mouse2" | "RightClick" | "BUTTON2" | "MOUSE2" => "MOUSE2".to_string(),
        "Mouse3" | "MiddleClick" | "BUTTON3" | "MOUSE3" => "MOUSE3".to_string(),
        "Mouse4" | "BUTTON4" | "MOUSE4" => "MOUSE4".to_string(),
        "Mouse5" | "BUTTON5" | "MOUSE5" => "MOUSE5".to_string(),
        "F1" | "f1" => "F1".to_string(),
        "F2" | "f2" => "F2".to_string(),
        "F3" | "f3" => "F3".to_string(),
        "F4" | "f4" => "F4".to_string(),
        "F5" | "f5" => "F5".to_string(),
        "F6" | "f6" => "F6".to_string(),
        "F7" | "f7" => "F7".to_string(),
        "F8" | "f8" => "F8".to_string(),
        "F9" | "f9" => "F9".to_string(),
        "F10" | "f10" => "F10".to_string(),
        "F11" | "f11" => "F11".to_string(),
        "F12" | "f12" => "F12".to_string(),
        other => {
            if other.starts_with("Key") && other.len() == 4 {
                other[3..].to_uppercase()
            } else {
                other.to_uppercase()
            }
        }
    }
}

/// Converts an egui `Key` event into our standardized key token.
pub fn egui_key_to_standard(key: egui::Key) -> Option<String> {
    match key {
        egui::Key::Escape => None, // Escape cancels rebinding
        egui::Key::Space => Some("SPACE".to_string()),
        egui::Key::Tab => Some("TAB".to_string()),
        egui::Key::Enter => Some("ENTER".to_string()),
        egui::Key::Backspace => Some("BACKSPACE".to_string()),
        egui::Key::Delete => Some("DELETE".to_string()),
        egui::Key::Insert => Some("INSERT".to_string()),
        egui::Key::Home => Some("HOME".to_string()),
        egui::Key::End => Some("END".to_string()),
        egui::Key::PageUp => Some("PAGEUP".to_string()),
        egui::Key::PageDown => Some("PAGEDOWN".to_string()),
        egui::Key::ArrowUp => Some("UP".to_string()),
        egui::Key::ArrowDown => Some("DOWN".to_string()),
        egui::Key::ArrowLeft => Some("LEFT".to_string()),
        egui::Key::ArrowRight => Some("RIGHT".to_string()),
        egui::Key::F1 => Some("F1".to_string()),
        egui::Key::F2 => Some("F2".to_string()),
        egui::Key::F3 => Some("F3".to_string()),
        egui::Key::F4 => Some("F4".to_string()),
        egui::Key::F5 => Some("F5".to_string()),
        egui::Key::F6 => Some("F6".to_string()),
        egui::Key::F7 => Some("F7".to_string()),
        egui::Key::F8 => Some("F8".to_string()),
        egui::Key::F9 => Some("F9".to_string()),
        egui::Key::F10 => Some("F10".to_string()),
        egui::Key::F11 => Some("F11".to_string()),
        egui::Key::F12 => Some("F12".to_string()),
        egui::Key::A => Some("A".to_string()),
        egui::Key::B => Some("B".to_string()),
        egui::Key::C => Some("C".to_string()),
        egui::Key::D => Some("D".to_string()),
        egui::Key::E => Some("E".to_string()),
        egui::Key::F => Some("F".to_string()),
        egui::Key::G => Some("G".to_string()),
        egui::Key::H => Some("H".to_string()),
        egui::Key::I => Some("I".to_string()),
        egui::Key::J => Some("J".to_string()),
        egui::Key::K => Some("K".to_string()),
        egui::Key::L => Some("L".to_string()),
        egui::Key::M => Some("M".to_string()),
        egui::Key::N => Some("N".to_string()),
        egui::Key::O => Some("O".to_string()),
        egui::Key::P => Some("P".to_string()),
        egui::Key::Q => Some("Q".to_string()),
        egui::Key::R => Some("R".to_string()),
        egui::Key::S => Some("S".to_string()),
        egui::Key::T => Some("T".to_string()),
        egui::Key::U => Some("U".to_string()),
        egui::Key::V => Some("V".to_string()),
        egui::Key::W => Some("W".to_string()),
        egui::Key::X => Some("X".to_string()),
        egui::Key::Y => Some("Y".to_string()),
        egui::Key::Z => Some("Z".to_string()),
        egui::Key::Num0 => Some("0".to_string()),
        egui::Key::Num1 => Some("1".to_string()),
        egui::Key::Num2 => Some("2".to_string()),
        egui::Key::Num3 => Some("3".to_string()),
        egui::Key::Num4 => Some("4".to_string()),
        egui::Key::Num5 => Some("5".to_string()),
        egui::Key::Num6 => Some("6".to_string()),
        egui::Key::Num7 => Some("7".to_string()),
        egui::Key::Num8 => Some("8".to_string()),
        egui::Key::Num9 => Some("9".to_string()),
        other => Some(standardize_key(&format!("{:?}", other))),
    }
}

/// Converts a standardized key token into an egui `Key` if representable in egui.
pub fn standard_token_to_egui_key(token: &str) -> Option<egui::Key> {
    match token.to_uppercase().as_str() {
        "F1" => Some(egui::Key::F1),
        "F2" => Some(egui::Key::F2),
        "F3" => Some(egui::Key::F3),
        "F4" => Some(egui::Key::F4),
        "F5" => Some(egui::Key::F5),
        "F6" => Some(egui::Key::F6),
        "F7" => Some(egui::Key::F7),
        "F8" => Some(egui::Key::F8),
        "F9" => Some(egui::Key::F9),
        "F10" => Some(egui::Key::F10),
        "F11" => Some(egui::Key::F11),
        "F12" => Some(egui::Key::F12),
        "SPACE" => Some(egui::Key::Space),
        "TAB" => Some(egui::Key::Tab),
        "ENTER" => Some(egui::Key::Enter),
        "ESCAPE" => Some(egui::Key::Escape),
        "BACKSPACE" => Some(egui::Key::Backspace),
        "DELETE" => Some(egui::Key::Delete),
        "INSERT" => Some(egui::Key::Insert),
        "HOME" => Some(egui::Key::Home),
        "END" => Some(egui::Key::End),
        "PAGEUP" => Some(egui::Key::PageUp),
        "PAGEDOWN" => Some(egui::Key::PageDown),
        "UP" => Some(egui::Key::ArrowUp),
        "DOWN" => Some(egui::Key::ArrowDown),
        "LEFT" => Some(egui::Key::ArrowLeft),
        "RIGHT" => Some(egui::Key::ArrowRight),
        "A" => Some(egui::Key::A),
        "B" => Some(egui::Key::B),
        "C" => Some(egui::Key::C),
        "D" => Some(egui::Key::D),
        "E" => Some(egui::Key::E),
        "F" => Some(egui::Key::F),
        "G" => Some(egui::Key::G),
        "H" => Some(egui::Key::H),
        "I" => Some(egui::Key::I),
        "J" => Some(egui::Key::J),
        "K" => Some(egui::Key::K),
        "L" => Some(egui::Key::L),
        "M" => Some(egui::Key::M),
        "N" => Some(egui::Key::N),
        "O" => Some(egui::Key::O),
        "P" => Some(egui::Key::P),
        "Q" => Some(egui::Key::Q),
        "R" => Some(egui::Key::R),
        "S" => Some(egui::Key::S),
        "T" => Some(egui::Key::T),
        "U" => Some(egui::Key::U),
        "V" => Some(egui::Key::V),
        "W" => Some(egui::Key::W),
        "X" => Some(egui::Key::X),
        "Y" => Some(egui::Key::Y),
        "Z" => Some(egui::Key::Z),
        "0" => Some(egui::Key::Num0),
        "1" => Some(egui::Key::Num1),
        "2" => Some(egui::Key::Num2),
        "3" => Some(egui::Key::Num3),
        "4" => Some(egui::Key::Num4),
        "5" => Some(egui::Key::Num5),
        "6" => Some(egui::Key::Num6),
        "7" => Some(egui::Key::Num7),
        "8" => Some(egui::Key::Num8),
        "9" => Some(egui::Key::Num9),
        _ => None,
    }
}

/// Converts a winit physical and logical key into our standardized key token.
/// Prioritizes physical scan-code for layout independence (e.g. WASD on Cyrillic layouts),
/// falling back to logical key if physical is unidentified or non-standard.
pub fn winit_key_to_standard(
    physical_key: &winit::keyboard::PhysicalKey,
    logical_key: &winit::keyboard::Key,
) -> Option<String> {
    // 1. Physical key (layout-independent)
    if let winit::keyboard::PhysicalKey::Code(code) = physical_key {
        let mapped = match code {
            // Modifiers
            winit::keyboard::KeyCode::ShiftLeft => Some("LSHIFT"),
            winit::keyboard::KeyCode::ShiftRight => Some("RSHIFT"),
            winit::keyboard::KeyCode::ControlLeft => Some("LCONTROL"),
            winit::keyboard::KeyCode::ControlRight => Some("RCONTROL"),
            winit::keyboard::KeyCode::AltLeft => Some("LALT"),
            winit::keyboard::KeyCode::AltRight => Some("RALT"),
            winit::keyboard::KeyCode::SuperLeft => Some("LSUPER"),
            winit::keyboard::KeyCode::SuperRight => Some("RSUPER"),

            // Functional / Controls
            winit::keyboard::KeyCode::CapsLock => Some("CAPSLOCK"),
            winit::keyboard::KeyCode::Tab => Some("TAB"),
            winit::keyboard::KeyCode::Space => Some("SPACE"),
            winit::keyboard::KeyCode::Enter => Some("ENTER"),
            winit::keyboard::KeyCode::NumpadEnter => Some("NUMPAD_ENTER"),
            winit::keyboard::KeyCode::Escape => Some("ESCAPE"),
            winit::keyboard::KeyCode::Backspace => Some("BACKSPACE"),
            winit::keyboard::KeyCode::Insert => Some("INSERT"),
            winit::keyboard::KeyCode::Delete => Some("DELETE"),
            winit::keyboard::KeyCode::Home => Some("HOME"),
            winit::keyboard::KeyCode::End => Some("END"),
            winit::keyboard::KeyCode::PageUp => Some("PAGEUP"),
            winit::keyboard::KeyCode::PageDown => Some("PAGEDOWN"),
            winit::keyboard::KeyCode::NumLock => Some("NUMLOCK"),
            winit::keyboard::KeyCode::ScrollLock => Some("SCROLLLOCK"),
            winit::keyboard::KeyCode::PrintScreen => Some("PRINTSCREEN"),
            winit::keyboard::KeyCode::Pause => Some("PAUSE"),

            // Arrows
            winit::keyboard::KeyCode::ArrowUp => Some("UP"),
            winit::keyboard::KeyCode::ArrowDown => Some("DOWN"),
            winit::keyboard::KeyCode::ArrowLeft => Some("LEFT"),
            winit::keyboard::KeyCode::ArrowRight => Some("RIGHT"),

            // Function keys
            winit::keyboard::KeyCode::F1 => Some("F1"),
            winit::keyboard::KeyCode::F2 => Some("F2"),
            winit::keyboard::KeyCode::F3 => Some("F3"),
            winit::keyboard::KeyCode::F4 => Some("F4"),
            winit::keyboard::KeyCode::F5 => Some("F5"),
            winit::keyboard::KeyCode::F6 => Some("F6"),
            winit::keyboard::KeyCode::F7 => Some("F7"),
            winit::keyboard::KeyCode::F8 => Some("F8"),
            winit::keyboard::KeyCode::F9 => Some("F9"),
            winit::keyboard::KeyCode::F10 => Some("F10"),
            winit::keyboard::KeyCode::F11 => Some("F11"),
            winit::keyboard::KeyCode::F12 => Some("F12"),
            winit::keyboard::KeyCode::F13 => Some("F13"),
            winit::keyboard::KeyCode::F14 => Some("F14"),
            winit::keyboard::KeyCode::F15 => Some("F15"),
            winit::keyboard::KeyCode::F16 => Some("F16"),
            winit::keyboard::KeyCode::F17 => Some("F17"),
            winit::keyboard::KeyCode::F18 => Some("F18"),
            winit::keyboard::KeyCode::F19 => Some("F19"),
            winit::keyboard::KeyCode::F20 => Some("F20"),
            winit::keyboard::KeyCode::F21 => Some("F21"),
            winit::keyboard::KeyCode::F22 => Some("F22"),
            winit::keyboard::KeyCode::F23 => Some("F23"),
            winit::keyboard::KeyCode::F24 => Some("F24"),

            // Standard Letters
            winit::keyboard::KeyCode::KeyA => Some("A"),
            winit::keyboard::KeyCode::KeyB => Some("B"),
            winit::keyboard::KeyCode::KeyC => Some("C"),
            winit::keyboard::KeyCode::KeyD => Some("D"),
            winit::keyboard::KeyCode::KeyE => Some("E"),
            winit::keyboard::KeyCode::KeyF => Some("F"),
            winit::keyboard::KeyCode::KeyG => Some("G"),
            winit::keyboard::KeyCode::KeyH => Some("H"),
            winit::keyboard::KeyCode::KeyI => Some("I"),
            winit::keyboard::KeyCode::KeyJ => Some("J"),
            winit::keyboard::KeyCode::KeyK => Some("K"),
            winit::keyboard::KeyCode::KeyL => Some("L"),
            winit::keyboard::KeyCode::KeyM => Some("M"),
            winit::keyboard::KeyCode::KeyN => Some("N"),
            winit::keyboard::KeyCode::KeyO => Some("O"),
            winit::keyboard::KeyCode::KeyP => Some("P"),
            winit::keyboard::KeyCode::KeyQ => Some("Q"),
            winit::keyboard::KeyCode::KeyR => Some("R"),
            winit::keyboard::KeyCode::KeyS => Some("S"),
            winit::keyboard::KeyCode::KeyT => Some("T"),
            winit::keyboard::KeyCode::KeyU => Some("U"),
            winit::keyboard::KeyCode::KeyV => Some("V"),
            winit::keyboard::KeyCode::KeyW => Some("W"),
            winit::keyboard::KeyCode::KeyX => Some("X"),
            winit::keyboard::KeyCode::KeyY => Some("Y"),
            winit::keyboard::KeyCode::KeyZ => Some("Z"),

            // Digits
            winit::keyboard::KeyCode::Digit0 => Some("0"),
            winit::keyboard::KeyCode::Digit1 => Some("1"),
            winit::keyboard::KeyCode::Digit2 => Some("2"),
            winit::keyboard::KeyCode::Digit3 => Some("3"),
            winit::keyboard::KeyCode::Digit4 => Some("4"),
            winit::keyboard::KeyCode::Digit5 => Some("5"),
            winit::keyboard::KeyCode::Digit6 => Some("6"),
            winit::keyboard::KeyCode::Digit7 => Some("7"),
            winit::keyboard::KeyCode::Digit8 => Some("8"),
            winit::keyboard::KeyCode::Digit9 => Some("9"),

            // Numpad
            winit::keyboard::KeyCode::Numpad0 => Some("NUMPAD0"),
            winit::keyboard::KeyCode::Numpad1 => Some("NUMPAD1"),
            winit::keyboard::KeyCode::Numpad2 => Some("NUMPAD2"),
            winit::keyboard::KeyCode::Numpad3 => Some("NUMPAD3"),
            winit::keyboard::KeyCode::Numpad4 => Some("NUMPAD4"),
            winit::keyboard::KeyCode::Numpad5 => Some("NUMPAD5"),
            winit::keyboard::KeyCode::Numpad6 => Some("NUMPAD6"),
            winit::keyboard::KeyCode::Numpad7 => Some("NUMPAD7"),
            winit::keyboard::KeyCode::Numpad8 => Some("NUMPAD8"),
            winit::keyboard::KeyCode::Numpad9 => Some("NUMPAD9"),
            winit::keyboard::KeyCode::NumpadAdd => Some("NUMPAD_ADD"),
            winit::keyboard::KeyCode::NumpadSubtract => Some("NUMPAD_SUB"),
            winit::keyboard::KeyCode::NumpadMultiply => Some("NUMPAD_MUL"),
            winit::keyboard::KeyCode::NumpadDivide => Some("NUMPAD_DIV"),
            winit::keyboard::KeyCode::NumpadDecimal => Some("NUMPAD_DECIMAL"),
            winit::keyboard::KeyCode::NumpadEqual => Some("NUMPAD_EQUALS"),

            // Symbols
            winit::keyboard::KeyCode::Minus => Some("MINUS"),
            winit::keyboard::KeyCode::Equal => Some("EQUALS"),
            winit::keyboard::KeyCode::BracketLeft => Some("LBRACKET"),
            winit::keyboard::KeyCode::BracketRight => Some("RBRACKET"),
            winit::keyboard::KeyCode::Backslash => Some("BACKSLASH"),
            winit::keyboard::KeyCode::Semicolon => Some("SEMICOLON"),
            winit::keyboard::KeyCode::Quote => Some("APOSTROPHE"),
            winit::keyboard::KeyCode::Backquote => Some("GRAVE"),
            winit::keyboard::KeyCode::Comma => Some("COMMA"),
            winit::keyboard::KeyCode::Period => Some("PERIOD"),
            winit::keyboard::KeyCode::Slash => Some("SLASH"),

            _ => None,
        };

        if let Some(m) = mapped {
            return Some(m.to_string());
        }
    }

    // 2. Logical key fallback
    match logical_key {
        winit::keyboard::Key::Named(named) => {
            let named_str = match named {
                winit::keyboard::NamedKey::Shift => "LSHIFT",
                winit::keyboard::NamedKey::Control => "LCONTROL",
                winit::keyboard::NamedKey::Alt => "LALT",
                winit::keyboard::NamedKey::Super => "LSUPER",
                winit::keyboard::NamedKey::CapsLock => "CAPSLOCK",
                winit::keyboard::NamedKey::Tab => "TAB",
                winit::keyboard::NamedKey::Space => "SPACE",
                winit::keyboard::NamedKey::Enter => "ENTER",
                winit::keyboard::NamedKey::Backspace => "BACKSPACE",
                winit::keyboard::NamedKey::Escape => "ESCAPE",
                winit::keyboard::NamedKey::Delete => "DELETE",
                winit::keyboard::NamedKey::Insert => "INSERT",
                winit::keyboard::NamedKey::Home => "HOME",
                winit::keyboard::NamedKey::End => "END",
                winit::keyboard::NamedKey::PageUp => "PAGEUP",
                winit::keyboard::NamedKey::PageDown => "PAGEDOWN",
                winit::keyboard::NamedKey::ArrowUp => "UP",
                winit::keyboard::NamedKey::ArrowDown => "DOWN",
                winit::keyboard::NamedKey::ArrowLeft => "LEFT",
                winit::keyboard::NamedKey::ArrowRight => "RIGHT",
                winit::keyboard::NamedKey::F1 => "F1",
                winit::keyboard::NamedKey::F2 => "F2",
                winit::keyboard::NamedKey::F3 => "F3",
                winit::keyboard::NamedKey::F4 => "F4",
                winit::keyboard::NamedKey::F5 => "F5",
                winit::keyboard::NamedKey::F6 => "F6",
                winit::keyboard::NamedKey::F7 => "F7",
                winit::keyboard::NamedKey::F8 => "F8",
                winit::keyboard::NamedKey::F9 => "F9",
                winit::keyboard::NamedKey::F10 => "F10",
                winit::keyboard::NamedKey::F11 => "F11",
                winit::keyboard::NamedKey::F12 => "F12",
                _ => "",
            };
            if !named_str.is_empty() {
                return Some(named_str.to_string());
            }
        }
        winit::keyboard::Key::Character(s) => {
            return Some(standardize_key(s.as_str()));
        }
        _ => {}
    }

    None
}

/// Converts a winit mouse button into our standardized key token.
/// Left click returns None so normal UI clicking does not bind mouse1 accidentally.
pub fn winit_mouse_to_standard(button: &winit::event::MouseButton) -> Option<String> {
    match button {
        winit::event::MouseButton::Right => Some("MOUSE2".to_string()),
        winit::event::MouseButton::Middle => Some("MOUSE3".to_string()),
        winit::event::MouseButton::Back => Some("MOUSE4".to_string()),
        winit::event::MouseButton::Forward => Some("MOUSE5".to_string()),
        winit::event::MouseButton::Other(n) => Some(format!("MOUSE{}", n)),
        winit::event::MouseButton::Left => None,
    }
}

/// Checks whether a winit mouse button matches a configured standard key code.
pub fn matches_winit_mouse(
    configured_key: &str,
    button: &winit::event::MouseButton,
) -> bool {
    let std_code = standardize_key(configured_key);
    if let Some(pressed) = winit_mouse_to_standard(button) {
        if pressed.eq_ignore_ascii_case(&std_code) {
            return true;
        }
    }
    false
}

/// Checks whether a winit keyboard event matches a configured standard key code.
/// Checks both physical scan-code (layout-independent WASD) and logical key.
pub fn matches_winit_key(
    configured_key: &str,
    logical_key: &winit::keyboard::Key,
    physical_key: &winit::keyboard::PhysicalKey,
) -> bool {
    let std_code = standardize_key(configured_key);
    if let Some(pressed) = winit_key_to_standard(physical_key, logical_key) {
        if pressed.eq_ignore_ascii_case(&std_code) {
            return true;
        }
        // Generic modifier matches (e.g. "SHIFT" matches either "LSHIFT" or "RSHIFT")
        if (std_code == "SHIFT" && (pressed == "LSHIFT" || pressed == "RSHIFT"))
            || (std_code == "CONTROL" && (pressed == "LCONTROL" || pressed == "RCONTROL"))
            || (std_code == "ALT" && (pressed == "LALT" || pressed == "RALT"))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardize_key() {
        assert_eq!(standardize_key("KeyW"), "W");
        assert_eq!(standardize_key("w"), "W");
        assert_eq!(standardize_key("ShiftLeft"), "LSHIFT");
        assert_eq!(standardize_key("ControlLeft"), "LCONTROL");
        assert_eq!(standardize_key("Space"), "SPACE");
        assert_eq!(standardize_key("F11"), "F11");
    }

    #[test]
    fn test_egui_key_to_standard() {
        assert_eq!(egui_key_to_standard(egui::Key::W), Some("W".to_string()));
        assert_eq!(egui_key_to_standard(egui::Key::Space), Some("SPACE".to_string()));
        assert_eq!(egui_key_to_standard(egui::Key::F11), Some("F11".to_string()));
        assert_eq!(egui_key_to_standard(egui::Key::Escape), None);
    }

    #[test]
    fn test_matches_winit_physical_and_logical() {
        let physical = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW);
        let logical = winit::keyboard::Key::Character("w".into());
        assert!(matches_winit_key("W", &logical, &physical));
        assert!(matches_winit_key("KeyW", &logical, &physical));

        let physical_f11 = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11);
        let logical_f11 = winit::keyboard::Key::Named(winit::keyboard::NamedKey::F11);
        assert!(matches_winit_key("F11", &logical_f11, &physical_f11));
    }

    #[test]
    fn test_winit_modifiers_and_functional() {
        // ShiftLeft
        let phys_shift = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ShiftLeft);
        let log_shift = winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift);
        assert_eq!(winit_key_to_standard(&phys_shift, &log_shift), Some("LSHIFT".to_string()));
        assert!(matches_winit_key("LSHIFT", &log_shift, &phys_shift));
        assert!(matches_winit_key("SHIFT", &log_shift, &phys_shift));

        // ControlLeft
        let phys_ctrl = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ControlLeft);
        let log_ctrl = winit::keyboard::Key::Named(winit::keyboard::NamedKey::Control);
        assert_eq!(winit_key_to_standard(&phys_ctrl, &log_ctrl), Some("LCONTROL".to_string()));
        assert!(matches_winit_key("LCONTROL", &log_ctrl, &phys_ctrl));
        assert!(matches_winit_key("CTRL", &log_ctrl, &phys_ctrl));

        // AltLeft & AltRight
        let phys_alt = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::AltLeft);
        let log_alt = winit::keyboard::Key::Named(winit::keyboard::NamedKey::Alt);
        assert_eq!(winit_key_to_standard(&phys_alt, &log_alt), Some("LALT".to_string()));
        assert!(matches_winit_key("LALT", &log_alt, &phys_alt));
        assert!(matches_winit_key("ALT", &log_alt, &phys_alt));

        let phys_ralt = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::AltRight);
        assert_eq!(winit_key_to_standard(&phys_ralt, &log_alt), Some("RALT".to_string()));

        // CapsLock
        let phys_caps = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::CapsLock);
        let log_caps = winit::keyboard::Key::Named(winit::keyboard::NamedKey::CapsLock);
        assert_eq!(winit_key_to_standard(&phys_caps, &log_caps), Some("CAPSLOCK".to_string()));
        assert!(matches_winit_key("CAPSLOCK", &log_caps, &phys_caps));

        // Tab
        let phys_tab = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Tab);
        let log_tab = winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab);
        assert_eq!(winit_key_to_standard(&phys_tab, &log_tab), Some("TAB".to_string()));
        assert!(matches_winit_key("TAB", &log_tab, &phys_tab));

        // Escape
        let phys_esc = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape);
        let log_esc = winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape);
        assert_eq!(winit_key_to_standard(&phys_esc, &log_esc), Some("ESCAPE".to_string()));
    }

    #[test]
    fn test_winit_mouse_buttons() {
        assert_eq!(winit_mouse_to_standard(&winit::event::MouseButton::Left), None);
        assert_eq!(winit_mouse_to_standard(&winit::event::MouseButton::Right), Some("MOUSE2".to_string()));
        assert_eq!(winit_mouse_to_standard(&winit::event::MouseButton::Middle), Some("MOUSE3".to_string()));
        assert_eq!(winit_mouse_to_standard(&winit::event::MouseButton::Back), Some("MOUSE4".to_string()));
        assert_eq!(winit_mouse_to_standard(&winit::event::MouseButton::Forward), Some("MOUSE5".to_string()));

        assert!(matches_winit_mouse("MOUSE2", &winit::event::MouseButton::Right));
        assert!(matches_winit_mouse("MOUSE3", &winit::event::MouseButton::Middle));
        assert!(matches_winit_mouse("MOUSE4", &winit::event::MouseButton::Back));
        assert!(matches_winit_mouse("MOUSE5", &winit::event::MouseButton::Forward));
    }
}
