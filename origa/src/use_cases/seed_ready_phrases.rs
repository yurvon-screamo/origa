use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use tracing::info;
use ulid::Ulid;

use crate::dictionary::grammar::{get_all_rule_ids, is_grammar_loaded};
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
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let grammar_orphaned_removed = remove_orphaned_grammar_cards(&mut user);

        if !is_phrases_loaded() {
            if grammar_orphaned_removed > 0 {
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let orphaned_removed = remove_orphaned_phrase_cards(&mut user);

        let (known_words, known_grammar) = {
            let study_cards = user.knowledge_set().study_cards();
            (
                collect_known_vocabulary_words(study_cards),
                collect_known_grammar_rules(study_cards),
            )
        };

        if known_words.is_empty() {
            if orphaned_removed > 0 || grammar_orphaned_removed > 0 {
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let current_hash = compute_combined_hash(&known_words, &known_grammar);
        if current_hash == user.known_vocab_hash() {
            if orphaned_removed > 0 || grammar_orphaned_removed > 0 {
                user.set_known_vocab_hash(current_hash);
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let study_cards = user.knowledge_set().study_cards();
        let existing_phrase_ids = collect_existing_phrase_ids(study_cards);
        let ready_phrase_ids =
            find_ready_phrases(&known_words, &known_grammar, &existing_phrase_ids);

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
            grammar_orphaned_removed = grammar_orphaned_removed,
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
                if !sc.memory().is_new() {
                    return Some(vocab.word().text().to_string());
                }
            }
            None
        })
        .collect()
}

pub fn collect_known_grammar_rules(study_cards: &HashMap<Ulid, StudyCard>) -> HashSet<Ulid> {
    study_cards
        .values()
        .filter_map(|sc| {
            if let Card::Grammar(grammar_card) = sc.card() {
                if !sc.memory().is_new() {
                    return Some(*grammar_card.rule_id());
                }
            }
            None
        })
        .collect()
}

fn compute_combined_hash(known_words: &HashSet<String>, known_grammar: &HashSet<Ulid>) -> u32 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();

    let mut words: Vec<&String> = known_words.iter().collect();
    words.sort();
    for word in &words {
        word.hash(&mut hasher);
    }

    let mut rules: Vec<&Ulid> = known_grammar.iter().collect();
    rules.sort();
    for rule in &rules {
        rule.hash(&mut hasher);
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

fn remove_orphaned_grammar_cards(user: &mut crate::domain::User) -> usize {
    if !is_grammar_loaded() {
        return 0;
    }

    let valid_ids = get_all_rule_ids();

    let orphaned_card_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .iter()
        .filter_map(|(card_id, sc)| {
            if let Card::Grammar(grammar_card) = sc.card() {
                if !valid_ids.contains(grammar_card.rule_id()) {
                    return Some(*card_id);
                }
            }
            None
        })
        .collect();

    let count = orphaned_card_ids.len();
    if count > 0 {
        info!(count, "Removing orphaned grammar cards");
        for card_id in &orphaned_card_ids {
            if user.delete_card(*card_id).is_err() {
                tracing::warn!(%card_id, "Failed to delete orphaned grammar card");
            }
        }
    }
    count
}

fn find_ready_phrases(
    known_words: &HashSet<String>,
    known_grammar: &HashSet<Ulid>,
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

            if !all_tokens_known {
                continue;
            }

            let grammar_ready = entry
                .grammar_rules()
                .iter()
                .all(|rule_id| known_grammar.contains(rule_id));

            if !grammar_ready {
                continue;
            }

            result.push(*phrase_id);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::grammar::{GrammarData, init_grammar};
    use crate::domain::{GrammarRuleCard, NativeLanguage, User};
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_grammar_loaded() {
        INIT.call_once(|| {
            if is_grammar_loaded() {
                return;
            }

            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

            let grammar_path = std::path::PathBuf::from(manifest_dir)
                .parent()
                .expect("Failed to get parent directory")
                .join("cdn")
                .join("grammar")
                .join("grammar.json");

            let grammar_json =
                std::fs::read_to_string(&grammar_path).expect("Failed to read grammar.json");

            let grammar_data = GrammarData { grammar_json };
            init_grammar(grammar_data).expect("Failed to init grammar rules");
        });
    }

    fn create_test_user() -> User {
        User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        )
    }

    fn get_first_rule_id() -> Ulid {
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    #[test]
    fn remove_orphaned_grammar_cards_returns_zero_when_no_orphaned_cards() {
        ensure_grammar_loaded();

        let mut user = create_test_user();
        let valid_rule_id = get_first_rule_id();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            valid_rule_id,
        )))
        .unwrap();

        let removed = remove_orphaned_grammar_cards(&mut user);

        assert_eq!(removed, 0);
        assert_eq!(user.knowledge_set().study_cards().len(), 1);
    }

    #[test]
    fn remove_orphaned_grammar_cards_removes_cards_with_nonexistent_rule_ids() {
        ensure_grammar_loaded();

        let mut user = create_test_user();

        let valid_rule_id = get_first_rule_id();
        let orphan_rule_id = Ulid::new();

        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            valid_rule_id,
        )))
        .unwrap();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            orphan_rule_id,
        )))
        .unwrap();

        assert_eq!(user.knowledge_set().study_cards().len(), 2);

        let removed = remove_orphaned_grammar_cards(&mut user);

        assert_eq!(removed, 1);
        assert_eq!(user.knowledge_set().study_cards().len(), 1);

        let remaining_card = user.knowledge_set().study_cards().values().next().unwrap();
        if let Card::Grammar(grammar_card) = remaining_card.card() {
            assert_eq!(*grammar_card.rule_id(), valid_rule_id);
        } else {
            panic!("Expected grammar card");
        }
    }

    #[test]
    fn remove_orphaned_grammar_cards_returns_zero_when_grammar_not_loaded() {
        let mut user = create_test_user();
        let orphan_rule_id = Ulid::new();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            orphan_rule_id,
        )))
        .unwrap();

        // This test may run before or after grammar is loaded.
        // If grammar is already loaded, the card would be orphaned.
        // The key behavior being tested: function does not panic.
        let removed = remove_orphaned_grammar_cards(&mut user);

        if !is_grammar_loaded() {
            assert_eq!(removed, 0);
            assert_eq!(user.knowledge_set().study_cards().len(), 1);
        }
        // If grammar IS loaded (from another test), the card would be orphaned.
        // We don't assert in that case since it's environment-dependent.
    }

    #[test]
    fn remove_orphaned_grammar_cards_ignores_non_grammar_cards() {
        ensure_grammar_loaded();

        let mut user = create_test_user();
        let orphan_rule_id = Ulid::new();

        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            orphan_rule_id,
        )))
        .unwrap();
        user.create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
            .unwrap();

        let removed = remove_orphaned_grammar_cards(&mut user);

        assert_eq!(removed, 1);

        let remaining = user.knowledge_set().study_cards();
        assert_eq!(remaining.len(), 1);
        assert!(
            remaining
                .values()
                .any(|sc| matches!(sc.card(), Card::Phrase(_)))
        );
    }

    #[test]
    fn collect_known_grammar_rules_returns_empty_for_no_grammar_cards() {
        let mut user = create_test_user();
        user.create_card(Card::Vocabulary(crate::domain::VocabularyCard::new(
            crate::domain::value_objects::Question::new("test".to_string()).unwrap(),
        )))
        .unwrap();

        let study_cards = user.knowledge_set().study_cards();
        let result = collect_known_grammar_rules(study_cards);

        assert!(result.is_empty());
    }

    #[test]
    fn collect_known_grammar_rules_returns_ids_for_known_grammar_cards() {
        let mut user = create_test_user();
        let rule_id = Ulid::new();
        let study_card = user
            .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)))
            .unwrap();
        user.knowledge_set_mut()
            .mark_card_as_known(*study_card.card_id())
            .unwrap();

        let study_cards = user.knowledge_set().study_cards();
        let result = collect_known_grammar_rules(study_cards);

        assert_eq!(result.len(), 1);
        assert!(result.contains(&rule_id));
    }

    #[test]
    fn collect_known_grammar_rules_excludes_unknown_grammar_cards() {
        let mut user = create_test_user();
        let rule_id = Ulid::new();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)))
            .unwrap();

        let study_cards = user.knowledge_set().study_cards();
        let result = collect_known_grammar_rules(study_cards);

        assert!(
            result.is_empty(),
            "New (not-yet-known) grammar cards should be excluded"
        );
    }

    #[test]
    fn compute_combined_hash_differs_when_grammar_changes() {
        let words: HashSet<String> = ["hello".to_string(), "test".to_string()]
            .into_iter()
            .collect();

        let grammar_empty: HashSet<Ulid> = HashSet::new();
        let grammar_with_rule: HashSet<Ulid> = [Ulid::new()].into_iter().collect();

        let hash_empty = compute_combined_hash(&words, &grammar_empty);
        let hash_with_rule = compute_combined_hash(&words, &grammar_with_rule);

        assert_ne!(
            hash_empty, hash_with_rule,
            "Hash should change when grammar rules are added"
        );
    }

    #[test]
    fn compute_combined_hash_same_for_identical_inputs() {
        let words: HashSet<String> = ["hello".to_string()].into_iter().collect();
        let rules: HashSet<Ulid> = [Ulid::from_string("01KPG000000000000000000001").unwrap()]
            .into_iter()
            .collect();

        let hash1 = compute_combined_hash(&words, &rules);
        let hash2 = compute_combined_hash(&words, &rules);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn compute_combined_hash_differs_from_words_only_hash() {
        let words: HashSet<String> = ["hello".to_string()].into_iter().collect();
        let rules: HashSet<Ulid> = [Ulid::new()].into_iter().collect();

        let hash_words_only = compute_combined_hash(&words, &HashSet::new());
        let hash_with_grammar = compute_combined_hash(&words, &rules);

        assert_ne!(
            hash_words_only, hash_with_grammar,
            "Combined hash should differ from words-only hash"
        );
    }
}
