use crate::domain::{
    KeikakuError, NativeLanguage, ReviewLog, get_rule_by_id,
    knowledge::{GrammarRuleCard, KanjiCard, VocabularyCard},
    memory::{MemoryHistory, MemoryState},
    value_objects::{Answer, Question},
};
use rand::seq::SliceRandom;
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

    pub fn shuffle_card(
        &self,
        lang: &NativeLanguage,
        known_grammars: &[GrammarRuleCard],
    ) -> Result<Card, KeikakuError> {
        let mut content = self.card().clone();

        if !self.memory().is_known_card() && !self.memory().is_in_progress() {
            return Ok(content);
        }

        content = match &content {
            Card::Vocabulary(vocab) => match rand::random_bool(0.5) {
                false => {
                    let reverted = vocab.revert()?;
                    Card::Vocabulary(reverted)
                }
                true => {
                    let word_part = vocab.part_of_speech()?;

                    let mut rules: Vec<_> = known_grammars
                        .iter()
                        .filter(|g| g.apply_to().contains(&word_part))
                        .collect();

                    rules.shuffle(&mut rand::rng());

                    if let Some(rule) = rules.first()
                        && let Some(rule) = get_rule_by_id(rule.rule_id())
                    {
                        let vocab_with_rule = vocab.with_grammar_rule(rule, lang)?;
                        Card::Vocabulary(vocab_with_rule)
                    } else {
                        content
                    }
                }
            },
            _ => content,
        };

        Ok(content)
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
