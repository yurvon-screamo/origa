use super::callbacks::GrammarCallback;
use crate::dialogue::{DialogueState, SessionData};
use crate::handlers::OrigaDialogue;
use crate::service::OrigaServiceProvider;
use chrono::{Datelike, TimeDelta};
use origa::domain::{Card, GRAMMAR_RULES, NativeLanguage};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use ulid::Ulid;

pub fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let today = now.date_naive();
    let date_naive = date.date_naive();

    if date_naive == today {
        "сегодня".to_string()
    } else if date_naive == today + TimeDelta::days(1) {
        "завтра".to_string()
    } else if date_naive == today + TimeDelta::days(2) {
        "послезавтра".to_string()
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

pub fn grammar_list_keyboard(
    page: usize,
    items_per_page: usize,
    review_dates: &HashMap<Ulid, String>,
) -> InlineKeyboardMarkup {
    let total_rules = GRAMMAR_RULES.len();
    let total_pages = total_rules.div_ceil(items_per_page);
    let start = page * items_per_page;
    let end = (start + items_per_page).min(total_rules);

    let mut rows: Vec<Vec<InlineKeyboardButton>> = vec![];

    rows.push(vec![InlineKeyboardButton::callback(
        "🔍 Поиск",
        GrammarCallback::Search.to_json(),
    )]);

    for i in start..end {
        let rule = &GRAMMAR_RULES[i];
        let content = rule.content(&NativeLanguage::Russian);
        let title = content.title();
        let rule_id = rule.rule_id();

        let button_text = if let Some(review_date) = review_dates.get(rule_id) {
            format!("✅ {}\nПовтор: {}", title, review_date)
        } else {
            title.to_string()
        };

        rows.push(vec![InlineKeyboardButton::callback(
            button_text,
            GrammarCallback::Detail { rule_id: *rule_id }.to_json(),
        )]);
    }

    rows.extend(build_navigation_buttons(page, total_pages));
    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Главная",
        "menu_home",
    )]);

    InlineKeyboardMarkup::new(rows)
}

fn build_navigation_buttons(page: usize, total_pages: usize) -> Option<Vec<InlineKeyboardButton>> {
    let mut nav_buttons = vec![];

    if page > 0 {
        nav_buttons.push(InlineKeyboardButton::callback(
            "⬅️ Назад",
            GrammarCallback::Page { page: page - 1 }.to_json(),
        ));
    }

    nav_buttons.push(InlineKeyboardButton::callback(
        format!("{}/{}", page + 1, total_pages),
        GrammarCallback::CurrentPage.to_json(),
    ));

    if page < total_pages - 1 {
        nav_buttons.push(InlineKeyboardButton::callback(
            "Далее ➡️",
            GrammarCallback::Page { page: page + 1 }.to_json(),
        ));
    }

    if nav_buttons.is_empty() {
        None
    } else {
        Some(nav_buttons)
    }
}

pub async fn get_grammar_review_dates(
    session: &SessionData,
) -> Result<HashMap<Ulid, String>, teloxide::RequestError> {
    let provider = OrigaServiceProvider::instance();
    let use_case = provider.knowledge_set_cards_use_case();
    let cards = use_case
        .execute(session.user_id)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    Ok(cards
        .into_iter()
        .filter_map(|sc| {
            if let Card::Grammar(grammar_card) = sc.card() {
                let rule_id = *grammar_card.rule_id();
                let next_review = sc
                    .memory()
                    .next_review_date()
                    .map(format_date)
                    .unwrap_or_else(|| "сегодня".to_string());
                Some((rule_id, next_review))
            } else {
                None
            }
        })
        .collect())
}

pub async fn get_added_grammar_rule_ids(
    session: &SessionData,
) -> Result<Vec<Ulid>, teloxide::RequestError> {
    let provider = OrigaServiceProvider::instance();
    let use_case = provider.knowledge_set_cards_use_case();
    let cards = use_case
        .execute(session.user_id)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?
        .into_iter()
        .filter_map(|sc| match sc.card() {
            Card::Grammar(grammar_card) => Some(*grammar_card.rule_id()),
            _ => None,
        })
        .collect();

    Ok(cards)
}

pub async fn grammar_list_handler(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    (page, items_per_page): (usize, usize),
    session: SessionData,
) -> ResponseResult<()> {
    let review_dates = get_grammar_review_dates(&session).await?;
    let text = "📖 Грамматика\n\nВыберите правило для просмотра:".to_string();
    let keyboard = grammar_list_keyboard(page, items_per_page, &review_dates);

    bot.send_message(msg.chat.id, text)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::GrammarList {
            page,
            items_per_page,
        })
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    respond(())
}
