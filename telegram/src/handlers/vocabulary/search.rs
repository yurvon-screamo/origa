use crate::handlers::vocabulary::list::fetch_vocabulary_cards;
use crate::telegram_domain::{DialogueState, SessionData};
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardMarkup, MessageId};
use ulid::Ulid;

pub async fn handle_vocabulary_search(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: &crate::handlers::OrigaDialogue,
    session: SessionData,
    search_query: &str,
    _page: usize,
    items_per_page: usize,
) -> ResponseResult<()> {
    let repository = crate::repository::build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let cards = fetch_vocabulary_cards(&repository, session.user_id).await?;
    let query_lower = search_query.to_lowercase();

    let filtered_cards: Vec<_> = cards
        .into_iter()
        .filter(|(_, card)| {
            let card_text = match card.card() {
                origa::domain::Card::Vocabulary(v) => {
                    format!("{} {}", v.word().text(), v.meaning().text())
                }
                _ => String::new(),
            };
            card_text.to_lowercase().contains(&query_lower)
        })
        .collect();

    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = 0;

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = build_search_results_text(search_query, current_page, total_pages, page_cards);
    let keyboard =
        build_search_results_keyboard(search_query, page_cards, current_page, total_pages);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularySearch {
            page: current_page,
            items_per_page,
            query: search_query.to_string(),
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

pub async fn handle_search_page_change(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    data: &str,
    dialogue: crate::handlers::OrigaDialogue,
    _session: SessionData,
) -> ResponseResult<()> {
    let parts: Vec<&str> = data.split('_').collect();
    if parts.len() < 4 {
        bot.send_message(chat_id, "❌ Ошибка формата данных")
            .await?;
        return respond(());
    }

    let new_page = match parts[3].parse::<usize>() {
        Ok(p) => p,
        Err(_) => {
            bot.send_message(chat_id, "❌ Неверный номер страницы")
                .await?;
            return respond(());
        }
    };

    let query = parts[4..].join("_");

    let repository = crate::repository::build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let cards = fetch_vocabulary_cards(&repository, Ulid::new()).await?;
    let query_lower = query.to_lowercase();

    let filtered_cards: Vec<_> = cards
        .into_iter()
        .filter(|(_, card)| {
            let card_text = match card.card() {
                origa::domain::Card::Vocabulary(v) => {
                    format!("{} {}", v.word().text(), v.meaning().text())
                }
                _ => String::new(),
            };
            card_text.to_lowercase().contains(&query_lower)
        })
        .collect();

    let items_per_page = 6;
    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = new_page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = build_search_results_text(&query, current_page, total_pages, page_cards);
    let keyboard = build_search_results_keyboard(&query, page_cards, current_page, total_pages);

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularySearch {
            page: current_page,
            items_per_page,
            query,
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

pub fn build_search_results_text(
    query: &str,
    current_page: usize,
    total_pages: usize,
    page_cards: &[(Ulid, origa::domain::StudyCard)],
) -> String {
    let mut text = format!("🔍 Результаты поиска: \"{}\"\n\n", query);

    if page_cards.is_empty() {
        text.push_str("Ничего не найдено. Попробуйте другой запрос.");
    } else {
        for (idx, (_, card)) in page_cards.iter().enumerate() {
            let num = current_page * 6 + idx + 1;
            text.push_str(&format_card_entry(num, card));
            text.push('\n');
        }
    }

    if total_pages > 0 {
        text.push_str(&format!("\nСтраница {}/{}", current_page + 1, total_pages));
    }

    text
}

fn format_card_entry(num: usize, card: &origa::domain::StudyCard) -> String {
    match card.card() {
        origa::domain::Card::Vocabulary(v) => {
            format!(
                "<b>{}.</b> {} — {}",
                num,
                v.word().text(),
                v.meaning().text()
            )
        }
        _ => String::from("Неизвестный тип карточки"),
    }
}

pub fn build_search_results_keyboard(
    query: &str,
    page_cards: &[(Ulid, origa::domain::StudyCard)],
    current_page: usize,
    total_pages: usize,
) -> InlineKeyboardMarkup {
    let mut rows = vec![];

    for (card_id, _) in page_cards {
        rows.push(vec![
            teloxide::types::InlineKeyboardButton::callback(
                "Подробнее",
                format!("vocab_detail_{}", card_id),
            ),
            teloxide::types::InlineKeyboardButton::callback(
                "Удалить 🗑️",
                format!("vocab_delete_{}", card_id),
            ),
        ]);
    }

    if total_pages > 1 {
        let mut pagination_row = vec![];

        if current_page > 0 {
            pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
                "⬅️ Назад",
                format!("vocab_search_page_{}_{}", current_page - 1, query),
            ));
        }

        pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
            format!("{}/{}", current_page + 1, total_pages),
            "vocab_search_current",
        ));

        if current_page < total_pages - 1 {
            pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
                "Далее ➡️",
                format!("vocab_search_page_{}_{}", current_page + 1, query),
            ));
        }

        rows.push(pagination_row);
    }

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "🔙 Назад к списку",
        "vocab_page_0",
    )]);

    InlineKeyboardMarkup::new(rows)
}
