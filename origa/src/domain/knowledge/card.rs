use crate::domain::{
    NativeLanguage, OrigaError, Rating, ReviewLog, get_rule_by_id,
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
    #[serde(default)]
    is_favorite: bool,
    #[serde(default)]
    perfect_streak_since_known: u8,
}

impl StudyCard {
    pub fn new(card: Card) -> Self {
        Self {
            card_id: Ulid::new(),
            card,
            memory_history: MemoryHistory::default(),
            is_favorite: false,
            perfect_streak_since_known: 0,
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

    pub fn is_favorite(&self) -> bool {
        self.is_favorite
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: ReviewLog) {
        self.memory_history.add_review(memory_state, review);
    }

    pub(crate) fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        if !self.is_favorite {
            self.perfect_streak_since_known = 0;
        }
    }

    pub(crate) fn handle_favorite_rating(&mut self, rating: Rating) {
        if !self.is_favorite || !self.memory_history.is_known_card() {
            return;
        }

        match rating {
            Rating::Easy => {
                self.perfect_streak_since_known += 1;
                if self.perfect_streak_since_known >= 5 {
                    self.is_favorite = false;
                    self.perfect_streak_since_known = 0;
                }
            }
            _ => {
                self.perfect_streak_since_known = 0;
            }
        }
    }

    pub fn shuffle_card(
        &self,
        lang: &NativeLanguage,
        known_grammars: &[GrammarRuleCard],
    ) -> Result<Card, OrigaError> {
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

    pub fn content_key(&self) -> String {
        match self {
            Card::Vocabulary(card) => card.word().text().to_string(),
            Card::Kanji(card) => card.kanji().text().to_string(),
            Card::Grammar(card) => card.rule_id().to_string(),
        }
    }
}
