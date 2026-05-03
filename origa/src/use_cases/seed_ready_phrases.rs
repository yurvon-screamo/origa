use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use tracing::info;
use ulid::Ulid;

use crate::dictionary::phrase::{get_all_index_ids, get_phrases_by_token, is_phrases_loaded};
use crate::domain::{Card, OrigaError, PhraseCard, StudyCard};
use crate::traits::UserRepository;

#[derive(Clone)]
pub struct SeedReadyPhrasesUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SeedReadyPhrasesUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<usize, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        if !is_phrases_loaded() {
            return Ok(0);
        }

        let orphaned_removed = remove_orphaned_phrase_cards(&mut user);

        let known_words = {
            let study_cards = user.knowledge_set().study_cards();
            collect_known_vocabulary_words(study_cards)
        };

        if known_words.is_empty() {
            if orphaned_removed > 0 {
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let current_hash = compute_known_words_hash(&known_words);
        if current_hash == user.known_vocab_hash() {
            if orphaned_removed > 0 {
                user.set_known_vocab_hash(current_hash);
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let study_cards = user.knowledge_set().study_cards();
        let existing_phrase_ids = collect_existing_phrase_ids(study_cards);
        let ready_phrase_ids = find_ready_phrases(&known_words, &existing_phrase_ids);

        let mut created_count = 0;
        for phrase_id in &ready_phrase_ids {
            let phrase_card = PhraseCard::new(*phrase_id);
            if user.create_card(Card::Phrase(phrase_card)).is_ok() {
                created_count += 1;
            }
        }

        user.set_known_vocab_hash(current_hash);
        self.repository.save(&user).await?;

        info!(
            created = created_count,
            orphaned_removed = orphaned_removed,
            known_vocab_count = known_words.len(),
            "Seeded ready phrase cards"
        );

        Ok(created_count)
    }
}

fn collect_known_vocabulary_words(study_cards: &HashMap<Ulid, StudyCard>) -> HashSet<String> {
    study_cards
        .values()
        .filter_map(|sc| {
            if let Card::Vocabulary(vocab) = sc.card() {
                if sc.memory().is_known_card() {
                    return Some(vocab.word().text().to_string());
                }
            }
            None
        })
        .collect()
}

fn compute_known_words_hash(known_words: &HashSet<String>) -> u32 {
    let mut words: Vec<&String> = known_words.iter().collect();
    words.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for word in &words {
        word.hash(&mut hasher);
    }
    hasher.finish() as u32
}

fn collect_existing_phrase_ids(study_cards: &HashMap<Ulid, StudyCard>) -> HashSet<Ulid> {
    study_cards
        .values()
        .filter_map(|sc| {
            if let Card::Phrase(phrase_card) = sc.card() {
                Some(*phrase_card.phrase_id())
            } else {
                None
            }
        })
        .collect()
}

fn remove_orphaned_phrase_cards(user: &mut crate::domain::User) -> usize {
    if !is_phrases_loaded() {
        return 0;
    }

    let valid_ids = get_all_index_ids();

    let orphaned_card_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .iter()
        .filter_map(|(card_id, sc)| {
            if let Card::Phrase(phrase_card) = sc.card() {
                if !valid_ids.contains(phrase_card.phrase_id()) {
                    return Some(*card_id);
                }
            }
            None
        })
        .collect();

    let count = orphaned_card_ids.len();
    if count > 0 {
        info!(count, "Removing orphaned phrase cards");
        for card_id in &orphaned_card_ids {
            if user.delete_card(*card_id).is_err() {
                tracing::warn!(%card_id, "Failed to delete orphaned phrase card");
            }
        }
    }
    count
}

fn find_ready_phrases(
    known_words: &HashSet<String>,
    existing_phrase_ids: &HashSet<Ulid>,
) -> Vec<Ulid> {
    let mut seen_phrase_ids: HashSet<Ulid> = HashSet::new();
    let mut result: Vec<Ulid> = Vec::new();

    for word in known_words {
        for entry in get_phrases_by_token(word) {
            let phrase_id = entry.id();
            if seen_phrase_ids.contains(phrase_id) || existing_phrase_ids.contains(phrase_id) {
                continue;
            }
            seen_phrase_ids.insert(*phrase_id);

            let all_tokens_known = entry
                .tokens()
                .iter()
                .all(|token| known_words.contains(token));

            if all_tokens_known {
                result.push(*phrase_id);
            }
        }
    }

    result
}
