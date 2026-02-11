use crate::handlers::vocabulary::list::fetch_vocabulary_cards;
use crate::telegram_domain::SessionData;
use chrono::{Datelike, TimeDelta};
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;
use ulid::Ulid;

pub async fn handle_show_detail(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    data: &str,
    session: SessionData,
) -> ResponseResult<()> {
    let card_id_str = data.strip_prefix("vocab_detail_").unwrap_or("");
    let Ok(card_id) = Ulid::from_string(card_id_str) else {
        bot.send_message(chat_id, "Ошибка: неверный ID карточки.")
            .await?;
        return respond(());
    };

    let repository = crate::repository::build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let cards = fetch_vocabulary_cards(&repository, session.user_id).await?;
    let Some((_, card)) = cards.iter().find(|(id, _)| *id == card_id) else {
        bot.send_message(chat_id, "Карточка не найдена.").await?;
        return respond(());
    };

    let text = format_card_detail(card);
    let keyboard =
        InlineKeyboardMarkup::new(vec![vec![teloxide::types::InlineKeyboardButton::callback(
            "🔙 Назад к списку",
            "vocab_page_current",
        )]]);

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    respond(())
}

fn format_card_detail(card: &origa::domain::StudyCard) -> String {
    let card_info = match card.card() {
        origa::domain::Card::Vocabulary(v) => format!("<b>{}</b>", v.word().text()),
        _ => String::from("Неизвестный тип карточки"),
    };

    let memory = card.memory();

    let meaning = match card.card() {
        origa::domain::Card::Vocabulary(v) => v.meaning().text().to_string(),
        _ => String::from("-"),
    };

    let next_review = memory
        .next_review_date()
        .map(format_date)
        .unwrap_or_else(|| "сегодня".to_string());

    let difficulty = memory
        .difficulty()
        .map(|d| format!("{:.1}", d.value()))
        .unwrap_or_else(|| "-".to_string());

    let stability = memory
        .stability()
        .map(|s| format!("{:.1}", s.value()))
        .unwrap_or_else(|| "-".to_string());

    let reviews_count = memory.reviews().len();

    let status = if memory.is_new() {
        "Новая"
    } else if memory.is_high_difficulty() {
        "Сложная"
    } else if memory.is_known_card() {
        "Изучена"
    } else if memory.is_in_progress() {
        "В процессе"
    } else {
        "Неизвестно"
    };

    format!(
        "{}\\n\\n<b>Перевод:</b> {}\\n\\n<b>Статус:</b> {}\\n<b>Повтор:</b> {}\\n<b>Сложность:</b> {}\\n<b>Стабильность:</b> {}\\n<b>Всего повторений:</b> {}",
        card_info, meaning, status, next_review, difficulty, stability, reviews_count
    )
}

fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let today = now.date_naive();
    let date_naive = date.date_naive();

    if date_naive == today {
        "сегодня".to_string()
    } else if date_naive == today + TimeDelta::days(1) {
        "завтра".to_string()
    } else if date_naive < today {
        "просрочено".to_string()
    } else {
        format!(
            "{}.{}.{}",
            date_naive.day(),
            date_naive.month(),
            date_naive.year()
        )
    }
}
