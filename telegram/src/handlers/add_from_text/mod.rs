pub mod callbacks;

use crate::bot::messaging::send_main_menu_with_stats;
use crate::dialogue::{DialogueState, SessionData};
use crate::handlers::add_from_text::callbacks::AddFromTextCallback;
use crate::service::OrigaServiceProvider;
use origa::domain::tokenize_text;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn add_from_text_handler(
    bot: Bot,
    msg: Message,
    dialogue: crate::handlers::OrigaDialogue,
    pending_words: Vec<String>,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if pending_words.is_empty() {
            handle_text_input(&bot, msg.chat.id, dialogue, text).await?;
        } else {
            handle_word_selection(&bot, msg.chat.id, dialogue, text, pending_words).await?;
        }
    }
    respond(())
}

async fn handle_text_input(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
    text: &str,
) -> ResponseResult<()> {
    let tokens = match tokenize_text(text) {
        Ok(tokens) => tokens,
        Err(e) => {
            bot.send_message(chat_id, format!("Ошибка при токенизации: {}", e))
                .await?;
            return respond(());
        }
    };

    let words: Vec<String> = tokens
        .into_iter()
        .filter(|t| t.part_of_speech().is_vocabulary_word())
        .map(|t| t.orthographic_base_form().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .take(10)
        .collect();

    if words.is_empty() {
        bot.send_message(
            chat_id,
            "Не найдено слов для добавления. Попробуйте другой текст.",
        )
        .await?;
        return respond(());
    }

    let text_message = format!(
        "Найдено {} слов(а). Выберите слова для добавления:",
        words.len()
    );
    let keyboard = build_word_selection_keyboard(&words, &[]);

    bot.send_message(chat_id, text_message)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::AddFromText {
            pending_words: vec![],
        })
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    respond(())
}

async fn handle_word_selection(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
    text: &str,
    pending_words: Vec<String>,
) -> ResponseResult<()> {
    if text == "✅ Добавить выбранные" {
        add_selected_words(bot, chat_id, dialogue, pending_words).await?;
    } else if text == "❌ Отмена" {
        cancel_addition(bot, chat_id, dialogue).await?;
    }

    respond(())
}

async fn add_selected_words(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
    pending_words: Vec<String>,
) -> ResponseResult<()> {
    bot.send_message(
        chat_id,
        format!("Добавление {} слов...", pending_words.len()),
    )
    .await?;

    dialogue
        .update(DialogueState::VocabularyList {
            page: 0,
            items_per_page: 6,
            filter: "all".to_string(),
        })
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    respond(())
}

async fn cancel_addition(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
) -> ResponseResult<()> {
    bot.send_message(chat_id, "Добавление отменено.").await?;

    dialogue
        .update(DialogueState::VocabularyList {
            page: 0,
            items_per_page: 6,
            filter: "all".to_string(),
        })
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    respond(())
}

fn build_word_selection_keyboard(words: &[String], selected: &[String]) -> InlineKeyboardMarkup {
    let mut rows = vec![];

    for word in words.iter().take(10) {
        let is_selected = selected.contains(word);
        let label = if is_selected {
            format!("☑️ {}", word)
        } else {
            format!("⬜ {}", word)
        };
        let callback_data = AddFromTextCallback::Toggle { word: word.clone() };
        rows.push(vec![InlineKeyboardButton::callback(
            label,
            callback_data.to_json(),
        )]);
    }

    rows.push(vec![InlineKeyboardButton::callback(
        "✅ Добавить выбранные",
        AddFromTextCallback::Confirm.to_json(),
    )]);

    rows.push(vec![InlineKeyboardButton::callback(
        "❌ Отмена",
        AddFromTextCallback::Cancel.to_json(),
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub async fn add_from_text_callback_handler(
    bot: Bot,
    q: CallbackQuery,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let Some(data) = q.data.clone() else {
        return respond(());
    };

    let Some(callback) = AddFromTextCallback::try_from_json(&data) else {
        return respond(());
    };

    let Some(message) = q.message else {
        return respond(());
    };

    let chat_id = message.chat().id;
    let message_id = message.id();

    let current_state = dialogue.get().await.ok().flatten();
    let (available_words, mut selected) = match current_state {
        Some(DialogueState::AddFromText { pending_words }) => {
            (pending_words.clone(), pending_words.clone())
        }
        _ => (vec![], vec![]),
    };

    match callback {
        AddFromTextCallback::Toggle { word } => {
            if selected.contains(&word) {
                selected.retain(|w| w != &word);
            } else {
                selected.push(word);
            }

            let keyboard = build_word_selection_keyboard(&available_words, &selected);

            bot.edit_message_reply_markup(chat_id, message_id)
                .reply_markup(keyboard)
                .await?;

            dialogue
                .update(DialogueState::AddFromText {
                    pending_words: selected,
                })
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
        }
        AddFromTextCallback::Confirm => {
            if selected.is_empty() {
                bot.edit_message_text(chat_id, message_id, "Не выбрано ни одного слова.")
                    .await?;
            } else {
                bot.edit_message_text(
                    chat_id,
                    message_id,
                    format!("Добавление {} слов...", selected.len()),
                )
                .await?;

                let provider = OrigaServiceProvider::instance();
                let use_case = provider.create_vocabulary_card_use_case();

                let mut added_count = 0;
                for word in &selected {
                    match use_case.execute(session.user_id, word.clone()).await {
                        Ok(_) => added_count += 1,
                        Err(e) => {
                            tracing::error!("Ошибка при добавлении слова {}: {}", word, e);
                        }
                    }
                }

                bot.send_message(
                    chat_id,
                    format!("✅ Добавлено {} из {} слов", added_count, selected.len()),
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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

                dialogue.exit().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
            }
        }
        AddFromTextCallback::Cancel => {
            bot.edit_message_text(chat_id, message_id, "❌ Добавление отменено.")
                .await?;

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
    }

    bot.answer_callback_query(q.id).await?;
    respond(())
}
