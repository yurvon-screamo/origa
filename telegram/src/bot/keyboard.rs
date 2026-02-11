use crate::handlers::menu::MenuCallback;
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
            MenuCallback::HistoryKnown.to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История в процессе",
            MenuCallback::HistoryInProgress.to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История новых",
            MenuCallback::HistoryNew.to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История сложных",
            MenuCallback::HistoryHard.to_json(),
        )],
        vec![
            InlineKeyboardButton::callback("🎯 Урок", MenuCallback::Lesson.to_json()),
            InlineKeyboardButton::callback("🔒 Закрепление", MenuCallback::Fixation.to_json()),
        ],
        vec![
            InlineKeyboardButton::callback("📚 Слова", MenuCallback::Vocabulary.to_json()),
            InlineKeyboardButton::callback("🈷 Кандзи", MenuCallback::Kanji.to_json()),
            InlineKeyboardButton::callback("📖 Грамматика", MenuCallback::Grammar.to_json()),
        ],
        vec![
            InlineKeyboardButton::callback("👤 Профиль", MenuCallback::Profile.to_json()),
            InlineKeyboardButton::callback("⚙️ Настройки", MenuCallback::Settings.to_json()),
            InlineKeyboardButton::callback("🏠 Главная", MenuCallback::MainMenu.to_json()),
        ],
    ];

    InlineKeyboardMarkup::new(rows)
}

pub fn history_keyboard() -> InlineKeyboardMarkup {
    let keyboard = vec![vec![InlineKeyboardButton::callback(
        "История 📜",
        MenuCallback::ShowHistory.to_json(),
    )]];
    InlineKeyboardMarkup::new(keyboard)
}
