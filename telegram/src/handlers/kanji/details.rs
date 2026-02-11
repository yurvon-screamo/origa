use super::{extract_kunyomi, extract_onyomi};
use origa::domain::{KANJI_DICTIONARY, KanjiInfo};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub async fn handle_kanji_detail(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    kanji_char: &str,
) -> teloxide::requests::ResponseResult<()> {
    let kanji_info = KANJI_DICTIONARY.get_kanji_info(kanji_char);

    let text = match kanji_info {
        Ok(info) => build_kanji_detail_text(info),
        Err(_) => format!("❌ Кандзи '{}' не найдено в словаре", kanji_char),
    };

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback(
                "Добавить",
                format!("kanji_add_{}", kanji_char),
            ),
            teloxide::types::InlineKeyboardButton::callback("Назад", "kanji_back_to_list"),
        ],
        vec![teloxide::types::InlineKeyboardButton::callback(
            "+ Добавить из списка",
            "kanji_add_new",
        )],
        vec![teloxide::types::InlineKeyboardButton::callback(
            "🏠 Главная",
            "menu_home",
        )],
    ]);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    teloxide::respond(())
}

pub fn build_kanji_detail_text(kanji: &KanjiInfo) -> String {
    let mut text = format!("<b>{}</b>\n\n", kanji.kanji());
    text.push_str(&format!("📚 Уровень: {}\n", kanji.jlpt().code()));
    text.push_str(&format!(
        "🔢 Используется в словах: {}\n\n",
        kanji.used_in()
    ));
    text.push_str(&format!("📝 Значения: {}\n", kanji.description()));

    let onyomi = extract_onyomi(kanji.description());
    let kunyomi = extract_kunyomi(kanji.description());

    if !onyomi.is_empty() {
        text.push_str(&format!("\n🔊 Онъёми: {}\n", onyomi.join(", ")));
    }
    if !kunyomi.is_empty() {
        text.push_str(&format!("🔉 Кунъёми: {}\n", kunyomi.join(", ")));
    }

    let radicals: Vec<String> = kanji
        .radicals()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    if !radicals.is_empty() {
        text.push_str(&format!("\n⛩ Радикалы: {}\n", radicals.join(", ")));
    }

    if !kanji.popular_words().is_empty() {
        text.push_str("\n📖 Популярные слова:\n");
        for word in kanji.popular_words().iter().take(5) {
            text.push_str(&format!("  • {}\n", word));
        }
    }

    text
}
