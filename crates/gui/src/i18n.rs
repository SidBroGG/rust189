use serde::{Deserialize, Serialize};

/// Supported client languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "ru_RU")]
    Russian,
    #[serde(rename = "en_US")]
    English,
}

impl Language {
    /// Returns the standard locale code (e.g. "ru_RU" or "en_US").
    pub fn code(&self) -> &'static str {
        match self {
            Self::Russian => "ru_RU",
            Self::English => "en_US",
        }
    }

    /// Returns human-readable language name for UI selection.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Russian => "Русский (ru_RU)",
            Self::English => "English (en_US)",
        }
    }

    /// Parses language from string code (tolerant to "ru", "ru_RU", "en", "en_US").
    pub fn from_code(code: &str) -> Self {
        let lower = code.to_lowercase();
        if lower.starts_with("en") {
            Self::English
        } else {
            Self::Russian
        }
    }
}

/// Translates a key for the given language.
pub fn t<'a>(lang: Language, key: &'a str) -> &'a str {
    match lang {
        Language::Russian => translate_ru(key),
        Language::English => translate_en(key),
    }
}

fn translate_ru<'a>(key: &'a str) -> &'a str {
    match key {
        // Top bar
        "top.nickname" => "Никнейм игрока:",
        "top.nickname_tooltip" => "Имя пользователя для оффлайн авторизации",
        "top.settings" => "⚙ Настройки",
        "top.language" => "🌐 Язык",

        // Main menu & Server list
        "menu.title" => "Сетевая игра / Список серверов",
        "menu.empty_servers" => "Список серверов пуст. Добавьте сервер или используйте прямое подключение.",
        "menu.pinging" => "Проверка соединения...",
        "menu.connection_failed" => "Не удалось подключиться",
        "menu.players" => "Игроки",
        "menu.ping_ms" => "мс",
        "menu.join_server" => "Войти на сервер",
        "menu.direct_connect" => "Прямое подключение",
        "menu.add_server" => "Добавить сервер",
        "menu.edit_server" => "Редактировать",
        "menu.delete_server" => "Удалить",
        "menu.refresh" => "Обновить",
        "menu.quit" => "Выйти из игры",

        // Server dialogs
        "dialog.add_title" => "➕ Добавить сервер",
        "dialog.edit_title" => "✏ Редактировать сервер",
        "dialog.direct_title" => "🌐 Прямое подключение",
        "dialog.name_placeholder" => "Например: Local Server",
        "dialog.ip_label" => "Адрес сервера (IP или домен):",
        "dialog.ip_placeholder" => "localhost:25565",
        "dialog.paste_tooltip" => "Вставить из буфера обмена (Ctrl+V)",
        "dialog.empty_ip_error" => "Адрес сервера не может быть пустым!",
        "dialog.save" => "Сохранить",
        "dialog.connect" => "Подключиться",
        "dialog.cancel" => "Отмена",

        // Settings window
        "settings.title" => "⚙ Настройки клиента",
        "settings.tab_video" => "🖥 Графика",
        "settings.tab_controls" => "⌨ Управление",
        "settings.tab_audio" => "🔊 Звук",
        "settings.tab_language" => "🌐 Язык",
        "settings.done" => "Готово",

        // Video tab
        "settings.video.fov" => "Угол обзора (FOV)",
        "settings.video.render_distance" => "Дальность прорисовки (Чанки)",
        "settings.video.max_fps" => "Макс. FPS",
        "settings.video.unlimited" => "Без ограничений",
        "settings.video.vsync" => "Вертикальная синхронизация (VSync)",
        "settings.video.fullscreen" => "Полноэкранный режим (F11)",
        "settings.video.view_bobbing" => "Покачивание камеры при ходьбе",
        "settings.video.gamma" => "Яркость",
        "settings.video.gui_scale" => "Масштаб интерфейса:",
        "settings.video.gui_scale_auto" => "Авто-масштаб (Auto)",
        "settings.video.gui_scale_custom" => "Пользовательский масштаб",
        "settings.video.antialiasing" => "Сглаживание (Anti-Aliasing):",
        "settings.video.antialiasing_off" => "Выкл",
        "settings.video.antialiasing_fxaa" => "FXAA (Быстрое)",
        "settings.video.antialiasing_high" => "FXAA High",

        // Controls tab
        "settings.controls.press_key" => "> Нажмите клавишу (Esc для отмены) <",
        "settings.controls.reset_defaults" => "Сбросить клавиши по умолчанию",
        "settings.controls.conflict_warning" => "⚠ Конфликт: клавиша используется в другом действии",
        "settings.controls.cat_movement" => "Движение",
        "settings.controls.cat_gameplay" => "Игровой процесс",
        "settings.controls.cat_multiplayer" => "Сетевая игра",
        "settings.controls.cat_display" => "Интерфейс и экран",

        // Key action names
        "key.forward" => "Движение вперед",
        "key.back" => "Движение назад",
        "key.left" => "Стрейф влево",
        "key.right" => "Стрейф вправо",
        "key.jump" => "Прыжок",
        "key.sneak" => "Приседание (Sneak)",
        "key.sprint" => "Спринт",
        "key.drop_item" => "Выбросить предмет",
        "key.inventory" => "Инвентарь",
        "key.chat" => "Открыть чат",
        "key.player_list" => "Список игроков (Tab)",
        "key.toggle_perspective" => "Смена вида (F5)",
        "key.zoom" => "Приближение (Zoom)",
        "key.fullscreen_toggle" => "Полноэкранный режим",
        "key.toggle_fps" => "Переключить FPS / телеметрию (F3)",

        // In-game escape menu
        "ingame.menu_title" => "Игровое меню",
        "ingame.back_to_game" => "Вернуться в игру",
        "ingame.options" => "⚙ Настройки...",
        "ingame.disconnect" => "Отключиться",
        "ingame.hide_fps" => "Скрыть телеметрию HUD (F3)",
        "ingame.show_fps" => "Показать телеметрию HUD (F3)",

        // Death screen
        "death.title" => "Вы погибли!",
        "death.score" => "Счёт:",
        "death.respawn" => "Возродиться",
        "death.title_screen" => "В главное меню",

        // Audio tab
        "settings.audio.master" => "Общая громкость",
        "settings.audio.music" => "Музыка",
        "settings.audio.blocks" => "Блоки",
        "settings.audio.players" => "Игроки",
        "settings.audio.hostile" => "Враждебные мобы",
        "settings.audio.ambient" => "Окружение / Климат",

        // Language tab
        "settings.lang.select_prompt" => "Выберите язык интерфейса:",
        "settings.lang.current" => "Текущий язык:",

        _ => key,
    }
}

