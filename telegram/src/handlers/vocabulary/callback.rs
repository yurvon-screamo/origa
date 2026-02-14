use crate::dialogue::{DialogueState, SessionData};
use crate::formatters::format_japanese_text;
use crate::handlers::callbacks::CallbackData;
use crate::handlers::vocabulary::VocabularyCallback;
use crate::service::OrigaServiceProvider;
use teloxide::prelude::*;
use teloxide::types::MessageId;
use ulid::Ulid;

pub async fn vocabulary_callback_handler(
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
    let message_id = message.id();

    let Some(callback) = VocabularyCallback::try_from_json(&data) else {
        return respond(());
    };

    handle_vocabulary_callback(&bot, chat_id, message_id, callback, dialogue, session).await?;

    respond(())
}

async fn handle_vocabulary_callback(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    callback: VocabularyCallback,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    match callback {
        VocabularyCallback::Filter { filter } => {
            handle_filter(bot, chat_id, message_id, &filter, dialogue, session).await?;
        }
        VocabularyCallback::Page { page } => {
            handle_page(bot, chat_id, message_id, page, dialogue, session).await?;
        }
        VocabularyCallback::PageCurrent => {
            return respond(());
        }
        VocabularyCallback::Detail { card_id } => {
            handle_detail(bot, chat_id, card_id, session).await?;
        }
        VocabularyCallback::Delete { card_id } => {
            handle_delete_request_typed(bot, chat_id, message_id, card_id).await?;
        }
        VocabularyCallback::ConfirmDelete { card_id } => {
            handle_confirm_delete_typed(bot, chat_id, message_id, card_id, dialogue, session)
                .await?;
        }
        VocabularyCallback::CancelDelete => {
            handle_cancel_delete(bot, chat_id, message_id).await?;
        }
        VocabularyCallback::AddFromText => {
            handle_add_from_text(bot, chat_id, message_id, dialogue).await?;
        }
        VocabularyCallback::Search => {
            handle_search_request(bot, chat_id, message_id, dialogue).await?;
        }
        VocabularyCallback::SearchPage { page, query } => {
            handle_search_page(bot, chat_id, message_id, page, &query, dialogue).await?;
        }
        VocabularyCallback::SearchCurrent => {
            return respond(());
        }
        VocabularyCallback::BackToList => {
            handle_back_to_list(bot, chat_id, message_id, dialogue, session).await?;
        }
        VocabularyCallback::MainMenu => {
            return handle_main_menu(bot, chat_id, dialogue, session).await;
        }
        VocabularyCallback::Add { .. } => {
            bot.send_message(chat_id, "Функция добавления не реализована.")
                .await?;
        }
    }

    respond(())
}

