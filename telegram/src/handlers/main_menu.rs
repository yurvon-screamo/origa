use super::OrigaDialogue;
use super::grammar::grammar_list_handler;
use super::kanji::handle_kanji_list;
use super::lesson::{start_fixation, start_lesson};
use super::profile::profile_handler;
use super::vocabulary::vocabulary_list_handler;
use crate::bot::{keyboard::reply_keyboard, messaging::send_main_menu_with_stats};
use crate::dialogue::{DialogueState, ProfileView};
use crate::service::OrigaServiceProvider;
use std::sync::Arc;
use teloxide::prelude::*;

fn get_user_info(msg: &Message) -> (&str, u64) {
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");
    let telegram_id = msg.chat.id.0 as u64;
    (username, telegram_id)
}

async fn get_session(
    msg: &Message,
) -> Result<(String, u64, crate::handlers::SessionData), teloxide::RequestError> {
    let (username, telegram_id) = get_user_info(msg);
    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;
    Ok((username.to_string(), telegram_id, session))
}

async fn handle_lesson_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;
    let session = provider.get_or_create_session(telegram_id, "User").await?;
    start_lesson(bot, msg, dialogue, session).await
}

async fn handle_fixation_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;
    let session = provider.get_or_create_session(telegram_id, "User").await?;
    start_fixation(bot, msg, dialogue, session).await
}

async fn handle_vocabulary_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let (_username, _, session) = get_session(&msg).await?;

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

async fn handle_kanji_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let (_, _, session) = get_session(&msg).await?;

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

async fn handle_grammar_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let (_, _, session) = get_session(&msg).await?;

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

async fn handle_profile_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let current_view = match dialogue.get().await.ok().flatten() {
        Some(DialogueState::Profile { current_view }) => current_view,
        _ => ProfileView::Main,
    };
    profile_handler(bot, msg, DialogueState::Profile { current_view }).await
}

async fn handle_settings_command(bot: Bot, msg: Message) -> teloxide::requests::ResponseResult<()> {
    profile_handler(
        bot,
        msg,
        DialogueState::Profile {
            current_view: ProfileView::Settings,
        },
    )
    .await
}

async fn handle_home_command(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> teloxide::requests::ResponseResult<()> {
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("Друг");
    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(msg.chat.id.0 as u64, username)
        .await?;

    send_main_menu_with_stats(
        &bot,
        msg.chat.id,
        username,
        provider,
        session.user_id,
        Some(teloxide::types::ReplyMarkup::Keyboard(reply_keyboard())),
    )
    .await
    .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    dialogue
        .exit()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    respond(())
}

pub async fn main_menu_handler(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        match text {
            "🎯 Урок" => {
                handle_lesson_command(bot, msg, dialogue).await?;
            }
            "🔒 Закрепление" => {
                handle_fixation_command(bot, msg, dialogue).await?;
            }
            "📚 Слова" => {
                handle_vocabulary_command(bot, msg, dialogue).await?;
            }
            "🈷 Кандзи" => {
                handle_kanji_command(bot, msg, dialogue).await?;
            }
            "📖 Грамматика" => {
                handle_grammar_command(bot, msg, dialogue).await?;
            }
            "👤 Профиль" => {
                handle_profile_command(bot, msg, dialogue).await?;
            }
            "⚙️ Настройки" => {
                handle_settings_command(bot, msg).await?;
            }
            "🏠 Главная" => {
                handle_home_command(bot, msg, dialogue).await?;
            }
            _ => {}
        }
    }
    respond(())
}