fn translate_en<'a>(key: &'a str) -> &'a str {
    match key {
        // Top bar
        "top.nickname" => "Player Nickname:",
        "top.nickname_tooltip" => "Username for offline authentication profile",
        "top.settings" => "⚙ Settings",
        "top.language" => "🌐 Language",

        // Main menu & Server list
        "menu.title" => "Multiplayer / Server List",
        "menu.empty_servers" => "Server list is empty. Add a server or use direct connect.",
        "menu.pinging" => "Pinging server...",
        "menu.connection_failed" => "Cannot connect to server",
        "menu.players" => "Players",
        "menu.ping_ms" => "ms",
        "menu.join_server" => "Join Server",
        "menu.direct_connect" => "Direct Connect",
        "menu.add_server" => "Add Server",
        "menu.edit_server" => "Edit",
        "menu.delete_server" => "Delete",
        "menu.refresh" => "Refresh",
        "menu.quit" => "Quit Game",

        // Server dialogs
        "dialog.add_title" => "➕ Add Server",
        "dialog.edit_title" => "✏ Edit Server",
        "dialog.direct_title" => "🌐 Direct Connect",
        "dialog.name_label" => "Server Name:",
        "dialog.name_placeholder" => "e.g. Local Server",
        "dialog.ip_label" => "Server Address (IP or Domain):",
        "dialog.ip_placeholder" => "localhost:25565",
        "dialog.paste_tooltip" => "Paste from clipboard (Ctrl+V)",
        "dialog.empty_ip_error" => "Server address cannot be empty!",
        "dialog.save" => "Save",
        "dialog.connect" => "Connect",
        "dialog.cancel" => "Cancel",

        // Settings window
        "settings.title" => "⚙ Client Settings",
        "settings.tab_video" => "🖥 Video",
        "settings.tab_controls" => "⌨ Controls",
        "settings.tab_audio" => "🔊 Audio",
        "settings.tab_language" => "🌐 Language",
        "settings.done" => "Done",

        // Video tab
        "settings.video.fov" => "Field of View (FOV)",
        "settings.video.render_distance" => "Render Distance (Chunks)",
        "settings.video.max_fps" => "Max Framerate",
        "settings.video.unlimited" => "Unlimited",
        "settings.video.vsync" => "Vertical Sync (VSync)",
        "settings.video.fullscreen" => "Fullscreen Mode (F11)",
        "settings.video.view_bobbing" => "View Bobbing",
        "settings.video.gamma" => "Brightness",
        "settings.video.gui_scale" => "GUI Scale:",
        "settings.video.gui_scale_auto" => "Auto Scale",
        "settings.video.gui_scale_custom" => "Custom Scale",
        "settings.video.antialiasing" => "Anti-Aliasing:",
        "settings.video.antialiasing_off" => "Off",
        "settings.video.antialiasing_fxaa" => "FXAA (Fast)",
        "settings.video.antialiasing_high" => "FXAA High",

        // Controls tab
        "settings.controls.press_key" => "> Press any key (Esc to cancel) <",
        "settings.controls.reset_defaults" => "Reset Keys to Default",
        "settings.controls.conflict_warning" => "⚠ Conflict: key is already bound to another action",
        "settings.controls.cat_movement" => "Movement",
        "settings.controls.cat_gameplay" => "Gameplay",
        "settings.controls.cat_multiplayer" => "Multiplayer",
        "settings.controls.cat_display" => "Display & Interface",

        // Key action names
        "key.forward" => "Walk Forwards",
        "key.back" => "Walk Backwards",
        "key.left" => "Strafe Left",
        "key.right" => "Strafe Right",
        "key.jump" => "Jump",
        "key.sneak" => "Sneak",
        "key.sprint" => "Sprint",
        "key.drop_item" => "Drop Item",
        "key.inventory" => "Inventory",
        "key.chat" => "Open Chat",
        "key.player_list" => "List Players (Tab)",
        "key.toggle_perspective" => "Toggle Perspective (F5)",
        "key.zoom" => "Camera Zoom",
        "key.fullscreen_toggle" => "Toggle Fullscreen",
        "key.toggle_fps" => "Toggle FPS / HUD Telemetry (F3)",

        // In-game escape menu
        "ingame.menu_title" => "Game Menu",
        "ingame.back_to_game" => "Back to Game",
        "ingame.options" => "⚙ Options...",
        "ingame.disconnect" => "Disconnect",
        "ingame.hide_fps" => "Hide HUD Telemetry (F3)",
        "ingame.show_fps" => "Show HUD Telemetry (F3)",

        // Death screen
        "death.title" => "You died!",
        "death.score" => "Score:",
        "death.respawn" => "Respawn",
        "death.title_screen" => "Title Screen",

        // Audio tab
        "settings.audio.master" => "Master Volume",
        "settings.audio.music" => "Music",
        "settings.audio.blocks" => "Blocks",
        "settings.audio.players" => "Players",
        "settings.audio.hostile" => "Hostile Creatures",
        "settings.audio.ambient" => "Ambient / Environment",

        // Language tab
        "settings.lang.select_prompt" => "Select Interface Language:",
        "settings.lang.current" => "Current Language:",

        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_fallback() {
        assert_eq!(t(Language::Russian, "settings.done"), "Готово");
        assert_eq!(t(Language::English, "settings.done"), "Done");
        assert_eq!(t(Language::Russian, "unknown.key"), "unknown.key");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("ru_RU"), Language::Russian);
        assert_eq!(Language::from_code("ru"), Language::Russian);
        assert_eq!(Language::from_code("en_US"), Language::English);
        assert_eq!(Language::from_code("en"), Language::English);
    }
}
