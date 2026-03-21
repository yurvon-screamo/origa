use crate::domain::{
    OrigaError, Rating, ReviewLog,
    knowledge::{GrammarRuleCard, KanjiCard, RadicalCard, VocabularyCard},
    memory::{MemoryHistory, MemoryState},
    value_objects::{Answer, NativeLanguage, Question},
};
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

    pub fn is_new(&self) -> bool {
        self.memory_history.is_new()
    }

    pub fn perfect_streak_since_known(&self) -> u8 {
        self.perfect_streak_since_known
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
            Rating::Good => {}
            Rating::Hard | Rating::Again => {
                self.perfect_streak_since_known = 0;
            }
        }
    }

    pub fn merge(&mut self, other: &StudyCard) {
        self.memory_history.merge(&other.memory_history);
        self.is_favorite = self.is_favorite || other.is_favorite;
        self.perfect_streak_since_known = self
            .perfect_streak_since_known
            .max(other.perfect_streak_since_known);
    }

    pub fn shuffle_card(&self) -> Card {
        self.card.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Card {
    Vocabulary(VocabularyCard),
    Kanji(KanjiCard),
    Grammar(GrammarRuleCard),
    Radical(RadicalCard),
}

impl Card {
    pub fn question(&self, lang: &NativeLanguage) -> Result<Question, OrigaError> {
        match self {
            Card::Vocabulary(card) => Ok(card.word().clone()),
            Card::Kanji(card) => Ok(card.kanji().clone()),
            Card::Grammar(card) => card.title(lang),
            Card::Radical(card) => Ok(card.question().clone()),
        }
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        match self {
            Card::Vocabulary(card) => card.answer(lang),
            Card::Kanji(card) => card.description(),
            Card::Grammar(card) => card.description(lang),
            Card::Radical(card) => {
                let info = card.radical_info()?;
                Answer::new(info.radical().to_string()).map_err(|e| OrigaError::InvalidAnswer {
                    reason: e.to_string(),
                })
            }
        }
    }

    pub fn content_key(&self) -> String {
        match self {
            Card::Vocabulary(card) => card.word().text().to_string(),
            Card::Kanji(card) => card.kanji().text().to_string(),
            Card::Grammar(card) => card.rule_id().to_string(),
            Card::Radical(card) => card.radical_char().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    Vocabulary,
    Kanji,
    Grammar,
    Radical,
}

impl From<&Card> for CardType {
    fn from(card: &Card) -> Self {
        match card {
            Card::Vocabulary(_) => CardType::Vocabulary,
            Card::Kanji(_) => CardType::Kanji,
            Card::Grammar(_) => CardType::Grammar,
            Card::Radical(_) => CardType::Radical,
        }
    }
}