async fn handle_filter(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    filter: &str,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let cards = crate::handlers::vocabulary::list::fetch_vocabulary_cards_for_page_change(
        provider,
        session.user_id,
    )
    .await?;
    let filtered_cards = crate::handlers::vocabulary::list::apply_filter_cards(&cards, filter);

    let items_per_page = 6;
    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = 0;

    let start = 0;
    let end = items_per_page.min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = crate::handlers::vocabulary::list::build_vocabulary_text_for_pagination(
        cards.len(),
        filter,
        page_cards,
        current_page,
        total_pages,
    );
    let keyboard = crate::handlers::vocabulary::list::build_vocabulary_keyboard(
        filter,
        page_cards,
        current_page,
        total_pages,
    );

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularyList {
            page: current_page,
            items_per_page,
            filter: filter.to_string(),
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

async fn handle_page(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    new_page: usize,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let current_state = dialogue.get().await.ok().flatten();
    let filter = match current_state {
        Some(DialogueState::VocabularyList { filter, .. }) => filter,
        _ => "all".to_string(),
    };

    let provider = OrigaServiceProvider::instance().await;

    let cards = crate::handlers::vocabulary::list::fetch_vocabulary_cards_for_page_change(
        provider,
        session.user_id,
    )
    .await?;
    let filtered_cards = crate::handlers::vocabulary::list::apply_filter_cards(&cards, &filter);

    let items_per_page = 6;
    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = new_page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = crate::handlers::vocabulary::list::build_vocabulary_text_for_pagination(
        cards.len(),
        &filter,
        page_cards,
        current_page,
        total_pages,
    );
    let keyboard = crate::handlers::vocabulary::list::build_vocabulary_keyboard(
        &filter,
        page_cards,
        current_page,
        total_pages,
    );

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularyList {
            page: current_page,
            items_per_page,
            filter,
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

async fn handle_detail(
    bot: &Bot,
    chat_id: ChatId,
    card_id: ulid::Ulid,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;
    let cards =
        crate::handlers::vocabulary::list::fetch_vocabulary_cards(provider, session.user_id)
            .await?;
    let Some((_, card)) = cards.iter().find(|(id, _)| *id == card_id) else {
        bot.send_message(chat_id, "Карточка не найдена.").await?;
        return respond(());
    };

    let text = format_card_detail(card);
    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![vec![
        teloxide::types::InlineKeyboardButton::callback(
            "🔙 Назад к списку",
            CallbackData::Vocabulary(VocabularyCallback::PageCurrent).to_json(),
        ),
    ]]);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    respond(())
}

fn format_card_detail(card: &origa::domain::StudyCard) -> String {
    let card_info = match card.card() {
        origa::domain::Card::Vocabulary(v) => {
            format!("<b>{}</b>", format_japanese_text(v.word().text()))
        }
        _ => String::from("Неизвестный тип карточки"),
    };

    let memory = card.memory();
    let next_review = memory
        .next_review_date()
        .map(|d| {
            let now = chrono::Utc::now();
            let diff = d.signed_duration_since(now);
            if diff.num_days() > 0 {
                format!("через {} дн.", diff.num_days())
            } else if diff.num_hours() > 0 {
                format!("через {} ч.", diff.num_hours())
            } else {
                "сегодня".to_string()
            }
        })
        .unwrap_or("нет данных".to_string());

    let reviews_count = memory.reviews().len();
    let difficulty = memory
        .difficulty()
        .map(|d| format!("{:.1}", d.value()))
        .unwrap_or_else(|| "-".to_string());
    let stability = memory
        .stability()
        .map(|s| format!("{:.0} дней", s.value()))
        .unwrap_or_else(|| "-".to_string());

    format!(
        r#"{} 📚 Детали карточки

<b>Слово:</b> {}
<b>Перевод:</b> {}

📊 Память:
• Следующий повтор: {}
• Кол-во повторов: {}
• Стабильность: {}
• Сложность: {}"#,
        card_info,
        format_japanese_text(card.card().question().text()),
        format_japanese_text(card.card().answer().text()),
        next_review,
        reviews_count,
        stability,
        difficulty
    )
}

async fn handle_delete_request_typed(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    card_id: ulid::Ulid,
) -> ResponseResult<()> {
    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![vec![
        teloxide::types::InlineKeyboardButton::callback(
            "✅ Да, удалить",
            CallbackData::Vocabulary(VocabularyCallback::ConfirmDelete { card_id }).to_json(),
        ),
        teloxide::types::InlineKeyboardButton::callback(
            "❌ Отмена",
            CallbackData::Vocabulary(VocabularyCallback::CancelDelete).to_json(),
        ),
    ]]);

    bot.edit_message_text(
        chat_id,
        message_id,
        "Вы уверены, что хотите удалить эту карточку?",
    )
    .reply_markup(keyboard)
    .await?;

    respond(())
}

async fn handle_confirm_delete_typed(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    card_id: ulid::Ulid,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let use_case = provider.delete_card_use_case();
    match use_case.execute(session.user_id, card_id).await {
        Ok(_) => {
            bot.edit_message_text(chat_id, message_id, "✅ Карточка удалена.")
                .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let current_state = dialogue.get().await.ok().flatten();
            let (page, filter) = match current_state {
                Some(DialogueState::VocabularyList { page, filter, .. }) => (page, filter),
                _ => (0, "all".to_string()),
            };

            let cards = crate::handlers::vocabulary::list::fetch_vocabulary_cards_for_page_change(
                provider,
                session.user_id,
            )
            .await?;
            let filtered_cards =
                crate::handlers::vocabulary::list::apply_filter_cards(&cards, &filter);

            let items_per_page = 6;
            let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
            let current_page = page.min(total_pages.saturating_sub(1));

            let start = current_page * items_per_page;
            let end = (start + items_per_page).min(filtered_cards.len());
            let page_cards = &filtered_cards[start..end];

            let text = crate::handlers::vocabulary::list::build_vocabulary_text_for_pagination(
                cards.len(),
                &filter,
                page_cards,
                current_page,
                total_pages,
            );
            let keyboard = crate::handlers::vocabulary::list::build_vocabulary_keyboard(
                &filter,
                page_cards,
                current_page,
                total_pages,
            );

            bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(keyboard)
                .await?;

            dialogue
                .update(DialogueState::VocabularyList {
                    page: current_page,
                    items_per_page,
                    filter,
                })
                .await
                .map_err(|e| {
                    teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                        e.to_string(),
                    )))
                })?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Ошибка при удалении: {}", e))
                .await?;
        }
    }

    respond(())
}

