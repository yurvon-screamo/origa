use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use tracing::{debug, info};
use ulid::Ulid;

use crate::dictionary::kanji::get_kanji_info;
use crate::domain::{Card, OrigaError};
use crate::traits::UserRepository;

static MIGRATION_CUTOFF_DATE: OnceLock<DateTime<Utc>> = OnceLock::new();

fn cutoff_date() -> DateTime<Utc> {
    *MIGRATION_CUTOFF_DATE.get_or_init(|| {
        DateTime::parse_from_rfc3339("2025-05-19T00:00:00Z")
            .expect("valid cutoff date")
            .to_utc()
    })
}

pub struct CleanupResult {
    pub cards_scanned: usize,
    pub cards_removed: usize,
    pub cards_kept_has_progress: usize,
}

#[derive(Clone)]
pub struct CleanupKanjiCompanionsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CleanupKanjiCompanionsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<CleanupResult, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let kanji_popular_words = collect_current_popular_words(user.knowledge_set());

        let vocab_cards = collect_vocab_cards(user.knowledge_set());

        let mut to_delete: Vec<Ulid> = Vec::new();
        let mut cards_kept_has_progress = 0;

        for (card_id, word_text, study_card) in &vocab_cards {
            let matched_kanji = find_kanji_in_word(word_text, &kanji_popular_words);

            if matched_kanji.is_empty() {
                continue;
            }

            let is_in_current_popular = matched_kanji.iter().any(|kanji| {
                kanji_popular_words
                    .get(kanji)
                    .is_some_and(|set| set.contains(word_text.as_str()))
            });

            if is_in_current_popular {
                continue;
            }

            if is_migration_created(study_card) {
                debug!(
                    word = %word_text,
                    card_id = %card_id,
                    "Scheduling orphaned companion for deletion"
                );
                to_delete.push(*card_id);
            } else {
                cards_kept_has_progress += 1;
            }
        }

        let cards_scanned = vocab_cards.len();
        let cards_removed = to_delete.len();

        for card_id in &to_delete {
            user.delete_card(*card_id)?;
        }

        if cards_removed > 0 {
            self.repository.save_sync(&user).await?;
        }

        info!(
            cards_scanned,
            cards_removed, cards_kept_has_progress, "Kanji companion cleanup completed"
        );

        Ok(CleanupResult {
            cards_scanned,
            cards_removed,
            cards_kept_has_progress,
        })
    }
}

fn collect_current_popular_words(
    knowledge_set: &crate::domain::KnowledgeSet,
) -> HashMap<char, HashSet<&'static str>> {
    let mut result = HashMap::new();

    for study_card in knowledge_set.study_cards().values() {
        let Card::Kanji(kanji_card) = study_card.card() else {
            continue;
        };

        let kanji_char = kanji_card.kanji().text();
        let Some(kanji_ch) = kanji_char.chars().next() else {
            continue;
        };

        let Ok(kanji_info) = get_kanji_info(kanji_char) else {
            continue;
        };

        let popular: HashSet<&'static str> = kanji_info
            .popular_words()
            .iter()
            .map(|s| s.as_str())
            .collect();

        result.insert(kanji_ch, popular);
    }

    result
}

fn collect_vocab_cards(
    knowledge_set: &crate::domain::KnowledgeSet,
) -> Vec<(Ulid, String, crate::domain::StudyCard)> {
    knowledge_set
        .study_cards()
        .values()
        .filter_map(|study_card| {
            let Card::Vocabulary(vocab_card) = study_card.card() else {
                return None;
            };
            Some((
                *study_card.card_id(),
                vocab_card.word().text().to_string(),
                study_card.clone(),
            ))
        })
        .collect()
}

fn find_kanji_in_word(
    word: &str,
    kanji_popular_words: &HashMap<char, HashSet<&str>>,
) -> Vec<char> {
    word.chars()
        .filter(|ch| kanji_popular_words.contains_key(ch))
        .collect()
}

fn is_migration_created(study_card: &crate::domain::StudyCard) -> bool {
    if study_card.is_new() {
        return true;
    }

    study_card
        .memory()
        .reviews()
        .iter()
        .all(|review| review.timestamp() > cutoff_date())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Card, StudyCard, VocabularyCard, value_objects::Question};

    #[test]
    fn is_migration_created_true_for_new_card() {
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new("日曜日".to_string()).unwrap(),
        ));
        let study_card = StudyCard::new(card);

        assert!(is_migration_created(&study_card));
    }

    #[test]
    fn is_migration_created_true_when_all_reviews_after_cutoff() {
        let mut study_card = StudyCard::new(Card::Vocabulary(VocabularyCard::new(
            Question::new("日曜日".to_string()).unwrap(),
        )));

        let state = crate::domain::MemoryState::new(
            crate::domain::Stability::new(5.0).unwrap(),
            crate::domain::Difficulty::new(3.0).unwrap(),
            Utc::now(),
        );
        let review =
            crate::domain::ReviewLog::new(crate::domain::Rating::Good, chrono::Duration::days(1));
        study_card.add_review(state, review);

        assert!(is_migration_created(&study_card));
    }

    #[test]
    fn find_kanji_in_word_returns_matching_kanji() {
        let mut map = HashMap::new();
        map.insert('日', HashSet::new());
        map.insert('月', HashSet::new());

        let result = find_kanji_in_word("日曜日", &map);
        assert!(result.contains(&'日'));
        assert!(!result.contains(&'月'));
    }

    #[test]
    fn find_kanji_in_word_returns_empty_for_no_match() {
        let mut map = HashMap::new();
        map.insert('山', HashSet::new());

        let result = find_kanji_in_word("日曜日", &map);
        assert!(result.is_empty());
    }
}
