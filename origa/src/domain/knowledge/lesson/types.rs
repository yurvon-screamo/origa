use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::Card;
use crate::domain::knowledge::card::CardType;
use crate::domain::memory::Rating;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizOption {
    text: String,
    is_correct: bool,
    #[serde(default)]
    tag: Option<String>,
}

impl QuizOption {
    pub fn new(text: String, is_correct: bool, tag: Option<String>) -> Self {
        Self {
            text,
            is_correct,
            tag,
        }
    }

    pub fn new_simple(text: String, is_correct: bool) -> Self {
        Self {
            text,
            is_correct,
            tag: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuizMode {
    #[default]
    Single,
    Multi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizCard {
    card: Card,
    options: Vec<QuizOption>,
    #[serde(default)]
    mode: QuizMode,
}

impl QuizCard {
    pub fn new(card: Card, options: Vec<QuizOption>, mode: QuizMode) -> Self {
        Self {
            card,
            options,
            mode,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn options(&self) -> &[QuizOption] {
        &self.options
    }

    pub fn mode(&self) -> QuizMode {
        self.mode
    }

    pub fn check_answer(&self, index: usize) -> bool {
        self.options
            .get(index)
            .map(|o| o.is_correct())
            .unwrap_or(false)
    }

    pub fn check_multi_answers(&self, selected: &[usize]) -> MultiQuizResult {
        let correct_indices: Vec<usize> = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_correct())
            .map(|(i, _)| i)
            .collect();

        let selected_set: std::collections::HashSet<usize> = selected.iter().copied().collect();
        let correct_set: std::collections::HashSet<usize> =
            correct_indices.iter().copied().collect();

        let correct_selections: Vec<usize> = selected
            .iter()
            .filter(|&&i| correct_set.contains(&i))
            .copied()
            .collect();

        let missed: Vec<usize> = correct_indices
            .iter()
            .filter(|&&i| !selected_set.contains(&i))
            .copied()
            .collect();

        let wrong_selections: Vec<usize> = selected
            .iter()
            .filter(|&&i| !correct_set.contains(&i))
            .copied()
            .collect();

        let is_perfect = missed.is_empty() && wrong_selections.is_empty();

        MultiQuizResult {
            correct_selections,
            missed,
            wrong_selections,
            is_perfect,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiQuizResult {
    pub correct_selections: Vec<usize>,
    pub missed: Vec<usize>,
    pub wrong_selections: Vec<usize>,
    pub is_perfect: bool,
}

impl MultiQuizResult {
    pub fn rating(&self) -> Rating {
        if self.is_perfect {
            Rating::Good
        } else {
            Rating::Again
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YesNoCard {
    card: Card,
    statement_text: String,
    is_correct: bool,
}

impl YesNoCard {
    pub fn new(card: Card, statement_text: String, is_correct: bool) -> Self {
        Self {
            card,
            statement_text,
            is_correct,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn statement_text(&self) -> &str {
        &self.statement_text
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn check_answer(&self, user_said_yes: bool) -> bool {
        (self.is_correct && user_said_yes) || (!self.is_correct && !user_said_yes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarInfo {
    rule_id: Option<Ulid>,
    title: String,
    description: String,
}

impl GrammarInfo {
    pub fn new(rule_id: Option<Ulid>, title: String, description: String) -> Self {
        Self {
            rule_id,
            title,
            description,
        }
    }

    pub fn rule_id(&self) -> Option<Ulid> {
        self.rule_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarQuizCard {
    card: Card,
    grammar_info: GrammarInfo,
    word_text: String,
    quiz: QuizCard,
}

impl GrammarQuizCard {
    pub fn new(card: Card, grammar_info: GrammarInfo, word_text: String, quiz: QuizCard) -> Self {
        Self {
            card,
            grammar_info,
            word_text,
            quiz,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn grammar_info(&self) -> &GrammarInfo {
        &self.grammar_info
    }

    pub fn word_text(&self) -> &str {
        &self.word_text
    }

    pub fn quiz(&self) -> &QuizCard {
        &self.quiz
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LessonCardView {
    Normal(Card),
    Quiz(QuizCard),
    YesNo(YesNoCard),
    Reversed(Card),
    GrammarMutated {
        card: Card,
        grammar_info: GrammarInfo,
    },
    Writing(Card),
    PhraseListen {
        card: Card,
        audio_file: String,
        options: Vec<QuizOption>,
    },
    KanjiReadingQuiz(QuizCard),
    KanjiRadicalQuiz(QuizCard),
    GrammarQuiz(GrammarQuizCard),
}

impl LessonCardView {
    pub fn card(&self) -> &Card {
        match self {
            LessonCardView::Normal(card)
            | LessonCardView::Reversed(card)
            | LessonCardView::GrammarMutated { card, .. }
            | LessonCardView::Writing(card)
            | LessonCardView::PhraseListen { card, .. } => card,
            LessonCardView::Quiz(quiz) => quiz.card(),
            LessonCardView::YesNo(yc) => yc.card(),
            LessonCardView::KanjiReadingQuiz(quiz) => quiz.card(),
            LessonCardView::KanjiRadicalQuiz(quiz) => quiz.card(),
            LessonCardView::GrammarQuiz(gq) => gq.card(),
        }
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        match self {
            LessonCardView::GrammarMutated { grammar_info, .. } => Some(grammar_info),
            LessonCardView::GrammarQuiz(gq) => Some(gq.grammar_info()),
            LessonCardView::Normal(_)
            | LessonCardView::Quiz(_)
            | LessonCardView::YesNo(_)
            | LessonCardView::Reversed(_)
            | LessonCardView::Writing(_)
            | LessonCardView::PhraseListen { .. }
            | LessonCardView::KanjiReadingQuiz(_)
            | LessonCardView::KanjiRadicalQuiz(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonCard {
    view: LessonCardView,
    is_short_term: bool,
}

impl LessonCard {
    pub fn new(view: LessonCardView, is_short_term: bool) -> Self {
        Self {
            view,
            is_short_term,
        }
    }

    pub fn view(&self) -> &LessonCardView {
        &self.view
    }

    pub fn into_view(self) -> LessonCardView {
        self.view
    }

    pub fn is_short_term(&self) -> bool {
        self.is_short_term
    }

    pub fn card(&self) -> &Card {
        self.view.card()
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        self.view.grammar_info()
    }

    fn card_type(&self) -> CardType {
        CardType::from(self.view.card())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonData {
    pub cards: Vec<(Ulid, LessonCard)>,
    pub core_count: usize,
}

impl LessonData {
    pub fn card_ids(&self) -> Vec<Ulid> {
        self.cards.iter().map(|(id, _)| *id).collect()
    }

    pub fn cards_map(&self) -> HashMap<Ulid, LessonCard> {
        self.cards
            .iter()
            .map(|(id, card)| (*id, card.clone()))
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.cards.len()
    }

    pub fn phrase_count(&self) -> usize {
        self.cards
            .iter()
            .filter(|(_, lc)| lc.card_type() == CardType::Phrase)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn contains_key(&self, id: &Ulid) -> bool {
        self.cards.iter().any(|(card_id, _)| card_id == id)
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn get(&self, id: &Ulid) -> Option<&LessonCard> {
        self.cards
            .iter()
            .find(|(card_id, _)| card_id == id)
            .map(|(_, card)| card)
    }

    pub fn keys(&self) -> impl Iterator<Item = &Ulid> {
        self.cards.iter().map(|(id, _)| id)
    }

    pub fn values(&self) -> impl Iterator<Item = &LessonCard> {
        self.cards.iter().map(|(_, card)| card)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Ulid, &LessonCard)> {
        self.cards.iter().map(|(id, card)| (id, card))
    }

    pub fn into_cards(self) -> Vec<(Ulid, LessonCard)> {
        self.cards
    }

    pub fn reorder_core_first_phrases_last(cards: Vec<(Ulid, LessonCard)>) -> Self {
        let mut core = Vec::new();
        let mut phrases = Vec::new();

        for entry in cards {
            if entry.1.card_type() == CardType::Phrase {
                phrases.push(entry);
            } else {
                core.push(entry);
            }
        }

        let core_count = core.len();
        core.extend(phrases);

        Self {
            cards: core,
            core_count,
        }
    }
}

impl IntoIterator for LessonData {
    type Item = (Ulid, LessonCard);
    type IntoIter = std::vec::IntoIter<(Ulid, LessonCard)>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}
