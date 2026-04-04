use crate::domain::knowledge::KnowledgeSet;
use crate::domain::knowledge::VocabularyCard;
use crate::domain::memory::{Difficulty, MemoryState, Rating, ReviewLog, Stability};
use crate::domain::value_objects::Question;
use crate::domain::{Card, GrammarRuleCard, StudyCard};
use chrono::{Duration, Utc};
use ulid::Ulid;

mod card_views;
mod filtering;
mod quiz;
mod types;
mod yesno;

pub(super) use super::generation;

fn create_vocab_card(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ))
}

fn create_grammar_card(rule_id: Ulid) -> Card {
    Card::Grammar(GrammarRuleCard::new(rule_id).unwrap())
}

pub(crate) fn create_study_card_with_memory(
    word: &str,
    stability: f64,
    difficulty: f64,
    interval_days: i64,
    rating: Rating,
) -> StudyCard {
    let card = Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ));
    let mut study_card = StudyCard::new(card);
    let memory = MemoryState::new(
        Stability::new(stability).unwrap(),
        Difficulty::new(difficulty).unwrap(),
        Utc::now(),
    );
    study_card.add_review(
        memory,
        ReviewLog::new(rating, Duration::days(interval_days)),
    );
    study_card
}

pub(crate) fn create_knowledge_set_with_vocab(words: &[&str]) -> KnowledgeSet {
    let mut ks = KnowledgeSet::new();
    for word in words {
        ks.create_card(Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        )))
        .unwrap();
    }
    ks
}
