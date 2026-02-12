use crate::bot::messaging::send_main_menu_with_stats;
use crate::handlers::add_from_text;
use crate::handlers::grammar::grammar_callback_handler;
use crate::handlers::kanji::kanji_callback_handler;
use crate::handlers::lesson::handle_lesson_callback;
use crate::handlers::menu_callback::handle_menu_callback;
use crate::handlers::profile::profile_callback_handler;
use crate::handlers::vocabulary::vocabulary_callback_handler;
use crate::handlers::{OrigaDialogue, SessionData, callbacks::CallbackData};
use crate::service::OrigaServiceProvider;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum Command {
    #[command(description = "Показать справку")]
    Help,
    #[command(description = "Запустить бота")]
    Start,
}

pub async fn help_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = "/start - Запустить бота\n/help - Показать эту справку";
    bot.send_message(msg.chat.id, text).await?;
    respond(())
}

pub async fn start_handler(bot: Bot, msg: Message, dialogue: OrigaDialogue) -> ResponseResult<()> {
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("Друг");

    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;

    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    send_main_menu_with_stats(
        &bot,
        msg.chat.id,
        &session.username,
        provider,
        session.user_id,
        Some(teloxide::types::ReplyMarkup::Keyboard(
            crate::bot::keyboard::reply_keyboard(),
        )),
    )
    .await
    .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    dialogue
        .exit()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    respond(())
}

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    dialogue: OrigaDialogue,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data.clone() else {
        return respond(());
    };
    let Some(message) = &q.message else {
        return respond(());
    };

    let chat_id = message.chat().id;
    let telegram_id = chat_id.0 as u64;
    let username = q.from.first_name.as_str();
    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    let Some(callback) = CallbackData::try_from_json(&data) else {
        return respond(());
    };

    handle_typed_callback(bot, q, dialogue, session, callback).await
}

async fn handle_typed_callback(
    bot: Bot,
    q: CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
    callback: CallbackData,
) -> ResponseResult<()> {
    match callback {
        CallbackData::AddFromText(_cb) => {
            add_from_text::add_from_text_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Lesson(_cb) => {
            handle_lesson_callback(bot, q, dialogue, session).await?;
        }
        CallbackData::Profile(cb) => {
            profile_callback_handler(&bot, &q, cb, &dialogue).await?;
        }
        CallbackData::Vocabulary(_cb) => {
            vocabulary_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Kanji(_cb) => {
            kanji_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Grammar(_cb) => {
            grammar_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Menu(cb) => {
            handle_menu_callback(&bot, &q, dialogue, session, cb).await?;
        }
    }
    respond(())
}
