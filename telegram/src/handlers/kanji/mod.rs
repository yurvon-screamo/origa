pub mod actions;
pub mod details;
pub mod list;

use crate::handlers::OrigaDialogue;
use crate::telegram_domain::SessionData;

use chrono::Datelike;
pub use list::handle_kanji_list;

use crate::telegram_domain::DialogueState;
use origa::domain::KanjiInfo;
use teloxide::prelude::*;

pub async fn handle_kanji_callback(
    bot: Bot,
    data: String,
    chat_id: ChatId,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let current_state = dialogue.get().await.ok().flatten();

    let (_page, level) = match current_state {
        Some(DialogueState::KanjiList { page, level, .. }) => (page, level),
        _ => (0, "all".to_string()),
    };

    if data.starts_with("kanji_level_") {
        let new_level = data.strip_prefix("kanji_level_").unwrap_or("all");
        dialogue
            .update(DialogueState::KanjiList {
                level: new_level.to_string(),
                page: 0,
                items_per_page: 6,
            })
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                    e.to_string(),
                )))
            })?;

        list::handle_kanji_list_by_level(&bot, chat_id, new_level, 0, 6, session.user_id).await?;
    } else if data.starts_with("kanji_page_") {
        let parts: Vec<&str> = data.split('_').collect();
        if parts.len() >= 3
            && let Ok(new_page) = parts[2].parse::<usize>()
        {
            list::handle_kanji_list_by_level(&bot, chat_id, &level, new_page, 6, session.user_id)
                .await?;
        }
    } else if data.starts_with("kanji_add_") {
        let kanji_char = data.strip_prefix("kanji_add_").unwrap_or("");
        actions::handle_kanji_add(&bot, chat_id, kanji_char, session.user_id).await?;
    } else if data.starts_with("kanji_delete_") {
        let kanji_char = data.strip_prefix("kanji_delete_").unwrap_or("");
        actions::handle_kanji_delete(&bot, chat_id, kanji_char, session.user_id).await?;
    } else if data.starts_with("kanji_detail_") {
        let kanji_char = data.strip_prefix("kanji_detail_").unwrap_or("");
        details::handle_kanji_detail(&bot, chat_id, kanji_char).await?;
    } else if data == "kanji_back_to_list" {
        list::handle_kanji_list_by_level(&bot, chat_id, &level, 0, 6, session.user_id).await?;
    } else if data == "kanji_add_new" {
        actions::handle_kanji_add_new(&bot, chat_id).await?;
    } else if data == "kanji_current_page" {
        // Do nothing
    } else if data.starts_with("kanji_search_page_") {
        let parts: Vec<&str> = data.split('_').collect();
        if parts.len() >= 4
            && let Ok(page) = parts[3].parse::<usize>()
        {
            let query = parts[4..].join("_");
            actions::handle_kanji_search(&bot, chat_id, &query, page).await?;
        }
    }

    respond(())
}

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
