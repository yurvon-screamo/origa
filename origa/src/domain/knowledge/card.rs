use crate::domain::{
    OrigaError, Rating, ReviewLog,
    knowledge::{GrammarRuleCard, KanjiCard, VocabularyCard},
    memory::{MemoryHistory, MemoryState},
    value_objects::{Answer, NativeLanguage, Question},
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
}

impl Card {
    pub fn question(&self, lang: &NativeLanguage) -> Result<Question, OrigaError> {
        match self {
            Card::Vocabulary(card) => Ok(card.word().clone()),
            Card::Kanji(card) => Ok(card.kanji().clone()),
            Card::Grammar(card) => card.title(lang),
        }
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        match self {
            Card::Vocabulary(card) => card.answer(lang),
            Card::Kanji(card) => card.description(),
            Card::Grammar(card) => card.description(lang),
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
pub struct GrammarInfo {
    title: String,
    description: String,
}

impl GrammarInfo {
    pub fn new(title: String, description: String) -> Self {
        Self { title, description }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LessonCardView {
    Normal(Card),
    Quiz(QuizCard),
    Reversed(Card),
    GrammarMutated {
        card: Card,
        grammar_info: GrammarInfo,
    },
    Writing(Card),
}

impl LessonCardView {
    pub fn card(&self) -> &Card {
        match self {
            LessonCardView::Normal(card)
            | LessonCardView::Reversed(card)
            | LessonCardView::GrammarMutated { card, .. }
            | LessonCardView::Writing(card) => card,
            LessonCardView::Quiz(quiz) => quiz.card(),
        }
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        match self {
            LessonCardView::GrammarMutated { grammar_info, .. } => Some(grammar_info),
            _ => None,
        }
    }

    pub fn generate_quiz(
        original_card: Card,
        same_type_cards: &[Card],
        lang: &NativeLanguage,
    ) -> Result<Self, OrigaError> {
        match &original_card {
            Card::Grammar(_) => return Ok(LessonCardView::Normal(original_card)),
            Card::Vocabulary(_) | Card::Kanji(_) => {}
        }

        let correct_answer = original_card.answer(lang)?;

        let mut distractors: Vec<String> = same_type_cards
            .iter()
            .filter_map(|c| {
                c.answer(lang)
                    .ok()
                    .filter(|a| a.text() != correct_answer.text())
            })
            .map(|a| a.text().to_string())
            .collect();

        distractors.shuffle(&mut rand::rng());
        let needed_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
        let selected_distractors: Vec<String> =
            distractors.into_iter().take(needed_distractors).collect();

        if selected_distractors.len() < needed_distractors {
            return Ok(LessonCardView::Normal(original_card));
        }

        let mut options: Vec<QuizOption> = selected_distractors
            .into_iter()
            .map(|text| QuizOption::new(text, false))
            .collect();

        options.push(QuizOption::new(correct_answer.text().to_string(), true));
        options.shuffle(&mut rand::rng());

        let quiz = QuizCard::new(original_card, options);
        Ok(LessonCardView::Quiz(quiz))
    }
}
