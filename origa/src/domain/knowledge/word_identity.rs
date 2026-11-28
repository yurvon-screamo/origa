//! Read-only scan of duplicate vocabulary cards by their stored content key.
//!
//! Word creation resolves the SudachiDict lemma once in `from_text`. Re-tokenizing
//! an already stored word is NOT equivalent (normalized forms are not
//! idempotent), so this module never rewrites card content — it only reports
//! groups that share the exact key the creation paths use.

use std::collections::HashMap;

use ulid::Ulid;

use super::{CardType, KnowledgeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordIdentityScanGroup {
    pub card_type: CardType,
    pub word: String,
    pub card_ids: Vec<Ulid>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordIdentityScanReport {
    pub groups: Vec<WordIdentityScanGroup>,
}

impl WordIdentityScanReport {
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    pub fn duplicate_cards(&self) -> usize {
        self.groups
            .iter()
            .map(|group| group.card_ids.len() - 1)
            .sum()
    }
}

/// Groups vocabulary cards by the exact `(CardType, content_key)` identity
/// used by `validate_unique_card`. Other card types are out of scope: their
/// duplicates are handled by their own pipelines. No card is mutated.
pub fn scan_duplicate_cards(set: &KnowledgeSet) -> WordIdentityScanReport {
    let mut groups: HashMap<String, Vec<Ulid>> = HashMap::new();
    for (id, study) in set.study_cards() {
        if let super::Card::Vocabulary(_) = study.card() {
            groups
                .entry(study.card().content_key())
                .or_default()
                .push(*id);
        }
    }

    let mut report = WordIdentityScanReport::default();
    for (word, mut ids) in groups {
        if ids.len() < 2 {
            continue;
        }
        ids.sort_unstable();
        report.groups.push(WordIdentityScanGroup {
            card_type: CardType::Vocabulary,
            word,
            card_ids: ids,
        });
    }
    report
        .groups
        .sort_by(|left, right| left.word.cmp(&right.word));
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::domain::{Card, StudyCard, VocabularyCard};

    fn card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    #[test]
    fn scan_reports_exact_duplicates_without_mutating_deck() {
        // Arrange
        let mut set = KnowledgeSet::new();
        set.create_card(card("食べる")).unwrap();
        set.study_cards
            .insert(Ulid::new(), StudyCard::new(card("食べる")));
        let before = set.study_cards().len();

        // Act
        let report = scan_duplicate_cards(&set);

        // Assert
        assert_eq!(report.group_count(), 1);
        assert_eq!(report.duplicate_cards(), 1);
        assert_eq!(report.groups[0].word, "食べる");
        assert_eq!(set.study_cards().len(), before);
    }

    #[test]
    fn scan_keeps_independent_articles_in_separate_groups() {
        // Arrange
        let mut set = KnowledgeSet::new();
        set.create_card(card("止める")).unwrap();
        set.create_card(card("止む")).unwrap();

        // Act
        let report = scan_duplicate_cards(&set);

        // Assert
        assert_eq!(report.group_count(), 0);
        assert_eq!(report.duplicate_cards(), 0);
    }

    #[test]
    fn scan_ignores_example_card_pairs() {
        // Arrange
        let mut set = KnowledgeSet::new();
        set.create_card(Card::Example(crate::domain::ExampleCard::new("食べる", 7)))
            .unwrap();
        set.study_cards.insert(
            Ulid::new(),
            StudyCard::new(Card::Example(crate::domain::ExampleCard::new("食べる", 7))),
        );

        // Act
        let report = scan_duplicate_cards(&set);

        // Assert
        assert_eq!(report.group_count(), 0);
    }
}
