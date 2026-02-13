use crate::bot::keyboard::{lesson_keyboard, reply_keyboard};
use crate::bot::messaging::send_history;
use crate::bot::messaging::send_main_menu_with_stats;
use crate::dialogue::{DialogueState, LessonMode, ProfileView};
use crate::handlers::OrigaDialogue;
use crate::handlers::SessionData;
use crate::handlers::grammar;
use crate::handlers::kanji;
use crate::handlers::lesson;
use crate::handlers::menu;
use crate::handlers::profile;
use crate::handlers::vocabulary;
use crate::service::OrigaServiceProvider;
use origa::domain::Card;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;
use ulid::Ulid;

pub async fn handle_menu_callback(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
    callback: menu::MenuCallback,
) -> ResponseResult<()> {
    match callback {
        menu::MenuCallback::MainMenu => {
            handle_menu_main_menu(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Vocabulary => {
            handle_menu_vocabulary(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Kanji => {
            handle_menu_kanji(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Grammar => {
            handle_menu_grammar(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Profile => {
            handle_menu_profile(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Settings => {
            handle_menu_settings(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Lesson => {
            handle_menu_lesson(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::Fixation => {
            handle_menu_fixation(bot, q, dialogue, session).await?;
        }
        menu::MenuCallback::HistoryKnown
        | menu::MenuCallback::HistoryInProgress
        | menu::MenuCallback::HistoryNew
        | menu::MenuCallback::HistoryHard
        | menu::MenuCallback::ShowHistory => {
            handle_menu_history(bot, q, session).await?;
        }
    }
    respond(())
}

async fn handle_menu_main_menu(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        dialogue.exit().await.map_err(|e| {
            teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
        })?;
        send_main_menu_with_stats(
            bot,
            chat_id,
            &session.username,
            OrigaServiceProvider::instance().await,
            session.user_id,
            Some(teloxide::types::ReplyMarkup::Keyboard(reply_keyboard())),
        )
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    }
    respond(())
}

async fn build_vocabulary_content(
    dialogue: &OrigaDialogue,
    session: &SessionData,
) -> ResponseResult<(
    (usize, usize, String),
    String,
    teloxide::types::InlineKeyboardMarkup,
)> {
    let (page, items_per_page, filter) = match dialogue.get().await.ok().flatten() {
        Some(DialogueState::VocabularyList {
            page,
            items_per_page,
            filter,
        }) => (page, items_per_page, filter),
        _ => (0, 6, "all".to_string()),
    };

    let state = dialogue.get().await.ok().flatten();
    let (text, keyboard) = match state {
        Some(DialogueState::VocabularyList {
            page,
            items_per_page,
            filter,
        }) => {
            let provider = OrigaServiceProvider::instance().await;
            let cards = vocabulary::list::fetch_vocabulary_cards(provider, session.user_id).await?;
            let filtered_cards = vocabulary::list::apply_filter(&cards, &filter);

            let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
            let current_page = page.min(total_pages.saturating_sub(1));

            let start = current_page * items_per_page;
            let end = (start + items_per_page).min(filtered_cards.len());
            let page_cards = &filtered_cards[start..end];

            let text = vocabulary::list::build_vocabulary_text(
                cards.len(),
                &filter,
                page_cards,
                current_page,
                total_pages,
            );
            let keyboard = vocabulary::list::build_vocabulary_keyboard(
                &filter,
                page_cards,
                current_page,
                total_pages,
            );
            (text, keyboard)
        }
        _ => {
            let provider = OrigaServiceProvider::instance().await;
            let cards = vocabulary::list::fetch_vocabulary_cards(provider, session.user_id).await?;

            let text = vocabulary::list::build_vocabulary_text(cards.len(), "all", &[], 0, 1);
            let keyboard = vocabulary::list::build_vocabulary_keyboard("all", &[], 0, 1);
            (text, keyboard)
        }
    };

    Ok(((page, items_per_page, filter), text, keyboard))
}

async fn handle_menu_vocabulary(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let ((page, items_per_page, filter), text, keyboard) =
            build_vocabulary_content(&dialogue, &session).await?;

        bot.edit_message_text(chat_id, q.message.as_ref().unwrap().id(), text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await?;

        dialogue
            .update(DialogueState::VocabularyList {
                page,
                items_per_page,
                filter,
            })
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
    }
    respond(())
}

async fn handle_menu_kanji(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let (page, items_per_page) = match dialogue.get().await.ok().flatten() {
            Some(DialogueState::KanjiList {
                page,
                items_per_page,
                ..
            }) => (page, items_per_page),
            _ => (0, 6),
        };

        let provider = OrigaServiceProvider::instance().await;
        let kanji_review_dates =
            kanji::list::fetch_kanji_review_dates(session.user_id, provider).await?;

        let kanji_list = kanji::list::get_kanji_by_level(None);
        let total_pages = (kanji_list.len() + items_per_page - 1) / items_per_page.max(1);
        let current_page = page.min(total_pages.saturating_sub(1));

        let start = current_page * items_per_page;
        let end = (start + items_per_page).min(kanji_list.len());
        let page_kanji = &kanji_list[start..end];

        let text = kanji::list::build_kanji_list_text(
            page_kanji,
            current_page,
            total_pages,
            None,
            &kanji_review_dates,
        );
        let keyboard = kanji::list::build_kanji_list_keyboard(
            page_kanji,
            current_page,
            total_pages,
            &kanji_review_dates,
        );

        bot.edit_message_text(chat_id, q.message.as_ref().unwrap().id(), text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await?;

        dialogue
            .update(DialogueState::KanjiList {
                level: None,
                page,
                items_per_page,
            })
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
    }
    respond(())
}

async fn handle_menu_grammar(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let (page, items_per_page) = match dialogue.get().await.ok().flatten() {
            Some(DialogueState::GrammarList {
                page,
                items_per_page,
                ..
            }) => (page, items_per_page),
            _ => (0, 6),
        };

        let review_dates = grammar::list::get_grammar_review_dates(&session).await?;
        let text = "📖 Грамматика\n\nВыберите правило для просмотра:".to_string();
        let keyboard = grammar::list::grammar_list_keyboard(page, items_per_page, &review_dates);

        bot.edit_message_text(chat_id, q.message.as_ref().unwrap().id(), text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await?;

        dialogue
            .update(DialogueState::GrammarList {
                page,
                items_per_page,
            })
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
    }
    respond(())
}

async fn handle_menu_profile(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    _session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let provider = OrigaServiceProvider::instance().await;
        let telegram_id = chat_id.0 as u64;

        let (profile, _) = match profile::load_user_profile(provider, telegram_id).await {
            Ok(p) => p,
            Err(_) => {
                bot.send_message(chat_id, "Ошибка загрузки профиля").await?;
                return respond(());
            }
        };

        let duolingo_status = if profile.duolingo_jwt_token.is_some() {
            "Подключено ✓"
        } else {
            "Не подключено"
        };

        let text = format!(
            "👤 Профиль\n\nИмя: {}\n\nЦелевой уровень JLPT: {}\n\n🔗 Duolingo: {}",
            profile.username,
            profile.current_japanese_level.code(),
            duolingo_status
        );

        bot.edit_message_text(chat_id, q.message.as_ref().unwrap().id(), text)
            .reply_markup(profile::profile_main_keyboard())
            .await?;

        dialogue
            .update(DialogueState::Profile {
                current_view: ProfileView::Main,
            })
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
    }
    respond(())
}

async fn handle_menu_settings(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    _session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let provider = OrigaServiceProvider::instance().await;
        let telegram_id = chat_id.0 as u64;

        let (profile, _) = match profile::load_user_profile(provider, telegram_id).await {
            Ok(p) => p,
            Err(_) => {
                bot.send_message(chat_id, "Ошибка загрузки настроек")
                    .await?;
                return respond(());
            }
        };

        let reminders_status = if profile.reminders_enabled {
            "Вкл"
        } else {
            "Выкл"
        };
        let text = format!("⚙️ Настройки\n\n• Напоминания: {}", reminders_status);

        bot.edit_message_text(chat_id, q.message.as_ref().unwrap().id(), text)
            .reply_markup(profile::settings_keyboard(profile.reminders_enabled))
            .await?;

        dialogue
            .update(DialogueState::Profile {
                current_view: ProfileView::Settings,
            })
            .await
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;
    }
    respond(())
}

async fn handle_menu_lesson(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let _telegram_id = chat_id.0 as u64;
        let provider = OrigaServiceProvider::instance().await;

        let use_case = provider.select_cards_to_lesson_use_case();
        let cards_result = use_case.execute(session.user_id).await;

        let cards: HashMap<Ulid, Card> = match cards_result {
            Ok(cards) => cards,
            Err(_) => {
                return respond(());
            }
        };

        if cards.is_empty() {
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
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;

        let lesson_start_text = format!(
            "{}\n{}: {}\n{}: 0/{}",
            lesson::LessonCallback::LESSON_STARTED,
            lesson::LessonCallback::CARDS,
            total_cards,
            lesson::LessonCallback::PROGRESS,
            total_cards
        );

        bot.send_message(chat_id, lesson_start_text).await?;

        if let Some(first_card_id) = card_ids.first()
            && let Some(first_card) = cards.get(first_card_id)
        {
            let card_text = lesson::format_card_front(first_card);
            let keyboard = ReplyMarkup::Keyboard(lesson_keyboard());
            bot.send_message(chat_id, card_text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(keyboard)
                .await?;
        }
    }
    respond(())
}

async fn handle_menu_fixation(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        let _telegram_id = chat_id.0 as u64;
        let provider = OrigaServiceProvider::instance().await;

        let use_case = provider.select_cards_to_fixation_use_case();
        let cards_result = use_case.execute(session.user_id).await;

        let cards: HashMap<Ulid, Card> = match cards_result {
            Ok(cards) => cards,
            Err(_) => {
                return respond(());
            }
        };

        if cards.is_empty() {
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
            .map_err(|e| {
                teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
            })?;

        let lesson_start_text = format!(
            "{}\n{}: {}\n{}: 0/{}",
            lesson::LessonCallback::FIXATION_STARTED,
            lesson::LessonCallback::CARDS,
            total_cards,
            lesson::LessonCallback::PROGRESS,
            total_cards
        );

        bot.send_message(chat_id, lesson_start_text).await?;

        if let Some(first_card_id) = card_ids.first()
            && let Some(first_card) = cards.get(first_card_id)
        {
            let card_text = lesson::format_card_front(first_card);
            let keyboard = ReplyMarkup::Keyboard(lesson_keyboard());
            bot.send_message(chat_id, card_text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(keyboard)
                .await?;
        }
    }
    respond(())
}

async fn handle_menu_history(
    bot: &Bot,
    q: &CallbackQuery,
    session: SessionData,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    if let Some(chat_id) = chat_id {
        send_history(
            bot,
            chat_id,
            session.user_id,
            OrigaServiceProvider::instance().await,
        )
        .await
        .map_err(|e| teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;
    }
    respond(())
}
