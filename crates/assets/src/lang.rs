use crate::error::AssetError;
use std::collections::HashMap;
use std::path::Path;

/// Parser and storage for Minecraft `.lang` translation files (`key=value` format).
#[derive(Debug, Clone, Default)]
pub struct LanguageMap {
    translations: HashMap<String, String>,
}

impl LanguageMap {
    /// Create a new empty language map.
    pub fn new() -> Self {
        Self {
            translations: HashMap::new(),
        }
    }

    /// Parse `.lang` format string directly.
    /// Follows vanilla Minecraft `net.minecraft.util.StringTranslate`:
    /// - Skips empty lines and lines starting with `#`
    /// - Splits line by first `=` into key and value
    pub fn parse_str(content: &str) -> Self {
        let mut translations = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                translations.insert(key.trim().to_string(), value.to_string());
            }
        }

        Self { translations }
    }

    /// Load language mappings from a `.lang` file on disk.
    pub fn load_file(path: &Path) -> Result<Self, AssetError> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::parse_str(&content))
    }

    /// Retrieve raw translated string for key if present.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.translations.get(key).map(|s| s.as_str())
    }

    /// Returns the translated string, or the key itself if no translation is found.
    pub fn translate<'a>(&'a self, key: &'a str) -> &'a str {
        self.get(key).unwrap_or(key)
    }

    /// Formats translated string by substituting positional placeholders `%s`, `%1$s`, etc.
    pub fn translate_args(&self, key: &str, args: &[&str]) -> String {
        let template = self.translate(key);
        let mut result = template.to_string();

        for (i, &arg) in args.iter().enumerate() {
            let specifier1 = format!("%{}$s", i + 1);
            let specifier2 = format!("%{}s", i + 1);
            if result.contains(&specifier1) {
                result = result.replace(&specifier1, arg);
            } else if result.contains(&specifier2) {
                result = result.replace(&specifier2, arg);
            } else {
                // First unnumbered %s
                if let Some(pos) = result.find("%s") {
                    result.replace_range(pos..pos + 2, arg);
                }
            }
        }

        result
    }

    /// Merges another LanguageMap into self (overriding existing keys).
    pub fn merge(&mut self, other: LanguageMap) {
        self.translations.extend(other.translations);
    }

    /// Number of loaded translation keys.
    pub fn len(&self) -> usize {
        self.translations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.translations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lang_parsing() {
        let content = r#"
# Comment line
menu.singleplayer=Singleplayer
menu.multiplayer=Multiplayer
options.fov=FOV: %s
chat.type.text=<%s> %s
"#;
        let map = LanguageMap::parse_str(content);
        assert_eq!(map.translate("menu.singleplayer"), "Singleplayer");
        assert_eq!(map.translate("menu.multiplayer"), "Multiplayer");
        assert_eq!(map.translate("missing.key"), "missing.key");

        let fov_str = map.translate_args("options.fov", &["90"]);
        assert_eq!(fov_str, "FOV: 90");

        let chat_str = map.translate_args("chat.type.text", &["Steve", "Hello world!"]);
        assert_eq!(chat_str, "<Steve> Hello world!");
    }
}
