use origa::domain::DailyHistoryItem;

#[derive(Clone, Default)]
pub struct HomeStats {
    pub total_cards: usize,
    pub learned: usize,
    pub in_progress: usize,
    pub new: usize,
    pub high_difficulty: usize,
    pub weekly_delta: usize,
}

pub fn format_number(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

pub fn calculate_stats(history: &[DailyHistoryItem]) -> HomeStats {
    if history.is_empty() {
        return HomeStats::default();
    }

    let last = history.last().unwrap();
    let weekly_delta = if history.len() >= 2 {
        let prev = &history[history.len() - 2];
        last.total_words().saturating_sub(prev.total_words())
    } else {
        0
    };

    HomeStats {
        total_cards: last.total_words(),
        learned: last.known_words(),
        in_progress: last.in_progress_words(),
        new: last.new_words(),
        high_difficulty: last.high_difficulty_words(),
        weekly_delta,
    }
}
