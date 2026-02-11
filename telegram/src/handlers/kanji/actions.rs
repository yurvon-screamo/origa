use crate::repository::OrigaServiceProvider;
use origa::domain::{KANJI_DICTIONARY, KanjiInfo};
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub async fn handle_kanji_add(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    kanji_char: &str,
    user_id: ulid::Ulid,
) -> teloxide::requests::ResponseResult<()> {
    let provider = OrigaServiceProvider::instance();
    let use_case = provider.create_kanji_card_use_case();

    match use_case
        .execute(user_id, vec![kanji_char.to_string()])
        .await
    {
        Ok(cards) => {
            if cards.is_empty() {
                bot.send_message(chat_id, "❌ Не удалось добавить кандзи")
                    .await?;
            } else {
                let msg = format!("✅ Кандзи \'{}\' добавлено в изучаемые", kanji_char);
                bot.send_message(chat_id, msg).await?;
            }
        }
        Err(_) => {
            bot.send_message(
                chat_id,
                format!(
                    "⚠️ Кандзи \'{}\' уже добавлено или произошла ошибка",
                    kanji_char
                ),
            )
            .await?;
        }
    }

    teloxide::respond(())
}

pub async fn handle_kanji_delete(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    kanji_char: &str,
    user_id: ulid::Ulid,
) -> teloxide::requests::ResponseResult<()> {
    let provider = OrigaServiceProvider::instance();
    let _cards_use_case = provider.knowledge_set_cards_use_case();
    let delete_use_case = provider.delete_kanji_card_use_case();

    delete_use_case
        .execute(user_id, kanji_char.to_string())
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    bot.send_message(
        chat_id,
        format!("✅ Кандзи \'{}\' удалено из изучаемых", kanji_char),
    )
    .await?;

    teloxide::respond(())
}

pub async fn handle_kanji_add_new(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
) -> teloxide::requests::ResponseResult<()> {
    let text = "🔍 Введите кандзи для поиска и добавления:\n\nНапример: 日 или 日本".to_string();

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![vec![
        teloxide::types::InlineKeyboardButton::callback("🏠 Главная", "menu_home"),
    ]]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    teloxide::respond(())
}

pub async fn handle_kanji_search(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    query: &str,
    page: usize,
) -> teloxide::requests::ResponseResult<()> {
    let search_chars: Vec<char> = query.chars().filter(|c| !c.is_whitespace()).collect();

    if search_chars.is_empty() {
        bot.send_message(chat_id, "❌ Введите хотя бы один символ для поиска")
            .await?;
        return teloxide::respond(());
    }

    let mut found_kanji: Vec<&KanjiInfo> = vec![];
    for char in &search_chars {
        if let Ok(info) = KANJI_DICTIONARY.get_kanji_info(&char.to_string()) {
            found_kanji.push(info);
        }
    }

    if found_kanji.is_empty() {
        bot.send_message(
            chat_id,
            format!("❌ Кандзи \'{}\' не найдены в словаре", query),
        )
        .await?;
        return teloxide::respond(());
    }

    let items_per_page = 5;
    let total_pages = found_kanji.len().div_ceil(items_per_page);
    let current_page = page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(found_kanji.len());
    let page_kanji = &found_kanji[start..end];

    let mut text = format!(
        "🔍 Результаты поиска для \'{}\'\n\n",
        search_chars.iter().collect::<String>()
    );
    for (idx, kanji) in page_kanji.iter().enumerate() {
        let num = current_page * items_per_page + idx + 1;
        text.push_str(&format!(
            "{}. <b>{}</b> — {}\n",
            num,
            kanji.kanji(),
            kanji.description()
        ));
    }

    let mut rows: Vec<Vec<teloxide::types::InlineKeyboardButton>> = vec![];

    for kanji in page_kanji {
        let kanji_char = kanji.kanji().to_string();
        rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
            format!("Добавить \'{}\'", kanji_char),
            format!("kanji_add_{}", kanji_char),
        )]);
    }

    let mut nav_row = vec![];
    if current_page > 0 {
        nav_row.push(teloxide::types::InlineKeyboardButton::callback(
            "⬅️",
            format!("kanji_search_page_{}_{}", current_page - 1, query),
        ));
    }
    if current_page + 1 < total_pages {
        nav_row.push(teloxide::types::InlineKeyboardButton::callback(
            "➡️",
            format!("kanji_search_page_{}_{}", current_page + 1, query),
        ));
    }
    if !nav_row.is_empty() {
        rows.push(nav_row);
    }

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "🏠 Главная",
        "menu_home",
    )]);

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(rows);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    teloxide::respond(())
}
