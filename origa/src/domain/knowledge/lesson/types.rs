use std::collections::HashMap;

use rand::seq::SliceRandom;
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
            | LessonCardView::KanjiReadingQuiz(_) => None,
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
        core.shuffle(&mut rand::rng());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Card;
    use crate::domain::knowledge::{PhraseCard, VocabularyCard};
    use crate::domain::value_objects::Question;

    fn make_vocabulary_lesson_card(id: Ulid) -> (Ulid, LessonCard) {
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new("test".to_string()).unwrap(),
        ));
        (id, LessonCard::new(LessonCardView::Normal(card), false))
    }

    fn make_phrase_lesson_card(id: Ulid) -> (Ulid, LessonCard) {
        let card = Card::Phrase(PhraseCard::new_test_with_id(id));
        (id, LessonCard::new(LessonCardView::Normal(card), false))
    }

    #[test]
    fn reorder_core_first_phrases_last_preserves_all_cards() {
        let vocab_1 = Ulid::new();
        let vocab_2 = Ulid::new();
        let phrase_1 = Ulid::new();
        let phrase_2 = Ulid::new();

        let input = vec![
            make_phrase_lesson_card(phrase_1),
            make_vocabulary_lesson_card(vocab_1),
            make_phrase_lesson_card(phrase_2),
            make_vocabulary_lesson_card(vocab_2),
        ];

        let result = LessonData::reorder_core_first_phrases_last(input);
        let result_ids: Vec<Ulid> = result.card_ids();

        assert_eq!(result.total_count(), 4);
        assert_eq!(result.core_count, 2);
        assert_eq!(result.phrase_count(), 2);

        assert!(result_ids.contains(&vocab_1));
        assert!(result_ids.contains(&vocab_2));
        assert!(result_ids.contains(&phrase_1));
        assert!(result_ids.contains(&phrase_2));

        let core_ids: Vec<Ulid> = result_ids[..result.core_count].to_vec();
        let phrase_ids: Vec<Ulid> = result_ids[result.core_count..].to_vec();

        assert!(
            core_ids.iter().all(
                |id| !matches!(result.get(id), Some(lc) if lc.card_type() == CardType::Phrase)
            )
        );
        assert!(
            phrase_ids
                .iter()
                .all(|id| matches!(result.get(id), Some(lc) if lc.card_type() == CardType::Phrase))
        );
    }

    #[test]
    fn reorder_core_first_phrases_last_handles_empty() {
        let result = LessonData::reorder_core_first_phrases_last(vec![]);

        assert!(result.is_empty());
        assert_eq!(result.core_count, 0);
        assert_eq!(result.total_count(), 0);
    }

    #[test]
    fn reorder_core_first_phrases_last_only_core() {
        let vocab_1 = Ulid::new();
        let vocab_2 = Ulid::new();
        let vocab_3 = Ulid::new();

        let input = vec![
            make_vocabulary_lesson_card(vocab_1),
            make_vocabulary_lesson_card(vocab_2),
            make_vocabulary_lesson_card(vocab_3),
        ];

        let result = LessonData::reorder_core_first_phrases_last(input);
        let result_ids: Vec<Ulid> = result.card_ids();

        assert_eq!(result.total_count(), 3);
        assert_eq!(result.core_count, 3);
        assert_eq!(result.phrase_count(), 0);

        assert!(result_ids.contains(&vocab_1));
        assert!(result_ids.contains(&vocab_2));
        assert!(result_ids.contains(&vocab_3));
    }
}
