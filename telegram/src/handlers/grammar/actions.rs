use crate::dialogue::{DialogueState, SessionData};
use crate::handlers::OrigaDialogue;
use crate::service::OrigaServiceProvider;
use teloxide::prelude::*;
use teloxide::types::ChatId;
use ulid::Ulid;

use super::get_grammar_review_dates;
use super::grammar_list_keyboard;

pub async fn handle_grammar_add(
    bot: &Bot,
    chat_id: ChatId,
    rule_id: Ulid,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let use_case = provider.create_grammar_card_use_case();
    match use_case.execute(session.user_id, vec![rule_id]).await {
        Ok(_) => {
            bot.send_message(chat_id, "✅ Правило добавлено в ваш набор!")
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Ошибка: {}", e))
                .await?;
        }
    }

    send_grammar_list(bot, chat_id, dialogue, &session).await?;

    respond(())
}

pub async fn handle_grammar_delete(
    bot: &Bot,
    chat_id: ChatId,
    rule_id: Ulid,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let provider = OrigaServiceProvider::instance().await;

    let use_case = provider.delete_grammar_card_use_case();
    match use_case.execute(session.user_id, rule_id).await {
        Ok(_) => {
            bot.send_message(chat_id, "✅ Правило удалено из вашего набора!")
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Ошибка: {}", e))
                .await?;
        }
    }

    send_grammar_list(bot, chat_id, dialogue, &session).await?;

    respond(())
}

pub async fn handle_grammar_back_to_list(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: OrigaDialogue,
    session: SessionData,
) -> ResponseResult<()> {
    let state = dialogue.get().await.ok().flatten().unwrap_or_default();
    let (page, items_per_page) = match state {
        DialogueState::GrammarList {
            page,
            items_per_page,
        } => (page, items_per_page),
        _ => (0, 6),
    };

    let review_dates = get_grammar_review_dates(&session).await?;
    let text = "📖 Грамматика\n\nВыберите правило для просмотра:".to_string();
    let keyboard = grammar_list_keyboard(page, items_per_page, &review_dates);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    respond(())
}

pub async fn handle_grammar_search(
    bot: &Bot,
    chat_id: ChatId,
    message_id: teloxide::types::MessageId,
) -> ResponseResult<()> {
    bot.edit_message_text(
        chat_id,
        message_id,
        "🔍 Введите название правила или ключевое слово для поиска...",
    )
    .await?;

    respond(())
}

async fn send_grammar_list(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: OrigaDialogue,
    session: &SessionData,
) -> Result<(), teloxide::RequestError> {
    let state = dialogue.get().await.ok().flatten().unwrap_or_default();
    if let DialogueState::GrammarList {
        page,
        items_per_page,
    } = state
    {
        let review_dates = get_grammar_review_dates(session).await?;
        let text = "📖 Грамматика\n\nВыберите правило для просмотра:".to_string();
        let keyboard = grammar_list_keyboard(page, items_per_page, &review_dates);
        bot.send_message(chat_id, text)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}
