mod callbacks;

use crate::bot::messaging::send_main_menu_with_stats;
use crate::dialogue::{DialogueState, LessonMode, SessionData};
use crate::handlers::callbacks::CallbackData;
use crate::service::OrigaServiceProvider;
use chrono::Duration;
use origa::application::srs_service::RateMode;
use origa::domain::{Card, Rating};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use ulid::Ulid;

pub use callbacks::LessonCallback;

pub async fn start_lesson(
    bot: Bot,
    msg: Message,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, &session.username)
        .await?;

    let use_case = provider.select_cards_to_lesson_use_case();
    let cards_result = use_case.execute(session.user_id).await;

    let cards: HashMap<Ulid, Card> = match cards_result {
        Ok(cards) => cards,
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Ошибка при загрузке карточек: {}", e))
                .await?;
            return respond(());
        }
    };

    if cards.is_empty() {
        bot.send_message(msg.chat.id, LessonCallback::NO_CARDS)
            .await?;
        return respond(());
    }

    let total_cards = cards.len();
    let card_ids: Vec<Ulid> = cards.keys().cloned().collect();

    dialogue
        .update(DialogueState::Lesson {
            mode: LessonMode::Lesson,
            card_ids: card_ids.clone(),
            current_index: 0,
            showing_answer: false,
            new_count: 0,
            review_count: 0,
        })
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    let lesson_start_text = format!(
        "{}\n{}: {}\n{}: 0/{}",
        LessonCallback::LESSON_STARTED,
        LessonCallback::CARDS,
        total_cards,
        LessonCallback::PROGRESS,
        total_cards
    );

    bot.send_message(msg.chat.id, lesson_start_text).await?;

    if let Some(first_card_id) = card_ids.first()
        && let Some(first_card) = cards.get(first_card_id)
    {
        let card_text = format_card_front(first_card);
        let keyboard = lesson_rating_keyboard();
        bot.send_message(msg.chat.id, card_text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    }

    respond(())
}

pub async fn start_fixation(
    bot: Bot,
    msg: Message,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, &session.username)
        .await?;

    let use_case = provider.select_cards_to_fixation_use_case();
    let cards_result = use_case.execute(session.user_id).await;

    let cards: HashMap<Ulid, Card> = match cards_result {
        Ok(cards) => cards,
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Ошибка при загрузке карточек: {}", e))
                .await?;
            return respond(());
        }
    };

    if cards.is_empty() {
        bot.send_message(msg.chat.id, LessonCallback::NO_FIXATION_CARDS)
            .await?;
        return respond(());
    }

    let total_cards = cards.len();
    let card_ids: Vec<Ulid> = cards.keys().cloned().collect();

    dialogue
        .update(DialogueState::Lesson {
            mode: LessonMode::Fixation,
            card_ids: card_ids.clone(),
            current_index: 0,
            showing_answer: false,
            new_count: 0,
            review_count: 0,
        })
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    let lesson_start_text = format!(
        "{}\n{}: {}\n{}: 0/{}",
        LessonCallback::FIXATION_STARTED,
        LessonCallback::CARDS,
        total_cards,
        LessonCallback::PROGRESS,
        total_cards
    );

    bot.send_message(msg.chat.id, lesson_start_text).await?;

    if let Some(first_card_id) = card_ids.first()
        && let Some(first_card) = cards.get(first_card_id)
    {
        let card_text = format_card_front(first_card);
        let keyboard = lesson_rating_keyboard();
        bot.send_message(msg.chat.id, card_text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    }

    respond(())
}

