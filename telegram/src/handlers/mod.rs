use crate::bot::messaging::send_history;
use crate::bot::{keyboard::reply_keyboard, messaging::send_main_menu_with_stats};
use crate::dialogue::{self, DialogueState, LessonMode, ProfileView};
use crate::service::OrigaServiceProvider;
use callbacks::CallbackData;
use lesson::{handle_lesson_callback, start_fixation, start_lesson};
use origa::domain::Card;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::{dispatching::dialogue::InMemStorage, utils::command::BotCommands};
use ulid::Ulid;

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
pub use grammar::grammar_list_handler;
pub use kanji::handle_kanji_list;
pub use vocabulary::vocabulary_list_handler;
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
                let provider = OrigaServiceProvider::instance().await;

                let session = provider.get_or_create_session(telegram_id, "User").await?;

                start_lesson(bot, msg, dialogue, session).await?;
            }
            "🔒 Закрепление" => {
                let telegram_id = msg.chat.id.0 as u64;
                let provider = OrigaServiceProvider::instance().await;

                let session = provider.get_or_create_session(telegram_id, "User").await?;

                start_fixation(bot, msg, dialogue, session).await?;
            }
            "📚 Слова" => {
                let telegram_id = msg.chat.id.0 as u64;
                let username = msg
                    .from
                    .as_ref()
                    .map(|u| u.first_name.as_str())
                    .unwrap_or("User");
                let provider = OrigaServiceProvider::instance().await;
                let session = provider
                    .get_or_create_session(telegram_id, username)
                    .await?;

                let (page, items_per_page, filter) = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::VocabularyList {
                        page,
                        items_per_page,
                        filter,
                    }) => (page, items_per_page, filter),
                    _ => (0, 6, "all".to_string()),
                };

                vocabulary_list_handler(
                    bot,
                    msg,
                    dialogue,
                    (page, items_per_page, filter),
                    session,
                )
                .await?;
            }
            "🈷 Кандзи" => {
                let telegram_id = msg.chat.id.0 as u64;
                let username = msg
                    .from
                    .as_ref()
                    .map(|u| u.first_name.as_str())
                    .unwrap_or("User");
                let provider = OrigaServiceProvider::instance().await;
                let session = provider
                    .get_or_create_session(telegram_id, username)
                    .await?;

                let (page, items_per_page) = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::KanjiList {
                        page,
                        items_per_page,
                        ..
                    }) => (page, items_per_page),
                    _ => (0, 6),
                };

                handle_kanji_list(bot, msg, (page, items_per_page), session).await?;
            }
            "📖 Грамматика" => {
                let telegram_id = msg.chat.id.0 as u64;
                let username = msg
                    .from
                    .as_ref()
                    .map(|u| u.first_name.as_str())
                    .unwrap_or("User");
                let provider = OrigaServiceProvider::instance().await;
                let session = provider
                    .get_or_create_session(telegram_id, username)
                    .await?;

                let (page, items_per_page) = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::GrammarList {
                        page,
                        items_per_page,
                        ..
                    }) => (page, items_per_page),
                    _ => (0, 6),
                };

                grammar_list_handler(bot, msg, dialogue, (page, items_per_page), session).await?;
            }
            "👤 Профиль" => {
                let current_view = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::Profile { current_view }) => current_view,
                    _ => ProfileView::Main,
                };

                profile_handler(bot, msg, DialogueState::Profile { current_view }).await?;
            }
            "⚙️ Настройки" => {
                profile_handler(
                    bot,
                    msg,
                    DialogueState::Profile {
                        current_view: ProfileView::Settings,
                    },
                )
                .await?;
            }
            "🏠 Главная" => {
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
    dialogue: OrigaDialogue,
    session: SessionData,
    callback: menu::MenuCallback,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    match callback {
        menu::MenuCallback::MainMenu => {
            if let Some(chat_id) = chat_id {
                dialogue.exit().await.map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
                crate::bot::messaging::send_main_menu_with_stats(
                    bot,
                    chat_id,
                    &session.username,
                    OrigaServiceProvider::instance().await,
                    session.user_id,
                    Some(teloxide::types::ReplyMarkup::Keyboard(reply_keyboard())),
                )
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
            }
        }
        menu::MenuCallback::Vocabulary => {
            if let Some(chat_id) = chat_id {
                let (page, items_per_page, filter) = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::VocabularyList {
                        page,
                        items_per_page,
                        filter,
                    }) => (page, items_per_page, filter),
                    _ => (0, 6, "all".to_string()),
                };

                let text = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::VocabularyList {
                        page,
                        items_per_page,
                        filter,
                    }) => {
                        let provider = OrigaServiceProvider::instance().await;
                        let cards =
                            vocabulary::list::fetch_vocabulary_cards(provider, session.user_id)
                                .await?;
                        let filtered_cards = vocabulary::list::apply_filter(&cards, &filter);

                        let total_pages =
                            (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
                        let current_page = page.min(total_pages.saturating_sub(1));

                        let start = current_page * items_per_page;
                        let end = (start + items_per_page).min(filtered_cards.len());
                        let page_cards = &filtered_cards[start..end];

                        vocabulary::list::build_vocabulary_text(
                            cards.len(),
                            &filter,
                            page_cards,
                            current_page,
                            total_pages,
                        )
                    }
                    _ => {
                        let provider = OrigaServiceProvider::instance().await;
                        let cards =
                            vocabulary::list::fetch_vocabulary_cards(provider, session.user_id)
                                .await?;

                        vocabulary::list::build_vocabulary_text(cards.len(), "all", &[], 0, 1)
                    }
                };

                let keyboard = match dialogue.get().await.ok().flatten() {
                    Some(DialogueState::VocabularyList {
                        page,
                        items_per_page,
                        filter,
                    }) => {
                        let provider = OrigaServiceProvider::instance().await;
                        let cards =
                            vocabulary::list::fetch_vocabulary_cards(provider, session.user_id)
                                .await?;
                        let filtered_cards = vocabulary::list::apply_filter(&cards, &filter);

                        let total_pages =
                            (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
                        let current_page = page.min(total_pages.saturating_sub(1));

                        let start = current_page * items_per_page;
                        let end = (start + items_per_page).min(filtered_cards.len());
                        let page_cards = &filtered_cards[start..end];

                        vocabulary::list::build_vocabulary_keyboard(
                            &filter,
                            page_cards,
                            current_page,
                            total_pages,
                        )
                    }
                    _ => vocabulary::list::build_vocabulary_keyboard("all", &[], 0, 1),
                };

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
        }
        menu::MenuCallback::Kanji => {
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
        }
        menu::MenuCallback::Grammar => {
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
                let keyboard =
                    grammar::list::grammar_list_keyboard(page, items_per_page, &review_dates);

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
        }
        menu::MenuCallback::Profile => {
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
        }
        menu::MenuCallback::Settings => {
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
        }
        menu::MenuCallback::Lesson => {
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
                    "{}\\n{}: {}\\n{}: 0/{}",
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
                    let keyboard = lesson::lesson_rating_keyboard();
                    bot.send_message(chat_id, card_text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(keyboard)
                        .await?;
                }
            }
        }
        menu::MenuCallback::Fixation => {
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
                    "{}\\n{}: {}\\n{}: 0/{}",
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
                    let keyboard = lesson::lesson_rating_keyboard();
                    bot.send_message(chat_id, card_text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(keyboard)
                        .await?;
                }
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
                    OrigaServiceProvider::instance().await,
                )
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;
            }
        }
    }
    respond(())
}
