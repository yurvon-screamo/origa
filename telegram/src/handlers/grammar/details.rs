use crate::dialogue::{DialogueState, SessionData};
use crate::handlers::OrigaDialogue;
use crate::handlers::callbacks::CallbackData;
use crate::handlers::menu::MenuCallback;
use origa::domain::{NativeLanguage, get_rule_by_id};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use ulid::Ulid;

use super::callback::message_id;
use super::callbacks::GrammarCallback;
use super::get_added_grammar_rule_ids;
use super::get_grammar_review_dates;
use super::grammar_list_keyboard;

pub fn grammar_detail_keyboard(rule_id: &Ulid, is_added: bool) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = vec![];

    let action_button = if is_added {
        InlineKeyboardButton::callback(
            "❌ Удалить",
            CallbackData::Grammar(GrammarCallback::Delete { rule_id: *rule_id }).to_json(),
        )
    } else {
        InlineKeyboardButton::callback(
            "➕ Добавить",
            CallbackData::Grammar(GrammarCallback::Add { rule_id: *rule_id }).to_json(),
        )
    };

    rows.push(vec![action_button]);
    rows.push(vec![
        InlineKeyboardButton::callback("⬅️ Назад", CallbackData::Grammar(GrammarCallback::BackToList).to_json()),
        InlineKeyboardButton::callback("🏠 Главная", CallbackData::Menu(MenuCallback::MainMenu).to_json()),
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
    rule_id: Ulid,
    session: SessionData,
) -> ResponseResult<()> {
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
    page: usize,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
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
