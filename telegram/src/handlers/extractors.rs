use crate::bot::{keyboard::reply_keyboard, messaging::send_main_menu_with_stats};
use crate::dialogue::{DialogueState, ProfileView};
use crate::handlers::SessionData;
use crate::handlers::grammar::grammar_list_handler;
use crate::handlers::kanji::handle_kanji_list;
use crate::handlers::lesson::{start_fixation, start_lesson};
use crate::handlers::profile::profile_handler;
use crate::handlers::vocabulary::vocabulary_list_handler;
use crate::service::OrigaServiceProvider;
use std::sync::Arc;
use teloxide::prelude::*;

use super::OrigaDialogue;

pub fn username_from_msg(msg: &Message) -> &str {
    msg.from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User")
}

pub fn telegram_id_from_msg(msg: &Message) -> u64 {
    msg.chat.id.0 as u64
}

pub fn chat_id_from_msg(msg: &Message) -> ChatId {
    msg.chat.id
}

async fn prepare_session(msg: &Message) -> Result<(String, SessionData), teloxide::RequestError> {
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("Друг");
    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    Ok((username.to_string(), session))
}

async fn handle_home(
    bot: &Bot,
    msg: &Message,
    dialogue: OrigaDialogue,
    username: &str,
    provider: &OrigaServiceProvider,
    user_id: ulid::Ulid,
) -> Result<(), teloxide::RequestError> {
    send_main_menu_with_stats(
        bot,
        msg.chat.id,
        username,
        provider,
        user_id,
        Some(teloxide::types::ReplyMarkup::Keyboard(reply_keyboard())),
    )
    .await
    .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    dialogue
        .exit()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    Ok(())
}

async fn handle_lesson(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> Result<(), teloxide::RequestError> {
    start_lesson(bot, msg, dialogue, session).await
}

async fn handle_fixation(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> Result<(), teloxide::RequestError> {
    start_fixation(bot, msg, dialogue, session).await
}

async fn handle_vocabulary(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> Result<(), teloxide::RequestError> {
    let (page, items_per_page, filter) = match dialogue.get().await.ok().flatten() {
        Some(DialogueState::VocabularyList {
            page,
            items_per_page,
            filter,
        }) => (page, items_per_page, filter),
        _ => (0, 6, "all".to_string()),
    };
    vocabulary_list_handler(bot, msg, dialogue, (page, items_per_page, filter), session).await
}

async fn handle_kanji(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> Result<(), teloxide::RequestError> {
    let (page, items_per_page) = match dialogue.get().await.ok().flatten() {
        Some(DialogueState::KanjiList {
            page,
            items_per_page,
            ..
        }) => (page, items_per_page),
        _ => (0, 6),
    };
    handle_kanji_list(bot, msg, (page, items_per_page), session).await
}

async fn handle_grammar(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> Result<(), teloxide::RequestError> {
    let (page, items_per_page) = match dialogue.get().await.ok().flatten() {
        Some(DialogueState::GrammarList {
            page,
            items_per_page,
            ..
        }) => (page, items_per_page),
        _ => (0, 6),
    };
    grammar_list_handler(bot, msg, dialogue, (page, items_per_page), session).await
}

async fn handle_profile(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> Result<(), teloxide::RequestError> {
    let current_view = match dialogue.get().await.ok().flatten() {
        Some(DialogueState::Profile { current_view }) => current_view,
        _ => ProfileView::Main,
    };
    profile_handler(bot, msg, DialogueState::Profile { current_view }).await
}

async fn handle_settings(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    profile_handler(
        bot,
        msg,
        DialogueState::Profile {
            current_view: ProfileView::Settings,
        },
    )
    .await
}

pub async fn handle_common_text(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    text: &str,
) -> ResponseResult<bool> {
    let (username, session) = prepare_session(&msg).await?;
    let provider = OrigaServiceProvider::instance().await;

    match text {
        "🏠 Главная" => {
            handle_home(&bot, &msg, dialogue, &username, provider, session.user_id).await?;
            Ok(true)
        }
        "🎯 Урок" => {
            handle_lesson(bot, msg, dialogue, session).await?;
            Ok(true)
        }
        "🔒 Закрепление" => {
            handle_fixation(bot, msg, dialogue, session).await?;
            Ok(true)
        }
        "📚 Слова" => {
            handle_vocabulary(bot, msg, dialogue, session).await?;
            Ok(true)
        }
        "🈷 Кандзи" => {
            handle_kanji(bot, msg, dialogue, session).await?;
            Ok(true)
        }
        "📖 Грамматика" => {
            handle_grammar(bot, msg, dialogue, session).await?;
            Ok(true)
        }
        "👤 Профиль" => {
            handle_profile(bot, msg, dialogue).await?;
            Ok(true)
        }
        "⚙️ Настройки" => {
            handle_settings(bot, msg).await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
