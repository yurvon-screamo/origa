use crate::handlers::OrigaDialogue;
use crate::telegram_domain::{DialogueState, SessionData};
use origa::domain::{NativeLanguage, get_rule_by_id};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use ulid::Ulid;

use super::callback::message_id;
use super::get_added_grammar_rule_ids;
use super::get_grammar_review_dates;
use super::grammar_list_keyboard;

pub fn grammar_detail_keyboard(rule_id: &Ulid, is_added: bool) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = vec![];

    let action_button = if is_added {
        InlineKeyboardButton::callback("❌ Удалить", format!("grammar_delete_{}", rule_id))
    } else {
        InlineKeyboardButton::callback("➕ Добавить", format!("grammar_add_{}", rule_id))
    };

    rows.push(vec![action_button]);
    rows.push(vec![
        InlineKeyboardButton::callback("⬅️ Назад", "grammar_back_to_list"),
        InlineKeyboardButton::callback("🏠 Главная", "menu_home"),
    ]);

    InlineKeyboardMarkup::new(rows)
}

pub fn format_grammar_detail_text(rule_id: &Ulid) -> Result<String, teloxide::RequestError> {
    let rule = get_rule_by_id(rule_id).ok_or_else(|| {
        teloxide::RequestError::Io(Arc::new(std::io::Error::other("Rule not found")))
    })?;

    let content = rule.content(&NativeLanguage::Russian);

    let mut text = format!(
        "{}\n\nКратко: {}\n",
        content.title(),
        content.short_description()
    );

    text.push_str("\nПовтор: послезавтра");

    Ok(text)
}

pub async fn handle_grammar_detail(
    bot: &Bot,
    chat_id: ChatId,
    data: &str,
    session: SessionData,
) -> ResponseResult<()> {
    let rule_id = parse_rule_id(data)?;
    let added_rule_ids = get_added_grammar_rule_ids(&session).await?;
    let is_added = added_rule_ids.contains(&rule_id);

    let text = format_grammar_detail_text(&rule_id)?;
    let keyboard = grammar_detail_keyboard(&rule_id, is_added);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    respond(())
}

pub async fn handle_grammar_page(
    bot: &Bot,
    chat_id: ChatId,
    data: &str,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let page = parse_page(data);
    let items_per_page = 6;

    let review_dates = get_grammar_review_dates(&session).await?;
    let text = "📖 Грамматика\n\nВыберите правило для просмотра:".to_string();
    let keyboard = grammar_list_keyboard(page, items_per_page, &review_dates);

    bot.edit_message_text(chat_id, message_id(bot, chat_id).await?, text)
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

fn parse_rule_id(data: &str) -> Result<Ulid, teloxide::RequestError> {
    data.strip_prefix("grammar_detail_")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            teloxide::RequestError::Io(Arc::new(std::io::Error::other("Invalid rule ID")))
        })
}

fn parse_page(data: &str) -> usize {
    data.strip_prefix("grammar_page_")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
