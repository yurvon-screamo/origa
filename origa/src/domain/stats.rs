use super::{CardType, DailyHistoryItem, KnowledgeSet};

#[derive(Clone, Default)]
pub struct TodayOverview {
    pub new_count: usize,
    pub learned_count: usize,
    pub in_progress_count: usize,
    pub difficult_count: usize,
    pub new_delta: Option<i32>,
    pub learned_delta: Option<i32>,
    pub in_progress_delta: Option<i32>,
    pub difficult_delta: Option<i32>,
}

impl TodayOverview {
    pub fn total(&self) -> usize {
        self.new_count + self.learned_count + self.in_progress_count + self.difficult_count
    }
}

#[derive(Clone, Default)]
pub struct RatingRatio {
    pub percentage: u8,
    pub positive_count: usize,
    pub negative_count: usize,
}

pub fn compute_today_overview(
    knowledge_set: &KnowledgeSet,
    history: &[DailyHistoryItem],
) -> TodayOverview {
    let mut overview = TodayOverview::default();

    for study_card in knowledge_set.study_cards().values() {
        let card = study_card.card();
        if matches!(CardType::from(card), CardType::Phrase) {
            continue;
        }
        let memory = study_card.memory();
        if memory.is_new() {
            overview.new_count += 1;
        } else if memory.is_high_difficulty() {
            overview.difficult_count += 1;
        } else if memory.is_known_card() {
            overview.learned_count += 1;
        } else {
            overview.in_progress_count += 1;
        }
    }

    if history.len() >= 2 {
        let mut sorted: Vec<_> = history.to_vec();
        sorted.sort_by_key(|a| a.timestamp());
        let today = sorted.last().expect("last item exists");
        let yesterday = sorted
            .iter()
            .rev()
            .nth(1)
            .expect("second-to-last item exists");

        overview.new_delta = Some((today.new_words() as i64 - yesterday.new_words() as i64) as i32);
        overview.learned_delta =
            Some((today.known_words() as i64 - yesterday.known_words() as i64) as i32);
        overview.in_progress_delta =
            Some((today.in_progress_words() as i64 - yesterday.in_progress_words() as i64) as i32);
        overview.difficult_delta = Some(
            (today.high_difficulty_words() as i64 - yesterday.high_difficulty_words() as i64)
                as i32,
        );
    }

    overview
}

pub fn compute_rating_ratio(history: &[DailyHistoryItem]) -> Option<RatingRatio> {
    let mut positive = 0usize;
    let mut negative = 0usize;

    for item in history {
        positive += item.positive_ratings();
        negative += item.negative_ratings();
    }

    let total = positive + negative;
    if total == 0 {
        return None;
    }

    let percentage = (positive as f64 / total as f64 * 100.0).round() as u8;

    Some(RatingRatio {
        percentage,
        positive_count: positive,
        negative_count: negative,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Card, KnowledgeSet, RateMode, Rating, VocabularyCard, value_objects::Question,
    };

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    #[test]
    fn compute_today_overview_empty_data() {
        let ks = KnowledgeSet::new();
        let history: Vec<DailyHistoryItem> = vec![];

        let overview = compute_today_overview(&ks, &history);

        assert_eq!(overview.new_count, 0);
        assert_eq!(overview.learned_count, 0);
        assert_eq!(overview.in_progress_count, 0);
        assert_eq!(overview.difficult_count, 0);
        assert_eq!(overview.total(), 0);
        assert!(overview.new_delta.is_none());
        assert!(overview.learned_delta.is_none());
        assert!(overview.in_progress_delta.is_none());
        assert!(overview.difficult_delta.is_none());
    }

    #[test]
    fn compute_today_overview_with_cards_of_different_statuses() {
        let mut ks = KnowledgeSet::new();

        let _card1 = ks.create_card(create_vocab_card("猫")).unwrap();
        let _card2 = ks.create_card(create_vocab_card("犬")).unwrap();
        let card3 = ks.create_card(create_vocab_card("鳥")).unwrap();

        ks.mark_card_as_known(*card3.card_id()).unwrap();

        let overview = compute_today_overview(&ks, ks.lesson_history());

        assert_eq!(overview.new_count, 2);
        assert_eq!(overview.learned_count, 1);
        assert_eq!(overview.in_progress_count, 0);
        assert_eq!(overview.difficult_count, 0);
        assert_eq!(overview.total(), 3);
    }

    #[test]
    fn compute_rating_ratio_with_history() {
        let mut ks = KnowledgeSet::new();

        let card1 = ks.create_card(create_vocab_card("猫")).unwrap();
        let card2 = ks.create_card(create_vocab_card("犬")).unwrap();

        ks.rate_card(*card1.card_id(), Rating::Good, RateMode::ShortTerm)
            .unwrap();
        ks.rate_card(*card2.card_id(), Rating::Again, RateMode::ShortTerm)
            .unwrap();

        let result = compute_rating_ratio(ks.lesson_history());

        assert!(result.is_some());
        let ratio = result.unwrap();
        assert_eq!(ratio.positive_count, 1);
        assert_eq!(ratio.negative_count, 1);
        assert_eq!(ratio.percentage, 50);
    }

    #[test]
    fn compute_rating_ratio_empty_history() {
        let result = compute_rating_ratio(&[]);

        assert!(result.is_none());
    }
}
