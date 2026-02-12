use crate::handlers::callbacks::CallbackData;
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
            CallbackData::Menu(MenuCallback::HistoryKnown).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История в процессе",
            CallbackData::Menu(MenuCallback::HistoryInProgress).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История новых",
            CallbackData::Menu(MenuCallback::HistoryNew).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "📜 История сложных",
            CallbackData::Menu(MenuCallback::HistoryHard).to_json(),
        )],
        vec![
            InlineKeyboardButton::callback(
                "🎯 Урок",
                CallbackData::Menu(MenuCallback::Lesson).to_json(),
            ),
            InlineKeyboardButton::callback(
                "🔒 Закрепление",
                CallbackData::Menu(MenuCallback::Fixation).to_json(),
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "📚 Слова",
                CallbackData::Menu(MenuCallback::Vocabulary).to_json(),
            ),
            InlineKeyboardButton::callback(
                "🈷 Кандзи",
                CallbackData::Menu(MenuCallback::Kanji).to_json(),
            ),
            InlineKeyboardButton::callback(
                "📖 Грамматика",
                CallbackData::Menu(MenuCallback::Grammar).to_json(),
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "👤 Профиль",
                CallbackData::Menu(MenuCallback::Profile).to_json(),
            ),
            InlineKeyboardButton::callback(
                "⚙️ Настройки",
                CallbackData::Menu(MenuCallback::Settings).to_json(),
            ),
            InlineKeyboardButton::callback(
                "🏠 Главная",
                CallbackData::Menu(MenuCallback::MainMenu).to_json(),
            ),
        ],
    ];

    InlineKeyboardMarkup::new(rows)
}

pub fn history_keyboard() -> InlineKeyboardMarkup {
    let keyboard = vec![vec![InlineKeyboardButton::callback(
        "История 📜",
        CallbackData::Menu(MenuCallback::ShowHistory).to_json(),
    )]];
    InlineKeyboardMarkup::new(keyboard)
}
