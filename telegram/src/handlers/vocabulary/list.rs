use crate::dialogue::{DialogueState, SessionData};
use crate::handlers::callbacks::CallbackData;
use crate::handlers::vocabulary::VocabularyCallback;
use crate::service::OrigaServiceProvider;
use chrono::{Datelike, TimeDelta};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;
use ulid::Ulid;

pub async fn vocabulary_list_handler(
    bot: Bot,
    msg: Message,
    dialogue: crate::handlers::OrigaDialogue,
    (page, items_per_page, filter): (usize, usize, String),
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let cards = fetch_vocabulary_cards(provider, session.user_id).await?;
    let filtered_cards = apply_filter(&cards, &filter);

    let total_cards = cards.len();
    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = build_vocabulary_text(total_cards, &filter, page_cards, current_page, total_pages);
    let keyboard = build_vocabulary_keyboard(&filter, page_cards, current_page, total_pages);

    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularyList {
            page: current_page,
            items_per_page,
            filter,
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

pub async fn fetch_vocabulary_cards(
    provider: &OrigaServiceProvider,
    user_id: Ulid,
) -> Result<Vec<(Ulid, origa::domain::StudyCard)>, teloxide::RequestError> {
    let use_case = provider.knowledge_set_cards_use_case();
    let cards = use_case.execute(user_id).await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    Ok(cards
        .into_iter()
        .filter(|c| matches!(c.card(), origa::domain::Card::Vocabulary(_)))
        .map(|c| (*c.card_id(), c))
        .collect())
}

pub fn apply_filter(
    cards: &[(Ulid, origa::domain::StudyCard)],
    filter: &str,
) -> Vec<(Ulid, origa::domain::StudyCard)> {
    cards
        .iter()
        .filter(|(_, c)| match filter {
            "new" => c.memory().is_new(),
            "in_progress" => c.memory().is_in_progress(),
            "hard" => c.memory().is_high_difficulty(),
            "known" => c.memory().is_known_card(),
            _ => true,
        })
        .cloned()
        .collect()
}

pub fn build_vocabulary_text(
    total_cards: usize,
    filter: &str,
    page_cards: &[(Ulid, origa::domain::StudyCard)],
    current_page: usize,
    total_pages: usize,
) -> String {
    let filter_name = match filter {
        "new" => "Новые",
        "in_progress" => "В процессе",
        "hard" => "Сложные",
        "known" => "Изученные",
        _ => "Все",
    };

    let mut text = format!("📚 Слова (всего: {})\n", total_cards);
    text.push_str(&format!("Фильтр: {}\n\n", filter_name));

    if total_pages > 0 {
        text.push_str(&format!(
            "Страница {}/{}\n\n",
            current_page + 1,
            total_pages
        ));
    }

    if page_cards.is_empty() {
        text.push_str("Нет карточек по выбранному фильтру.");
    } else {
        for (idx, (_, card)) in page_cards.iter().enumerate() {
            let num = current_page * 6 + idx + 1;
            text.push_str(&format_card_entry(num, card));
            text.push('\n');
        }
    }

    text
}

fn format_card_entry(num: usize, card: &origa::domain::StudyCard) -> String {
    let card_info = match card.card() {
        origa::domain::Card::Vocabulary(v) => {
            format!("{} — {}", v.word().text(), v.meaning().text())
        }
        _ => String::from("Неизвестный тип карточки"),
    };

    let memory = card.memory();

    let next_review = memory
        .next_review_date()
        .map(format_date)
        .unwrap_or_else(|| "сегодня".to_string());

    let difficulty = memory
        .difficulty()
        .map(|d| format!("{:.1}", d.value()))
        .unwrap_or_else(|| "-".to_string());

    let stability = memory
        .stability()
        .map(|s| format!("{:.0} дней", s.value()))
        .unwrap_or_else(|| "-".to_string());

    let status = if memory.is_new() {
        "Новая"
    } else if memory.is_high_difficulty() {
        "Сложная"
    } else if memory.is_known_card() {
        "Изучена"
    } else if memory.is_in_progress() {
        "В процессе"
    } else {
        "Неизвестно"
    };

    format!(
        "<b>{}.</b> {}\n   Повтор: {}\n   Сложность: {} | Стабильность: {}\n   Теги: {}",
        num, card_info, next_review, difficulty, stability, status
    )
}

fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let today = now.date_naive();
    let date_naive = date.date_naive();

    if date_naive == today {
        "сегодня".to_string()
    } else if date_naive == today + TimeDelta::days(1) {
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

pub fn build_vocabulary_keyboard(
    current_filter: &str,
    page_cards: &[(Ulid, origa::domain::StudyCard)],
    current_page: usize,
    total_pages: usize,
) -> InlineKeyboardMarkup {
    let mut rows = vec![];

    let filters = vec![
        ("Все", "all"),
        ("Новые", "new"),
        ("В процессе", "in_progress"),
        ("Сложные", "hard"),
        ("Изученные", "known"),
    ];

    let filter_row: Vec<_> = filters
        .into_iter()
        .map(|(label, value)| {
            let label_text = if value == current_filter {
                format!("[{}]", label)
            } else {
                label.to_string()
            };
            teloxide::types::InlineKeyboardButton::callback(
                label_text,
                CallbackData::Vocabulary(VocabularyCallback::Filter {
                    filter: value.to_string(),
                })
                .to_json(),
            )
        })
        .collect();
    rows.push(filter_row);

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "🔍 Поиск",
        CallbackData::Vocabulary(VocabularyCallback::Search).to_json(),
    )]);

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "+ Добавить из текста",
        CallbackData::Vocabulary(VocabularyCallback::AddFromText).to_json(),
    )]);

    for (card_id, _) in page_cards {
        rows.push(vec![
            teloxide::types::InlineKeyboardButton::callback(
                "Подробнее",
                CallbackData::Vocabulary(VocabularyCallback::Detail { card_id: *card_id })
                    .to_json(),
            ),
            teloxide::types::InlineKeyboardButton::callback(
                "Удалить 🗑️",
                CallbackData::Vocabulary(VocabularyCallback::Delete { card_id: *card_id })
                    .to_json(),
            ),
        ]);
    }

    if total_pages > 1 {
        let mut pagination_row = vec![];

        if current_page > 0 {
            pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
                "⬅️ Назад",
                CallbackData::Vocabulary(VocabularyCallback::Page {
                    page: current_page - 1,
                })
                .to_json(),
            ));
        }

        pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
            format!("{}/{}", current_page + 1, total_pages),
            CallbackData::Vocabulary(VocabularyCallback::PageCurrent).to_json(),
        ));

        if current_page < total_pages - 1 {
            pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
                "Далее ➡️",
                CallbackData::Vocabulary(VocabularyCallback::Page {
                    page: current_page + 1,
                })
                .to_json(),
            ));
        }

        rows.push(pagination_row);
    }

    InlineKeyboardMarkup::new(rows)
}

pub async fn fetch_vocabulary_cards_for_page_change(
    provider: &OrigaServiceProvider,
    user_id: Ulid,
) -> Result<Vec<(Ulid, origa::domain::StudyCard)>, teloxide::RequestError> {
    fetch_vocabulary_cards(provider, user_id).await
}

pub fn apply_filter_cards(
    cards: &[(Ulid, origa::domain::StudyCard)],
    filter: &str,
) -> Vec<(Ulid, origa::domain::StudyCard)> {
    apply_filter(cards, filter)
}

pub fn build_vocabulary_text_for_pagination(
    total_cards: usize,
    filter: &str,
    page_cards: &[(Ulid, origa::domain::StudyCard)],
    current_page: usize,
    total_pages: usize,
) -> String {
    build_vocabulary_text(total_cards, filter, page_cards, current_page, total_pages)
}
