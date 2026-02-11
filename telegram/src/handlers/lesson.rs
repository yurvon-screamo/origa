use crate::bot::messaging::send_main_menu_with_stats;
use crate::repository::build_repository;
use crate::telegram_domain::state::LessonMode;
use crate::telegram_domain::{DialogueState, SessionData};
use chrono::Duration;
use origa::application::srs_service::RateMode;
use origa::application::{
    CompleteLessonUseCase, RateCardUseCase, SelectCardsToFixationUseCase,
    SelectCardsToLessonUseCase,
};
use origa::domain::{Card, Rating};
use origa::infrastructure::FsrsSrsService;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use ulid::Ulid;

pub async fn start_lesson(
    bot: Bot,
    msg: Message,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let repository = build_repository()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    let use_case = SelectCardsToLessonUseCase::new(&repository);
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
        bot.send_message(
            msg.chat.id,
            "Нет карточек для урока. Добавьте новые слова или подождите повторения.",
        )
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
        "🎯 Урок начат\\nКарточек: {}\\nПрогресс: 0/{}",
        total_cards, total_cards
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
    let repository = build_repository()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    let use_case = SelectCardsToFixationUseCase::new(&repository);
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
        bot.send_message(msg.chat.id, "Нет сложных карточек для закрепления.")
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
        "🔒 Закрепление начато\\nКарточек: {}\\nПрогресс: 0/{}",
        total_cards, total_cards
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
    let Some(data) = q.data else {
        return respond(());
    };
    let Some(message) = q.message else {
        return respond(());
    };
    let chat_id = message.chat().id;

    match data.as_str() {
        d if d.starts_with("rating_") => {
            handle_rating(&bot, chat_id, d, dialogue, session).await?;
        }
        "next_card" => {
            handle_next_card(&bot, chat_id, dialogue, session).await?;
        }
        "abort_lesson" => {
            handle_abort_lesson(&bot, chat_id, dialogue, session).await?;
        }
        "back_to_main" => {
            let repository = build_repository().await.map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;

            send_main_menu_with_stats(
                &bot,
                chat_id,
                &session.username,
                &repository,
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
        _ => {}
    }

    respond(())
}

async fn handle_rating(
    bot: &Bot,
    chat_id: ChatId,
    data: &str,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let rating = match data {
        "rating_again" => Rating::Again,
        "rating_hard" => Rating::Hard,
        "rating_good" => Rating::Good,
        "rating_easy" => Rating::Easy,
        _ => return respond(()),
    };

    let repository = build_repository()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

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
                SelectCardsToLessonUseCase::new(&repository)
                    .execute(session.user_id)
                    .await
            }
            LessonMode::Fixation => {
                SelectCardsToFixationUseCase::new(&repository)
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
            let srs_service = FsrsSrsService::new().map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;

            let rate_use_case = RateCardUseCase::new(&repository, &srs_service);

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
                "Далее ➡️",
                "next_card",
            )]];

            keyboard_rows.push(vec![InlineKeyboardButton::callback(
                "Прервать",
                "abort_lesson",
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
    let current_state = dialogue.get().await.ok().flatten();
    if let Some(DialogueState::Lesson {
        current_index,
        new_count,
        review_count,
        mode,
        ..
    }) = current_state
    {
        let repository = build_repository().await.map_err(|e| {
            teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
        })?;

        let cards_result = match mode {
            LessonMode::Lesson => {
                SelectCardsToLessonUseCase::new(&repository)
                    .execute(session.user_id)
                    .await
            }
            LessonMode::Fixation => {
                SelectCardsToFixationUseCase::new(&repository)
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

            let complete_use_case = CompleteLessonUseCase::new(&repository);
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
                format!("Карточка {}/{}", next_index + 1, total_cards),
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
    bot.send_message(chat_id, "Урок прерван.").await?;

    let repository = build_repository()
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

    send_main_menu_with_stats(
        bot,
        chat_id,
        &session.username,
        &repository,
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
        LessonMode::Lesson => "🎉 Урок завершён!",
        LessonMode::Fixation => "🎉 Закрепление завершено!",
    };

    let text = format!(
        "{}\\nНовых: {} | Повторено: {}\\n\\n🏠 На главную",
        mode_text, new_count, review_count
    );

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🏠 На главную",
        "back_to_main",
    )]]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;
    respond(())
}

fn format_card_front(card: &Card) -> String {
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
                "<b>{}</b>\\n\\n<b>Перевод:</b> {}\\n\\n<b>Примеры:</b>\\n日本語を勉強しています。\\n(Изучаю японский язык.)",
                question, answer
            )
        }
        Card::Kanji(_) => {
            format!("<b>{}</b>\\n\\n<b>Значения:</b> {}", question, answer)
        }
        Card::Grammar(grammar) => {
            format!(
                "<b>{}</b>\\n\\n<b>Кратко:</b> {}",
                grammar.title().text(),
                answer
            )
        }
    }
}

fn lesson_rating_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Не знаю ❌", "rating_again"),
        InlineKeyboardButton::callback("Плохо 😐", "rating_hard"),
        InlineKeyboardButton::callback("Знаю ✅", "rating_good"),
        InlineKeyboardButton::callback("Идеально 🌟", "rating_easy"),
    ]])
}
