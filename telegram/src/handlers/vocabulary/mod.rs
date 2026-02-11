pub mod actions;
pub mod details;
pub mod list;
pub mod pagination;
pub mod search;

use crate::telegram_domain::{DialogueState, SessionData};
use teloxide::prelude::*;

pub use list::vocabulary_list_handler;
pub use search::handle_search_page_change;

pub async fn vocabulary_callback_handler(
    bot: Bot,
    q: CallbackQuery,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let Some(data) = q.data.clone() else {
        return respond(());
    };

    let Some(message) = q.message else {
        return respond(());
    };
    let chat_id = message.chat().id;
    let message_id = message.id();

    if data.starts_with("vocab_filter_") {
        pagination::handle_filter_change(&bot, chat_id, message_id, &data, dialogue, session)
            .await?;
    } else if data.starts_with("vocab_page_") {
        pagination::handle_page_change(&bot, chat_id, message_id, &data, dialogue, session).await?;
    } else if data.starts_with("vocab_detail_") {
        details::handle_show_detail(&bot, chat_id, &data, session).await?;
    } else if data.starts_with("vocab_delete_") {
        actions::handle_delete_request(&bot, chat_id, message_id, &data, session).await?;
    } else if data.starts_with("vocab_confirm_delete_") {
        actions::handle_confirm_delete(&bot, chat_id, message_id, &data, dialogue, session).await?;
    } else if data == "vocab_cancel_delete" {
        actions::handle_cancel_delete(&bot, chat_id, message_id).await?;
    } else if data == "vocab_add_from_text" {
        actions::handle_add_from_text(&bot, chat_id, message_id, dialogue).await?;
    } else if data == "vocab_search" {
        handle_search_request(&bot, chat_id, message_id, dialogue, session).await?;
    } else if data.starts_with("vocab_search_page_") {
        handle_search_page_change(&bot, chat_id, message_id, &data, dialogue, session).await?;
    }

    bot.answer_callback_query(q.id).await?;
    respond(())
}

async fn handle_search_request(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
    dialogue: crate::handlers::OrigaDialogue,
    _session: SessionData,
) -> ResponseResult<()> {
    bot.edit_message_text(
        chat_id,
        message_id,
        "🔍 Введите японское слово или его перевод для поиска...",
    )
    .await?;

    dialogue
        .update(DialogueState::VocabularySearch {
            page: 0,
            items_per_page: 6,
            query: "".to_string(),
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}
