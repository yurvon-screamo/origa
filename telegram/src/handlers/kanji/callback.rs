use crate::bot::messaging::send_main_menu_with_stats;
use crate::dialogue::{DialogueState, SessionData};
use crate::handlers::OrigaDialogue;
use crate::service::OrigaServiceProvider;
use std::sync::Arc;
use teloxide::prelude::*;

use super::actions::{
    handle_kanji_add, handle_kanji_add_new, handle_kanji_delete, handle_kanji_search,
};
use super::callbacks::KanjiCallback;
use super::details::handle_kanji_detail;
use super::list::handle_kanji_list_by_level;

pub async fn kanji_callback_handler(
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

    let current_state = dialogue.get().await.ok().flatten();
    let level = match current_state {
        Some(DialogueState::KanjiList { level, .. }) => level,
        _ => None,
    };

    match KanjiCallback::try_from_json(data) {
        Some(KanjiCallback::Level { level: new_level }) => {
            dialogue
                .update(DialogueState::KanjiList {
                    level: Some(new_level),
                    page: 0,
                    items_per_page: 6,
                })
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

            handle_kanji_list_by_level(&bot, chat_id, Some(&new_level), 0, 6, session.user_id)
                .await?;
        }
        Some(KanjiCallback::Page { page: new_page }) => {
            handle_kanji_list_by_level(&bot, chat_id, level.as_ref(), new_page, 6, session.user_id)
                .await?;
        }
        Some(KanjiCallback::PageCurrent) => {}
        Some(KanjiCallback::Detail { kanji }) => {
            handle_kanji_detail(&bot, chat_id, &kanji).await?;
        }
        Some(KanjiCallback::Add { kanji }) => {
            handle_kanji_add(&bot, chat_id, &kanji, session.user_id).await?;
        }
        Some(KanjiCallback::Delete { kanji }) => {
            handle_kanji_delete(&bot, chat_id, &kanji, session.user_id).await?;
        }
        Some(KanjiCallback::AddNew) => {
            handle_kanji_add_new(&bot, chat_id).await?;
        }
        Some(KanjiCallback::Search { query, page }) => {
            handle_kanji_search(&bot, chat_id, &query, page).await?;
        }
        Some(KanjiCallback::BackToList) => {
            handle_kanji_list_by_level(&bot, chat_id, level.as_ref(), 0, 6, session.user_id)
                .await?;
        }
        Some(KanjiCallback::MainMenu) => {
            let provider = OrigaServiceProvider::instance();
            send_main_menu_with_stats(
                &bot,
                chat_id,
                &session.username,
                provider,
                session.user_id,
                None,
            )
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
        }
        None => {}
    }

    respond(())
}
