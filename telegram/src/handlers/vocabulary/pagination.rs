use crate::telegram_domain::DialogueState;
use crate::telegram_domain::SessionData;
use teloxide::prelude::*;

pub async fn handle_filter_change(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
    data: &str,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let filter = data
        .strip_prefix("vocab_filter_")
        .unwrap_or("all")
        .to_string();

    let repository = crate::repository::build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let cards = crate::handlers::vocabulary::list::fetch_vocabulary_cards_for_page_change(
        &repository,
        session.user_id,
    )
    .await?;
    let filtered_cards = crate::handlers::vocabulary::list::apply_filter_cards(&cards, &filter);

    let items_per_page = 6;
    let total_pages = (filtered_cards.len() + items_per_page - 1) / items_per_page.max(1);
    let current_page = 0;

    let start = 0;
    let end = items_per_page.min(filtered_cards.len());
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

pub async fn handle_page_change(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
    data: &str,
    dialogue: crate::handlers::OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    if data == "vocab_page_current" {
        return respond(());
    }

    let new_page: usize = data
        .strip_prefix("vocab_page_")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let current_state = dialogue.get().await.ok().flatten();
    let filter = match current_state {
        Some(DialogueState::VocabularyList { filter, .. }) => filter,
        _ => "all".to_string(),
    };

    let repository = crate::repository::build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let cards = crate::handlers::vocabulary::list::fetch_vocabulary_cards_for_page_change(
        &repository,
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
