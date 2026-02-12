pub mod actions;
pub mod callback;
pub mod callbacks;
pub mod details;
pub mod list;

pub use callback::kanji_callback_handler;
pub use callbacks::KanjiCallback;
use chrono::Datelike;
pub use list::handle_kanji_list;

use origa::domain::KanjiInfo;

pub fn format_kanji_entry(kanji: &KanjiInfo, idx: usize, page: usize) -> String {
    let mut text = format!("{}. <b>{}</b>\n", page * 6 + idx + 1, kanji.kanji());
    text.push_str(&format!("   Значения: {}\n", kanji.description()));

    let radicals: Vec<String> = kanji
        .radicals()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    if !radicals.is_empty() {
        text.push_str(&format!("   Радикал: {}\n", radicals.join(", ")));
    }

    text.push('\n');
    text
}

pub fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let today = now.date_naive();
    let date_naive = date.date_naive();

    if date_naive == today {
        "сегодня".to_string()
    } else if date_naive == today + chrono::TimeDelta::days(1) {
        "завтра".to_string()
    } else if date_naive < today {
        "просрочено".to_string()
    } else {
        format!(
            "{}.{}.{}",
            date_naive.day(),
            date_naive.month(),
            date_naive.year()
        )
    }
}
