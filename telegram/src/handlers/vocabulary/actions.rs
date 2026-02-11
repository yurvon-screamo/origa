use crate::telegram_domain::DialogueState;
use crate::telegram_domain::SessionData;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use ulid::Ulid;

pub async fn handle_delete_request(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
    data: &str,
    _session: SessionData,
) -> ResponseResult<()> {
    let card_id_str = data.strip_prefix("vocab_delete_").unwrap_or("");

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            "✅ Да, удалить",
            format!("vocab_confirm_delete_{}", card_id_str),
        ),
        InlineKeyboardButton::callback("❌ Отмена", "vocab_cancel_delete"),
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

pub async fn handle_confirm_delete(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
    data: &str,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let card_id_str = data.strip_prefix("vocab_confirm_delete_").unwrap_or("");
    let Ok(card_id) = Ulid::from_string(card_id_str) else {
        bot.send_message(chat_id, "Ошибка: неверный ID карточки.")
            .await?;
        return respond(());
    };

    let repository = crate::repository::build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let use_case = origa::application::use_cases::DeleteCardUseCase::new(&repository);
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
                &repository,
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

pub async fn handle_cancel_delete(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
) -> ResponseResult<()> {
    bot.edit_message_text(chat_id, message_id, "❌ Удаление отменено.")
        .await?;
    respond(())
}

pub async fn handle_add_from_text(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
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
