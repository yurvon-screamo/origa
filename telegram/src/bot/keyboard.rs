use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

pub fn reply_keyboard() -> KeyboardMarkup {
    let buttons: Vec<Vec<KeyboardButton>> = vec![
        vec![
            KeyboardButton::new("🎯 Урок"),
            KeyboardButton::new("🔒 Закрепление"),
        ],
        vec![
            KeyboardButton::new("📚 Слова"),
            KeyboardButton::new("🈷 Кандзи"),
            KeyboardButton::new("📖 Грамматика"),
        ],
        vec![
            KeyboardButton::new("👤 Профиль"),
            KeyboardButton::new("⚙️ Настройки"),
            KeyboardButton::new("🏠 Главная"),
        ],
    ];
    KeyboardMarkup::new(buttons)
}

pub fn main_menu_keyboard_with_stats() -> InlineKeyboardMarkup {
    let rows = vec![
        vec![InlineKeyboardButton::callback(
            "📜 История изучения",
            "history_known",
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История в процессе",
            "history_in_progress",
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История новых",
            "history_new",
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История сложных",
            "history_hard",
        )],
        vec![
            InlineKeyboardButton::callback("🎯 Урок", "menu_lesson"),
            InlineKeyboardButton::callback("🔒 Закрепление", "menu_fixation"),
        ],
        vec![
            InlineKeyboardButton::callback("📚 Слова", "menu_vocabulary"),
            InlineKeyboardButton::callback("🈷 Кандзи", "menu_kanji"),
            InlineKeyboardButton::callback("📖 Грамматика", "menu_grammar"),
        ],
        vec![
            InlineKeyboardButton::callback("👤 Профиль", "menu_profile"),
            InlineKeyboardButton::callback("⚙️ Настройки", "menu_settings"),
            InlineKeyboardButton::callback("🏠 Главная", "menu_home"),
        ],
    ];

    InlineKeyboardMarkup::new(rows)
}

pub fn history_keyboard() -> InlineKeyboardMarkup {
    let keyboard = vec![vec![InlineKeyboardButton::callback(
        "История 📜",
        "show_history",
    )]];
    InlineKeyboardMarkup::new(keyboard)
}
