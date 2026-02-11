use crate::service::OrigaServiceProvider;
use chrono::{Duration, Utc};

pub struct UserStatistics {
    pub total: usize,
    pub known: usize,
    pub in_progress: usize,
    pub due_today: usize,
    pub new: usize,
    pub hard: usize,
}

impl Default for UserStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStatistics {
    pub fn new() -> Self {
        Self {
            total: 0,
            known: 0,
            in_progress: 0,
            due_today: 0,
            new: 0,
            hard: 0,
        }
    }
}

pub async fn get_user_statistics(
    provider: &OrigaServiceProvider,
    user_id: ulid::Ulid,
) -> Result<UserStatistics, origa::domain::OrigaError> {
    let use_case = provider.knowledge_set_cards_use_case();
    let cards = use_case.execute(user_id).await?;

    let mut stats = UserStatistics::new();
    stats.total = cards.len();

    for card in cards {
        let memory = card.memory();

        if memory.is_known_card() {
            stats.known += 1;
        } else if memory.is_in_progress() {
            stats.in_progress += 1;
            if memory.is_due() {
                stats.due_today += 1;
            }
        } else if memory.is_new() {
            stats.new += 1;
        } else if memory.is_high_difficulty() {
            stats.hard += 1;
        }
    }

    Ok(stats)
}

pub async fn get_progress_history(
    user_id: ulid::Ulid,
    provider: &OrigaServiceProvider,
    metric: &str,
) -> Result<String, origa::domain::OrigaError> {
    let use_case = provider.knowledge_set_cards_use_case();
    let cards = use_case.execute(user_id).await?;

    let now = Utc::now();
    let today = now.date_naive();

    let mut counts_by_day: Vec<(String, usize)> = Vec::new();
    let days: usize = 7;

    for day_offset in (0..days).rev() {
        let _target_date = today - Duration::days(day_offset as i64);
        let day_label = if day_offset == 0 {
            "Сегодня".to_string()
        } else {
            format!("День {}", days - day_offset)
        };

        let count = cards
            .iter()
            .filter(|card| match metric {
                "known" => card.memory().is_known_card(),
                "in_progress" => card.memory().is_in_progress(),
                "hard" => card.memory().is_high_difficulty(),
                "new" => card.memory().is_new(),
                _ => false,
            })
            .count();

        counts_by_day.push((day_label, count));
    }

    let metric_name = match metric {
        "known" => "Изучено",
        "in_progress" => "В процессе",
        "hard" => "Сложные",
        "new" => "Новые",
        _ => "Прогресс",
    };

    let mut text = format!(r#"История "{}":"#, metric_name);
    for (day_label, count) in &counts_by_day {
        text.push_str(&format!("{}: {}\\n", day_label, count));
    }

    Ok(text)
}
