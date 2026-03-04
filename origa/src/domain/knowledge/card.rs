use crate::domain::{
    Rating, ReviewLog,
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

    pub fn shuffle_card(&self) -> Card {
        self.card.clone()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    Vocabulary,
    Kanji,
    Grammar,
}

impl From<&Card> for CardType {
    fn from(card: &Card) -> Self {
        match card {
            Card::Vocabulary(_) => CardType::Vocabulary,
            Card::Kanji(_) => CardType::Kanji,
            Card::Grammar(_) => CardType::Grammar,
        }
    }
}

const QUIZ_OPTIONS_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizOption {
    text: String,
    is_correct: bool,
}

impl QuizOption {
    pub fn new(text: String, is_correct: bool) -> Self {
        Self { text, is_correct }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizCard {
    card: Card,
    options: Vec<QuizOption>,
}

impl QuizCard {
    pub fn new(card: Card, options: Vec<QuizOption>) -> Self {
        Self { card, options }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn options(&self) -> &[QuizOption] {
        &self.options
    }

    pub fn check_answer(&self, index: usize) -> bool {
        self.options
            .get(index)
            .map(|o| o.is_correct())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LessonCardView {
    Normal(Card),
    Quiz(QuizCard),
    Reversed(Card),
    GrammarMutated(Card),
}

impl LessonCardView {
    pub fn card(&self) -> &Card {
        match self {
            LessonCardView::Normal(card)
            | LessonCardView::Reversed(card)
            | LessonCardView::GrammarMutated(card) => card,
            LessonCardView::Quiz(quiz) => quiz.card(),
        }
    }

    pub fn generate_quiz(original_card: Card, same_type_cards: &[Card]) -> Self {
        match &original_card {
            Card::Grammar(_) => return LessonCardView::Normal(original_card),
            Card::Vocabulary(_) | Card::Kanji(_) => {}
        }

        let correct_answer = original_card.answer().text().to_string();

        let mut distractors: Vec<String> = same_type_cards
            .iter()
            .filter(|c| c.answer().text() != correct_answer)
            .map(|c| c.answer().text().to_string())
            .collect();

        distractors.shuffle(&mut rand::rng());
        let needed_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
        let mut selected_distractors: Vec<String> =
            distractors.into_iter().take(needed_distractors).collect();

        if selected_distractors.len() < needed_distractors {
            let correct_len = correct_answer.len();
            let dummy = "—".repeat(correct_len.max(3));
            while selected_distractors.len() < needed_distractors {
                selected_distractors.push(dummy.clone());
            }
        }

        let mut options: Vec<QuizOption> = selected_distractors
            .into_iter()
            .map(|text| QuizOption::new(text, false))
            .collect();

        options.push(QuizOption::new(correct_answer, true));
        options.shuffle(&mut rand::rng());

        let quiz = QuizCard::new(original_card, options);
        LessonCardView::Quiz(quiz)
    }
}
