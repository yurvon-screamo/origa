use crate::bot::keyboard::{history_keyboard, main_menu_keyboard_with_stats};
use crate::bot::statistics::{get_progress_history, get_user_statistics};
use anyhow::Result;
use origa::infrastructure::FileSystemUserRepository;
use teloxide::prelude::*;
use teloxide::types::{ChatId, ReplyMarkup};
use ulid::Ulid;

pub async fn send_main_menu_with_stats(
    bot: &teloxide::Bot,
    chat_id: ChatId,
    username: &str,
    repository: &FileSystemUserRepository,
    user_id: Ulid,
    reply_markup: Option<ReplyMarkup>,
) -> Result<()> {
    let stats = get_user_statistics(repository, user_id).await?;

    let text = format!(
        r#"👋 Привет, {}!

📊 Статистика:
• Всего карточек: {}
• Изучено: {}
• В процессе: {} (нужно повторить сегодня: {})
• Новые: {}
• Сложные: {}

Готов учиться?"#,
        username,
        stats.total,
        stats.known,
        stats.in_progress,
        stats.due_today,
        stats.new,
        stats.hard
    );

    let keyboard = main_menu_keyboard_with_stats();
    let mut msg = bot
        .send_message(chat_id, text)
        .reply_markup(ReplyMarkup::InlineKeyboard(keyboard));
    if let Some(markup) = reply_markup {
        msg = msg.reply_markup(markup);
    }
    msg.await?;
    Ok(())
}

pub async fn send_history(
    bot: &teloxide::Bot,
    chat_id: ChatId,
    user_id: Ulid,
    repository: &FileSystemUserRepository,
) -> Result<()> {
    let history = get_progress_history(user_id, repository, "known").await?;
    bot.send_message(chat_id, history)
        .reply_markup(ReplyMarkup::InlineKeyboard(history_keyboard()))
        .await?;
    Ok(())
}
