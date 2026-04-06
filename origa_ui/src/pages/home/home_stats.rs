use origa::domain::DailyHistoryItem;

#[derive(Clone, Default)]
pub struct PrimaryStats {
    pub learned: usize,
    pub learned_delta: isize,
    pub in_progress: usize,
    pub in_progress_delta: isize,
    pub new: usize,
    pub new_delta: isize,
}

#[derive(Clone, Default)]
pub struct SecondaryStats {
    pub high_difficulty: usize,
    pub high_difficulty_delta: isize,
    pub positive: usize,
    pub positive_delta: isize,
    pub negative: usize,
    pub negative_delta: isize,
    pub total_ratings: usize,
    pub total_ratings_delta: isize,
}

#[derive(Clone, Default)]
pub struct StatsDeltas {
    pub learned: isize,
    pub in_progress: isize,
    pub new: isize,
    pub high_difficulty: isize,
    pub positive: isize,
    pub negative: isize,
    pub total_ratings: isize,
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

pub fn calculate_stats(history: &[DailyHistoryItem]) -> (PrimaryStats, SecondaryStats) {
    let last = match history.last() {
        Some(item) => item,
        None => return (PrimaryStats::default(), SecondaryStats::default()),
    };

    let deltas = compute_deltas(history, last);

    (
        PrimaryStats {
            learned: last.known_words(),
            learned_delta: deltas.learned,
            in_progress: last.in_progress_words(),
            in_progress_delta: deltas.in_progress,
            new: last.new_words(),
            new_delta: deltas.new,
        },
        SecondaryStats {
            high_difficulty: last.high_difficulty_words(),
            high_difficulty_delta: deltas.high_difficulty,
            positive: last.positive_ratings(),
            positive_delta: deltas.positive,
            negative: last.negative_ratings(),
            negative_delta: deltas.negative,
            total_ratings: last.total_ratings(),
            total_ratings_delta: deltas.total_ratings,
        },
    )
}

fn compute_deltas(history: &[DailyHistoryItem], last: &DailyHistoryItem) -> StatsDeltas {
    if history.len() < 2 {
        return StatsDeltas::default();
    }
    let prev = &history[history.len() - 2];
    StatsDeltas {
        learned: last.known_words() as isize - prev.known_words() as isize,
        in_progress: last.in_progress_words() as isize - prev.in_progress_words() as isize,
        new: last.new_words() as isize - prev.new_words() as isize,
        high_difficulty: last.high_difficulty_words() as isize
            - prev.high_difficulty_words() as isize,
        positive: last.positive_ratings() as isize - prev.positive_ratings() as isize,
        negative: last.negative_ratings() as isize - prev.negative_ratings() as isize,
        total_ratings: last.total_ratings() as isize - prev.total_ratings() as isize,
    }
}
