use super::{KanjiCallback, format_date, format_kanji_entry};
use crate::dialogue::SessionData;
use crate::service::OrigaServiceProvider;
use origa::application::KanjiListUseCase;
use origa::domain::{Card, JapaneseLevel, KanjiInfo};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub async fn handle_kanji_list(
    bot: teloxide::Bot,
    msg: teloxide::types::Message,
    (page, items_per_page): (usize, usize),
    session: SessionData,
) -> teloxide::requests::ResponseResult<()> {
    let chat_id = msg.chat.id;
    let provider = OrigaServiceProvider::instance();

    let kanji_review_dates = fetch_kanji_review_dates(session.user_id, provider).await?;

    let kanji_list = get_kanji_by_level(None);
    let total_pages = (kanji_list.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(kanji_list.len());
    let page_kanji = &kanji_list[start..end];

    let text = build_kanji_list_text(
        page_kanji,
        current_page,
        total_pages,
        None,
        &kanji_review_dates,
    );
    let keyboard =
        build_kanji_list_keyboard(page_kanji, current_page, total_pages, &kanji_review_dates);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    teloxide::respond(())
}

pub fn get_kanji_by_level(level: Option<&JapaneseLevel>) -> Vec<KanjiInfo> {
    let use_case = KanjiListUseCase::new();

    if let Some(level) = level {
        use_case.execute(level).unwrap_or_default()
    } else {
        vec![
            JapaneseLevel::N5,
            JapaneseLevel::N4,
            JapaneseLevel::N3,
            JapaneseLevel::N2,
            JapaneseLevel::N1,
        ]
        .into_iter()
        .flat_map(|lvl| use_case.execute(&lvl).unwrap_or_default())
        .collect::<Vec<_>>()
    }
}

async fn fetch_kanji_review_dates(
    user_id: ulid::Ulid,
    provider: &OrigaServiceProvider,
) -> Result<HashMap<String, String>, teloxide::RequestError> {
    let use_case = provider.knowledge_set_cards_use_case();
    let cards = use_case
        .execute(user_id)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    let mut review_dates: HashMap<String, String> = HashMap::new();

    for card in cards {
        if let Card::Kanji(kanji_card) = card.card() {
            let kanji_char = kanji_card.kanji().text().to_string();
            let next_review = card.memory().next_review_date().map(format_date);
            if let Some(date_str) = next_review {
                review_dates.insert(kanji_char, date_str);
            }
        }
    }

    Ok(review_dates)
}

pub fn build_kanji_list_text(
    kanji_list: &[KanjiInfo],
    page: usize,
    total_pages: usize,
    level: Option<&JapaneseLevel>,
    review_dates: &HashMap<String, String>,
) -> String {
    let mut text = format!(
        "🈷 <b>Кандзи</b> — Уровень: {}\n\n",
        level.map(|l| l.code()).unwrap_or("Все")
    );

    for (idx, kanji) in kanji_list.iter().enumerate() {
        text.push_str(&format_kanji_entry(kanji, idx, page));

        let kanji_char = kanji.kanji().to_string();
        if let Some(next_review) = review_dates.get(&kanji_char) {
            text.push_str(&format!("   Повтор: {}\n", next_review));
        }

        text.push('\n');
    }

    text.push_str(&format!("\nСтраница {}/{}", page + 1, total_pages.max(1)));
    text
}

pub fn build_kanji_list_keyboard(
    kanji_list: &[KanjiInfo],
    page: usize,
    total_pages: usize,
    review_dates: &HashMap<String, String>,
) -> teloxide::types::InlineKeyboardMarkup {
    let mut rows: Vec<Vec<teloxide::types::InlineKeyboardButton>> = vec![];

    rows.push(vec![
        teloxide::types::InlineKeyboardButton::callback("N5", "kanji_level_N5"),
        teloxide::types::InlineKeyboardButton::callback("N4", "kanji_level_N4"),
        teloxide::types::InlineKeyboardButton::callback("N3", "kanji_level_N3"),
        teloxide::types::InlineKeyboardButton::callback("N2", "kanji_level_N2"),
        teloxide::types::InlineKeyboardButton::callback("N1", "kanji_level_N1"),
        teloxide::types::InlineKeyboardButton::callback("Все", "kanji_level_all"),
    ]);

    for kanji in kanji_list {
        let kanji_char = kanji.kanji().to_string();
        let is_studying = review_dates.contains_key(&kanji_char);
        let action_button = if is_studying {
            teloxide::types::InlineKeyboardButton::callback(
                "Удалить",
                KanjiCallback::Delete {
                    kanji: kanji_char.clone(),
                }
                .to_json(),
            )
        } else {
            teloxide::types::InlineKeyboardButton::callback(
                "Добавить",
                KanjiCallback::Add {
                    kanji: kanji_char.clone(),
                }
                .to_json(),
            )
        };

        rows.push(vec![
            teloxide::types::InlineKeyboardButton::callback(
                "Подробнее",
                KanjiCallback::Detail {
                    kanji: kanji_char.clone(),
                }
                .to_json(),
            ),
            action_button,
        ]);
    }

    let mut nav_row = vec![];
    if page > 0 {
        nav_row.push(teloxide::types::InlineKeyboardButton::callback(
            "⬅️ Назад",
            KanjiCallback::Page { page: page - 1 }.to_json(),
        ));
    }
    nav_row.push(teloxide::types::InlineKeyboardButton::callback(
        format!("{}/{}", page + 1, total_pages.max(1)),
        KanjiCallback::PageCurrent.to_json(),
    ));
    if page + 1 < total_pages {
        nav_row.push(teloxide::types::InlineKeyboardButton::callback(
            "Далее ➡️",
            KanjiCallback::Page { page: page + 1 }.to_json(),
        ));
    }
    if !nav_row.is_empty() {
        rows.push(nav_row);
    }

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "+ Добавить кандзи",
        KanjiCallback::AddNew.to_json(),
    )]);

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "🏠 Главная",
        KanjiCallback::MainMenu.to_json(),
    )]);

    teloxide::types::InlineKeyboardMarkup::new(rows)
}

pub async fn handle_kanji_list_by_level(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    level: Option<&JapaneseLevel>,
    page: usize,
    items_per_page: usize,
    user_id: ulid::Ulid,
) -> teloxide::requests::ResponseResult<()> {
    let provider = OrigaServiceProvider::instance();
    let kanji_review_dates = fetch_kanji_review_dates(user_id, provider).await?;

    let kanji_list = get_kanji_by_level(level);
    let total_pages = (kanji_list.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(kanji_list.len());
    let page_kanji = &kanji_list[start..end];

    let text = build_kanji_list_text(
        page_kanji,
        current_page,
        total_pages,
        level,
        &kanji_review_dates,
    );
    let keyboard =
        build_kanji_list_keyboard(page_kanji, current_page, total_pages, &kanji_review_dates);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    teloxide::respond(())
}
