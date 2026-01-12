use crate::domain::{
    ReviewLog,
    knowledge::{GrammarRuleCard, KanjiCard, VocabularyCard},
    memory::{MemoryHistory, MemoryState},
    value_objects::{Answer, Question},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudyCard {
    card_id: Ulid,
    card: Card,
    memory_history: MemoryHistory,
}

impl StudyCard {
    pub fn new(card: Card) -> Self {
        Self {
            card_id: Ulid::new(),
            card,
            memory_history: MemoryHistory::default(),
        }
    }

    pub fn card_id(&self) -> &Ulid {
        &self.card_id
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn memory(&self) -> &MemoryHistory {
        &self.memory_history
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: ReviewLog) {
        self.memory_history.add_review(memory_state, review);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Card {
    Vocabulary(VocabularyCard),
    Kanji(KanjiCard),
    Grammar(GrammarRuleCard),
}

impl Card {
    pub fn question(&self) -> &Question {
        match self {
            Card::Vocabulary(card) => card.word(),
            Card::Kanji(card) => card.kanji(),
            Card::Grammar(card) => card.title(),
        }
    }

    pub fn answer(&self) -> &Answer {
        match self {
            Card::Vocabulary(card) => card.meaning(),
            Card::Kanji(card) => card.description(),
            Card::Grammar(card) => card.description(),
        }
    }
}
