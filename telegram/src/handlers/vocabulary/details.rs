use crate::handlers::vocabulary::list::fetch_vocabulary_cards;
use crate::repository::OrigaServiceProvider;
use crate::telegram_domain::SessionData;
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

    let provider = OrigaServiceProvider::instance();

    let cards = fetch_vocabulary_cards(provider, session.user_id).await?;
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
        card.card().question().text(),
        card.card().answer().text(),
        next_review,
        reviews_count,
        stability,
        difficulty
    )
}