pub async fn handle_lesson_callback(
    bot: Bot,
    q: CallbackQuery,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data else {
        return respond(());
    };
    let Some(message) = q.message else {
        return respond(());
    };
    let chat_id = message.chat().id;

    let Some(callback) = LessonCallback::try_from_json(&data) else {
        return respond(());
    };

    match callback {
        LessonCallback::Rating { rating } => {
            handle_rating(&bot, chat_id, rating, dialogue, session).await?;
        }
        LessonCallback::NextCard => {
            handle_next_card(&bot, chat_id, dialogue, session).await?;
        }
        LessonCallback::AbortLesson => {
            handle_abort_lesson(&bot, chat_id, dialogue, session).await?;
        }
        LessonCallback::BackToMain => {
            let provider = OrigaServiceProvider::instance().await;

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

    respond(())
}

async fn handle_rating(
    bot: &Bot,
    chat_id: ChatId,
    rating: Rating,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let current_state = dialogue.get().await.ok().flatten();
    if let Some(DialogueState::Lesson {
        current_index,
        mut new_count,
        mut review_count,
        mode,
        ..
    }) = current_state
    {
        let cards_result = match mode {
            LessonMode::Lesson => {
                provider
                    .select_cards_to_lesson_use_case()
                    .execute(session.user_id)
                    .await
            }
            LessonMode::Fixation => {
                provider
                    .select_cards_to_fixation_use_case()
                    .execute(session.user_id)
                    .await
            }
        };

        let cards: HashMap<Ulid, Card> = match cards_result {
            Ok(c) => c,
            Err(_) => return respond(()),
        };

        let card_id = cards.keys().nth(current_index);
        if let Some(card_id) = card_id
            && let Some(card) = cards.get(card_id)
        {
            let rate_use_case = provider.rate_card_use_case();

            let _ = rate_use_case
                .execute(session.user_id, *card_id, RateMode::StandardLesson, rating)
                .await;

            if card.question().text()
                == cards
                    .values()
                    .next()
                    .map(|c| c.question().text())
                    .unwrap_or_default()
            {
                new_count += 1;
            } else {
                review_count += 1;
            }

            let answer_text = format_card_back(card);
            let mut keyboard_rows = vec![vec![InlineKeyboardButton::callback(
                LessonCallback::NEXT_CARD,
                CallbackData::Lesson(LessonCallback::NextCard).to_json(),
            )]];

            keyboard_rows.push(vec![InlineKeyboardButton::callback(
                LessonCallback::ABORT_LESSON,
                CallbackData::Lesson(LessonCallback::AbortLesson).to_json(),
            )]);

            bot.send_message(chat_id, answer_text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(InlineKeyboardMarkup::new(keyboard_rows))
                .await?;

            dialogue
                .update(DialogueState::Lesson {
                    mode,
                    card_ids: cards.keys().cloned().collect(),
                    current_index,
                    showing_answer: true,
                    new_count,
                    review_count,
                })
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
        }
    }

    respond(())
}

async fn handle_next_card(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let current_state = dialogue.get().await.ok().flatten();
    if let Some(DialogueState::Lesson {
        current_index,
        new_count,
        review_count,
        mode,
        ..
    }) = current_state
    {
        let cards_result = match mode {
            LessonMode::Lesson => {
                provider
                    .select_cards_to_lesson_use_case()
                    .execute(session.user_id)
                    .await
            }
            LessonMode::Fixation => {
                provider
                    .select_cards_to_fixation_use_case()
                    .execute(session.user_id)
                    .await
            }
        };

        let cards: HashMap<Ulid, Card> = match cards_result {
            Ok(c) => c,
            Err(_) => return respond(()),
        };

        let total_cards = cards.len();
        let next_index = current_index + 1;

        if next_index >= total_cards {
            show_lesson_complete(bot, chat_id, new_count, review_count, mode).await?;

            let complete_use_case = provider.complete_lesson_use_case();
            let _ = complete_use_case
                .execute(session.user_id, Duration::seconds(0))
                .await;

            dialogue.exit().await.map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
            return respond(());
        }

        if let Some(card_id) = cards.keys().nth(next_index)
            && let Some(card) = cards.get(card_id)
        {
            let card_text = format_card_front(card);
            let keyboard = lesson_rating_keyboard();

            bot.send_message(
                chat_id,
                format!(
                    "{} {}/{}",
                    LessonCallback::CARD,
                    next_index + 1,
                    total_cards
                ),
            )
            .await?;

            bot.send_message(chat_id, card_text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(keyboard)
                .await?;

            dialogue
                .update(DialogueState::Lesson {
                    mode,
                    card_ids: cards.keys().cloned().collect(),
                    current_index: next_index,
                    showing_answer: false,
                    new_count,
                    review_count,
                })
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
        }
    }

    respond(())
}

async fn handle_abort_lesson(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    bot.send_message(chat_id, LessonCallback::LESSON_ABORTED)
        .await?;

    let provider = OrigaServiceProvider::instance().await;

    send_main_menu_with_stats(
        bot,
        chat_id,
        &session.username,
        provider,
        session.user_id,
        None,
    )
    .await
    .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    dialogue
        .exit()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    respond(())
}

async fn show_lesson_complete(
    bot: &Bot,
    chat_id: ChatId,
    new_count: usize,
    review_count: usize,
    mode: LessonMode,
) -> ResponseResult<()> {
    let mode_text = match mode {
        LessonMode::Lesson => LessonCallback::LESSON_COMPLETE,
        LessonMode::Fixation => LessonCallback::FIXATION_COMPLETE,
    };

    let text = format!(
        "{}\n{}: {} | {}: {}\n\n{}",
        mode_text,
        LessonCallback::NEW,
        new_count,
        LessonCallback::REVIEWED,
        review_count,
        LessonCallback::BACK_TO_MAIN
    );

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        LessonCallback::BACK_TO_MAIN,
        CallbackData::Lesson(LessonCallback::BackToMain).to_json(),
    )]]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;
    respond(())
}

pub fn format_card_front(card: &Card) -> String {
    let question = card.question().text();
    match card {
        Card::Vocabulary(_) => {
            format!("<b>{}</b>", question)
        }
        Card::Kanji(_) => {
            format!("<b>{}</b>", question)
        }
        Card::Grammar(grammar) => {
            format!("<b>{}</b>", grammar.title().text())
        }
    }
}

fn format_card_back(card: &Card) -> String {
    let question = card.question().text();
    let answer = card.answer().text();
    match card {
        Card::Vocabulary(_) => {
            format!(
                "<b>{}</b>\n\n<b>{}:</b> {}\n\n<b>{}:</b>\n{}",
                question,
                LessonCallback::TRANSLATION,
                answer,
                LessonCallback::EXAMPLES,
                LessonCallback::EXAMPLE_SENTENCE
            )
        }
        Card::Kanji(_) => {
            format!(
                "<b>{}</b>\n\n<b>{}:</b> {}",
                question,
                LessonCallback::MEANINGS,
                answer
            )
        }
        Card::Grammar(grammar) => {
            format!(
                "<b>{}</b>\n\n<b>{}:</b> {}",
                grammar.title().text(),
                LessonCallback::BRIEFLY,
                answer
            )
        }
    }
}

pub fn lesson_rating_keyboard() -> InlineKeyboardMarkup {
    let ratings = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];
    let buttons: Vec<_> = ratings
        .into_iter()
        .map(|rating| {
            InlineKeyboardButton::callback(
                LessonCallback::rating_button_text(rating),
                CallbackData::Lesson(LessonCallback::Rating { rating }).to_json(),
            )
        })
        .collect();
    InlineKeyboardMarkup::new(vec![buttons])
}