async fn handle_search_page(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    new_page: usize,
    query: &str,
    dialogue: crate::handlers::OrigaDialogue,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let cards =
        crate::handlers::vocabulary::list::fetch_vocabulary_cards(provider, Ulid::new()).await?;
    let query_lower = query.to_lowercase();

    let filtered_cards: Vec<_> = cards
        .into_iter()
        .filter(|(_, card)| {
            let card_text = match card.card() {
                origa::domain::Card::Vocabulary(v) => {
                    format!("{} {}", v.word().text(), v.meaning().text())
                }
                _ => String::new(),
            };
            card_text.to_lowercase().contains(&query_lower)
        })
        .collect();

    let items_per_page = 6;
    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = new_page.min(total_pages.saturating_sub(1));

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = build_search_results_text(query, current_page, total_pages, page_cards);
    let keyboard = build_search_results_keyboard(query, page_cards, current_page, total_pages);

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularySearch {
            page: current_page,
            items_per_page,
            query: query.to_string(),
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

async fn handle_back_to_list(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    handle_page(bot, chat_id, message_id, 0, dialogue, session).await
}

async fn handle_search_request(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    dialogue: crate::handlers::OrigaDialogue,
) -> ResponseResult<()> {
    bot.edit_message_text(
        chat_id,
        message_id,
        "🔍 Введите японское слово или его перевод для поиска...",
    )
    .await?;

    dialogue
        .update(DialogueState::VocabularySearch {
            page: 0,
            items_per_page: 6,
            query: "".to_string(),
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

async fn handle_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    dialogue.exit().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    crate::bot::messaging::send_main_menu_with_stats(
        bot,
        chat_id,
        &session.username,
        OrigaServiceProvider::instance().await,
        session.user_id,
        None,
    )
    .await
    .map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    respond(())
}

pub async fn handle_vocabulary_search(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: &crate::handlers::OrigaDialogue,
    session: SessionData,
    search_query: &str,
    _page: usize,
    items_per_page: usize,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let cards =
        crate::handlers::vocabulary::list::fetch_vocabulary_cards(provider, session.user_id)
            .await?;
    let query_lower = search_query.to_lowercase();

    let filtered_cards: Vec<_> = cards
        .into_iter()
        .filter(|(_, card)| {
            let card_text = match card.card() {
                origa::domain::Card::Vocabulary(v) => {
                    format!("{} {}", v.word().text(), v.meaning().text())
                }
                _ => String::new(),
            };
            card_text.to_lowercase().contains(&query_lower)
        })
        .collect();

    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = 0;

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(filtered_cards.len());
    let page_cards = &filtered_cards[start..end];

    let text = build_search_results_text(search_query, current_page, total_pages, page_cards);
    let keyboard =
        build_search_results_keyboard(search_query, page_cards, current_page, total_pages);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::VocabularySearch {
            page: current_page,
            items_per_page,
            query: search_query.to_string(),
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

pub fn build_search_results_text(
    query: &str,
    current_page: usize,
    total_pages: usize,
    page_cards: &[(ulid::Ulid, origa::domain::StudyCard)],
) -> String {
    let mut text = format!("🔍 Результаты поиска: \"{}\"\n\n", query);

    if page_cards.is_empty() {
        text.push_str("Ничего не найдено. Попробуйте другой запрос.");
    } else {
        for (idx, (_, card)) in page_cards.iter().enumerate() {
            let num = current_page * 6 + idx + 1;
            text.push_str(&format_search_card_entry(num, card));
            text.push('\n');
        }
    }

    if total_pages > 0 {
        text.push_str(&format!("\nСтраница {}/{}", current_page + 1, total_pages));
    }

    text
}

fn format_search_card_entry(num: usize, card: &origa::domain::StudyCard) -> String {
    match card.card() {
        origa::domain::Card::Vocabulary(v) => {
            format!(
                "<b>{}.</b> {} — {}",
                num,
                format_japanese_text(v.word().text()),
                format_japanese_text(v.meaning().text())
            )
        }
        _ => String::from("Неизвестный тип карточки"),
    }
}

pub fn build_search_results_keyboard(
    query: &str,
    page_cards: &[(ulid::Ulid, origa::domain::StudyCard)],
    current_page: usize,
    total_pages: usize,
) -> teloxide::types::InlineKeyboardMarkup {
    let mut rows = vec![];

    for (card_id, _) in page_cards {
        rows.push(vec![
            teloxide::types::InlineKeyboardButton::callback(
                "Подробнее",
                CallbackData::Vocabulary(VocabularyCallback::Detail { card_id: *card_id })
                    .to_json(),
            ),
            teloxide::types::InlineKeyboardButton::callback(
                "Удалить 🗑️",
                CallbackData::Vocabulary(VocabularyCallback::Delete { card_id: *card_id })
                    .to_json(),
            ),
        ]);
    }

    if total_pages > 1 {
        let mut pagination_row = vec![];

        if current_page > 0 {
            pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
                "⬅️ Назад",
                CallbackData::Vocabulary(VocabularyCallback::SearchPage {
                    page: current_page - 1,
                    query: query.to_string(),
                })
                .to_json(),
            ));
        }

        pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
            format!("{}/{}", current_page + 1, total_pages),
            CallbackData::Vocabulary(VocabularyCallback::SearchCurrent).to_json(),
        ));

        if current_page < total_pages - 1 {
            pagination_row.push(teloxide::types::InlineKeyboardButton::callback(
                "Далее ➡️",
                CallbackData::Vocabulary(VocabularyCallback::SearchPage {
                    page: current_page + 1,
                    query: query.to_string(),
                })
                .to_json(),
            ));
        }

        rows.push(pagination_row);
    }

    rows.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "🔙 Назад к списку",
        CallbackData::Vocabulary(VocabularyCallback::Page { page: 0 }).to_json(),
    )]);

    teloxide::types::InlineKeyboardMarkup::new(rows)
}

pub async fn handle_cancel_delete(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
) -> ResponseResult<()> {
    bot.edit_message_text(chat_id, message_id, "❌ Удаление отменено.")
        .await?;
    respond(())
}

pub async fn handle_add_from_text(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    dialogue: crate::handlers::OrigaDialogue,
) -> ResponseResult<()> {
    bot.edit_message_text(
        chat_id,
        message_id,
        "Отправьте японский текст — я выделю слова и предложу добавить.",
    )
    .await?;

    dialogue
        .update(DialogueState::AddFromText {
            pending_words: vec![],
        })
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}
