use crate::handlers::OrigaDialogue;
use crate::telegram_domain::SessionData;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{MaybeInaccessibleMessage, UpdateKind};

use super::actions::{
    handle_grammar_add, handle_grammar_back_to_list, handle_grammar_delete, handle_grammar_search,
};
use super::details::{handle_grammar_detail, handle_grammar_page};

pub async fn grammar_callback_handler(
    bot: Bot,
    q: CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = &q.data else {
        return respond(());
    };

    let Some(message) = &q.message else {
        return respond(());
    };

    let chat_id = message.chat().id;

    match data.as_str() {
        d if d.starts_with("grammar_page_") => {
            handle_grammar_page(&bot, chat_id, d, dialogue, session.clone()).await?;
        }
        d if d.starts_with("grammar_detail_") => {
            handle_grammar_detail(&bot, chat_id, d, session.clone()).await?;
        }
        d if d.starts_with("grammar_add_") => {
            handle_grammar_add(&bot, chat_id, d, dialogue, session.clone()).await?;
        }
        d if d.starts_with("grammar_delete_") => {
            handle_grammar_delete(&bot, chat_id, d, dialogue, session.clone()).await?;
        }
        "grammar_back_to_list" => {
            handle_grammar_back_to_list(&bot, chat_id, dialogue, session).await?;
        }
        "grammar_current_page" => {}
        "grammar_search" => {
            let Some(MaybeInaccessibleMessage::Regular(message)) = &q.message else {
                return respond(());
            };
            let message_id = message.id;
            handle_grammar_search(&bot, chat_id, message_id).await?;
        }
        _ => {}
    }

    respond(())
}

pub async fn message_id(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
) -> ResponseResult<teloxide::types::MessageId> {
    let updates = bot.get_updates().await?;
    let last_message = updates.iter().find_map(|u| {
        if let Some(chat) = u.chat()
            && chat.id == chat_id
            && let UpdateKind::Message(msg) = &u.kind
        {
            return Some(msg.id);
        }

        None
    });

    match last_message {
        Some(msg_id) => Ok(msg_id),
        None => Err(teloxide::RequestError::Io(Arc::new(std::io::Error::other(
            "No message found",
        )))),
    }
}
