use origa::domain::DailyHistoryItem;

#[derive(Clone, Default)]
pub struct HomeStats {
    pub total_cards: usize,
    pub learned: usize,
    pub in_progress: usize,
    pub new: usize,
    pub high_difficulty: usize,
    pub total_cards_delta: isize,
    pub learned_delta: isize,
    pub in_progress_delta: isize,
    pub new_delta: isize,
    pub high_difficulty_delta: isize,
}

pub fn format_number(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

pub fn format_delta(delta: isize) -> String {
    if delta > 0 {
        format!("+{}", delta)
    } else if delta < 0 {
        delta.to_string()
    } else {
        String::new()
    }
}

pub fn calculate_stats(history: &[DailyHistoryItem]) -> HomeStats {
    if history.is_empty() {
        return HomeStats::default();
    }

    let last = history.last().unwrap();

    let (total_delta, learned_delta, in_progress_delta, new_delta, high_difficulty_delta) =
        if history.len() >= 2 {
            let prev = &history[history.len() - 2];
            (
                last.total_words() as isize - prev.total_words() as isize,
                last.known_words() as isize - prev.known_words() as isize,
                last.in_progress_words() as isize - prev.in_progress_words() as isize,
                last.new_words() as isize - prev.new_words() as isize,
                last.high_difficulty_words() as isize - prev.high_difficulty_words() as isize,
            )
        } else {
            (0, 0, 0, 0, 0)
        };

    HomeStats {
        total_cards: last.total_words(),
        learned: last.known_words(),
        in_progress: last.in_progress_words(),
        new: last.new_words(),
        high_difficulty: last.high_difficulty_words(),
        total_cards_delta: total_delta,
        learned_delta,
        in_progress_delta,
        new_delta,
        high_difficulty_delta,
    }
}
