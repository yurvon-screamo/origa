use crate::bot::messaging::send_history;
use crate::repository::build_repository;
use crate::telegram_domain::DialogueState;
use crate::{
    bot::{keyboard::reply_keyboard, messaging::send_main_menu_with_stats},
    telegram_domain::SessionData,
};
use lesson::{handle_lesson_callback, start_fixation, start_lesson};
use origa::application::UserRepository;
use origa::infrastructure::FileSystemUserRepository;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::{dispatching::dialogue::InMemStorage, utils::command::BotCommands};

pub mod add_from_text;
pub mod grammar;
pub mod kanji;
pub mod lesson;
pub mod profile;
pub mod vocabulary;

pub use profile::{handle_duolingo_token, profile_handler};

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
    let text = "/start - Запустить бота\\n/help - Показать эту справку";
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
    let repository = build_repository().await.unwrap();

    let session = find_or_create_by_telegram_id(&repository, telegram_id, username)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    send_main_menu_with_stats(
        &bot,
        msg.chat.id,
        &session.username,
        &repository,
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
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, telegram_id, "User").await?;

                start_lesson(bot, msg, dialogue, session).await?;
            }
            "🔒 Закрепление" => {
                let telegram_id = msg.chat.id.0 as u64;
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, telegram_id, "User").await?;

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
                        level: "all".to_string(),
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
                        current_view: "main".to_string(),
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            "⚙️ Настройки" => {
                dialogue
                    .update(DialogueState::Profile {
                        current_view: "settings".to_string(),
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
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, msg.chat.id.0 as u64, username)
                        .await?;

                send_main_menu_with_stats(
                    &bot,
                    msg.chat.id,
                    username,
                    &repository,
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

    if let Some(data) = q.data.clone()
        && let Some(message) = &q.message
    {
        let chat_id = message.chat().id;

        match data.as_str() {
            d if d.starts_with("rating_")
                || d == "next_card"
                || d == "abort_lesson"
                || d == "back_to_main" =>
            {
                let telegram_id = chat_id.0 as u64;
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, telegram_id, "User").await?;

                handle_lesson_callback(bot, q, dialogue, session).await?;
            }
            "show_history"
            | "history_known"
            | "history_in_progress"
            | "history_new"
            | "history_hard" => {
                let telegram_id = chat_id.0 as u64;
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, telegram_id, "User").await?;

                send_history(&bot, chat_id, session.user_id, &repository)
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                    })?;
            }
            d if d.starts_with("text_") => {
                let telegram_id = chat_id.0 as u64;
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, telegram_id, "User").await?;

                let current_state = dialogue.get().await.ok().flatten();
                let pending_words = match current_state {
                    Some(DialogueState::AddFromText { pending_words }) => pending_words,
                    _ => vec![],
                };

                add_from_text::add_from_text_callback_handler(
                    bot,
                    q,
                    dialogue,
                    pending_words,
                    session,
                )
                .await?;
            }
            d if d.starts_with("profile_") || d.starts_with("jlpt_set_") => {
                if d == "profile_confirm_exit" {
                    profile::confirm_exit_handler(&bot, &q, &dialogue).await?;
                } else {
                    profile::profile_callback_handler(&bot, &q, d, &dialogue).await?;
                }
            }
            d if d.starts_with("vocab_") || d == "menu_vocabulary" => {
                let telegram_id = chat_id.0 as u64;
                let repository = build_repository().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                let session =
                    find_or_create_by_telegram_id(&repository, telegram_id, "User").await?;

                if d == "menu_vocabulary" {
                    dialogue
                        .update(DialogueState::VocabularyList {
                            page: 0,
                            items_per_page: 6,
                            filter: "all".to_string(),
                        })
                        .await
                        .map_err(|e| {
                            teloxide::RequestError::Io(Arc::new(std::io::Error::other(
                                e.to_string(),
                            )))
                        })?;
                }

                vocabulary::vocabulary_callback_handler(bot, q, dialogue, session).await?;
            }
            d if d.starts_with("grammar_") || d == "menu_grammar" => {
                grammar::grammar_callback_handler(bot, q, dialogue).await?;
            }
            d if d.starts_with("kanji_") || d == "menu_kanji" => {
                if d == "menu_kanji" {
                    dialogue
                        .update(DialogueState::KanjiList {
                            level: "all".to_string(),
                            page: 0,
                            items_per_page: 6,
                        })
                        .await
                        .map_err(|e| {
                            teloxide::RequestError::Io(Arc::new(std::io::Error::other(
                                e.to_string(),
                            )))
                        })?;
                }

                kanji::handle_kanji_callback(bot, d.to_string(), chat_id, dialogue).await?;
            }
            _ => {}
        }
    }
    respond(())
}

async fn find_or_create_by_telegram_id(
    repository: &FileSystemUserRepository,
    telegram_id: u64,
    username: &str,
) -> ResponseResult<SessionData> {
    let user = repository
        .find_by_telegram_id(&telegram_id)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    if let Some(user) = user {
        return Ok(SessionData {
            user_id: user.id(),
            username: username.to_string(),
        });
    }

    let user = origa::domain::User::new(
        username.to_string(),
        origa::domain::JapaneseLevel::N5,
        origa::domain::NativeLanguage::Russian,
    );

    repository
        .save(&user)
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    Ok(SessionData {
        user_id: user.id(),
        username: username.to_string(),
    })
}
