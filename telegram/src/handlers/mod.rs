use crate::bot::messaging::send_history;
use crate::bot::{keyboard::reply_keyboard, messaging::send_main_menu_with_stats};
use crate::dialogue::{self, DialogueState, ProfileView};
use crate::service::OrigaServiceProvider;
use callbacks::CallbackData;
use lesson::handle_lesson_callback;
use lesson::start_fixation;
use lesson::start_lesson;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::{dispatching::dialogue::InMemStorage, utils::command::BotCommands};

pub mod add_from_text;
pub mod callbacks;
pub mod grammar;
pub mod kanji;
pub mod lesson;
pub mod menu;
pub mod profile;
pub mod vocabulary;

pub use profile::{handle_duolingo_token, profile_handler};

pub use dialogue::SessionData;
pub type OrigaDialogue =
    teloxide::dispatching::dialogue::Dialogue<DialogueState, InMemStorage<DialogueState>>;

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
    let provider = OrigaServiceProvider::instance();

    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    send_main_menu_with_stats(
        &bot,
        msg.chat.id,
        &session.username,
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
                let telegram_id = msg.chat.id.0 as u64;
                let provider = OrigaServiceProvider::instance();

                let session = provider.get_or_create_session(telegram_id, "User").await?;

                start_lesson(bot, msg, dialogue, session).await?;
            }
            "🔒 Закрепление" => {
                let telegram_id = msg.chat.id.0 as u64;
                let provider = OrigaServiceProvider::instance();

                let session = provider.get_or_create_session(telegram_id, "User").await?;

                start_fixation(bot, msg, dialogue, session).await?;
            }
            "📚 Слова" => {
                dialogue
                    .update(DialogueState::VocabularyList {
                        page: 0,
                        items_per_page: 6,
                        filter: "all".to_string(),
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            "🈷 Кандзи" => {
                dialogue
                    .update(DialogueState::KanjiList {
                        level: None,
                        page: 0,
                        items_per_page: 6,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            "📖 Грамматика" => {
                dialogue
                    .update(DialogueState::GrammarList {
                        page: 0,
                        items_per_page: 6,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            "👤 Профиль" => {
                dialogue
                    .update(DialogueState::Profile {
                        current_view: ProfileView::Main,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            "⚙️ Настройки" => {
                dialogue
                    .update(DialogueState::Profile {
                        current_view: ProfileView::Settings,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            "🏠 Главная" => {
                let username = msg
                    .from
                    .as_ref()
                    .map(|u| u.first_name.as_str())
                    .unwrap_or("Друг");
                let provider = OrigaServiceProvider::instance();

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
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
                dialogue.exit().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
            }
            _ => {}
        }
    }
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
    let provider = OrigaServiceProvider::instance();
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
            profile::profile_callback_handler(&bot, &q, cb, &dialogue).await?;
        }
        CallbackData::Vocabulary(_cb) => {
            vocabulary::vocabulary_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Kanji(_cb) => {
            kanji::kanji_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Grammar(_cb) => {
            grammar::grammar_callback_handler(bot, q, dialogue, session).await?;
        }
        CallbackData::Menu(cb) => {
            handle_menu_callback(&bot, &q, dialogue, session, cb).await?;
        }
    }
    respond(())
}

async fn handle_menu_callback(
    bot: &Bot,
    q: &CallbackQuery,
    _dialogue: OrigaDialogue,
    session: SessionData,
    callback: menu::MenuCallback,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    match callback {
        menu::MenuCallback::MainMenu => {
            if let Some(chat_id) = chat_id {
                crate::bot::messaging::send_main_menu_with_stats(
                    bot,
                    chat_id,
                    &session.username,
                    OrigaServiceProvider::instance(),
                    session.user_id,
                    None,
                )
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
            }
        }
        menu::MenuCallback::HistoryKnown
        | menu::MenuCallback::HistoryInProgress
        | menu::MenuCallback::HistoryNew
        | menu::MenuCallback::HistoryHard
        | menu::MenuCallback::ShowHistory => {
            if let Some(chat_id) = chat_id {
                send_history(
                    bot,
                    chat_id,
                    session.user_id,
                    OrigaServiceProvider::instance(),
                )
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
            }
        }
        _ => {}
    }
    respond(())
}
