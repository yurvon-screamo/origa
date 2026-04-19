use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::NativeLanguage;
use crate::domain::{Card, CardType, GrammarRuleCard, MemoryHistory};
use rand::Rng;

use super::types::LessonCardView;

mod generation;
#[cfg(test)]
mod tests;
mod transforms;

const QUIZ_OPTIONS_COUNT: usize = 4;

const PROB_NORMAL_VIEW: f32 = 0.15;
const PROB_QUIZ_VIEW: f32 = 0.30;
const PROB_YESNO_VIEW: f32 = 0.50;
const PROB_REVERSED_VIEW: f32 = 0.75;

const PROB_KANJI_NORMAL: f32 = 0.25;
const PROB_KANJI_QUIZ: f32 = 0.50;
const PROB_KANJI_YESNO: f32 = 0.70;

const PROB_NEW_KANJI_NORMAL: f32 = 0.33;
const PROB_NEW_KANJI_QUIZ: f32 = 0.66;
const PROB_NEW_VOCAB_NORMAL: f32 = 0.50;

const PROB_NEW_PHRASE_NORMAL: f32 = 0.50;
const PROB_REVIEW_PHRASE_NORMAL: f32 = 0.15;

const EASY_REVIEWS_FOR_REVERSED: usize = 2;
const GOOD_REVIEWS_FOR_REVERSED: usize = 4;
const DEFAULT_LANG: NativeLanguage = NativeLanguage::Russian;

pub struct LessonViewGenerator<'a> {
    knowledge_set: &'a KnowledgeSet,
}

impl<'a> LessonViewGenerator<'a> {
    pub fn new(knowledge_set: &'a KnowledgeSet) -> Self {
        Self { knowledge_set }
    }

    pub fn apply_view(
        &self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        rng: &mut impl Rng,
    ) -> LessonCardView {
        let card = study_card.card();
        let card_type = CardType::from(card);

        let cards_by_type = self.knowledge_set.build_cards_by_type();
        let same_type_cards = cards_by_type
            .get(&card_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let known_grammars: Vec<_> = self
            .knowledge_set
            .study_cards()
            .values()
            .filter_map(|x| match x.card() {
                Card::Grammar(grammar_rule_card) => Some(grammar_rule_card.clone()),
                _ => None,
            })
            .collect();

        match card_type {
            CardType::Grammar => LessonCardView::Normal(card.clone()),
            CardType::Kanji if is_new => self.select_new_kanji_view(card, same_type_cards, rng),
            CardType::Kanji => {
                self.select_review_kanji_view(card, same_type_cards, study_card.memory(), rng)
            },
            CardType::Vocabulary if is_new => {
                self.select_new_vocab_view(card, same_type_cards, rng)
            },
            CardType::Vocabulary => self.select_review_vocab_view(
                card,
                same_type_cards,
                &known_grammars,
                study_card.memory(),
                rng,
            ),
            CardType::Phrase => self.select_phrase_view(card, same_type_cards, is_new, rng),
        }
    }

    fn select_new_kanji_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_NEW_KANJI_NORMAL {
            LessonCardView::Normal(card.clone())
        } else if rand_val < PROB_NEW_KANJI_QUIZ {
            generation::generate_quiz(card.clone(), same_type_cards, &DEFAULT_LANG)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else {
            LessonCardView::Writing(card.clone())
        }
    }

    fn select_review_kanji_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        memory: &MemoryHistory,
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_KANJI_NORMAL {
            LessonCardView::Normal(card.clone())
        } else if rand_val < PROB_KANJI_QUIZ {
            generation::generate_quiz(card.clone(), same_type_cards, &DEFAULT_LANG)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if !memory.is_high_difficulty() && rand_val < PROB_KANJI_YESNO {
            generation::generate_yesno(card.clone(), same_type_cards, &DEFAULT_LANG, rng)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else {
            LessonCardView::Writing(card.clone())
        }
    }

    fn select_new_vocab_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_NEW_VOCAB_NORMAL {
            LessonCardView::Normal(card.clone())
        } else {
            generation::generate_quiz(card.clone(), same_type_cards, &DEFAULT_LANG)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        }
    }

    fn select_review_vocab_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        known_grammars: &[GrammarRuleCard],
        memory: &MemoryHistory,
        rng: &mut R,
    ) -> LessonCardView {
        let is_high_difficulty = memory.is_high_difficulty();
        let eligible_for_advanced = memory.is_known_card() || memory.is_in_progress();
        let eligible_for_reversed = eligible_for_advanced
            || memory.easy_review_count() > EASY_REVIEWS_FOR_REVERSED
            || memory.good_review_count() >= GOOD_REVIEWS_FOR_REVERSED;
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_NORMAL_VIEW {
            LessonCardView::Normal(card.clone())
        } else if rand_val < PROB_QUIZ_VIEW {
            generation::generate_quiz(card.clone(), same_type_cards, &DEFAULT_LANG)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if !is_high_difficulty && rand_val < PROB_YESNO_VIEW {
            generation::generate_yesno(card.clone(), same_type_cards, &DEFAULT_LANG, rng)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if eligible_for_reversed && rand_val < PROB_REVERSED_VIEW {
            transforms::apply_reversed(card)
        } else if eligible_for_advanced {
            transforms::apply_grammar_mutated(card, known_grammars, rng)
        } else {
            LessonCardView::Normal(card.clone())
        }
    }

    fn select_phrase_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        is_new: bool,
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        let normal_threshold = if is_new {
            PROB_NEW_PHRASE_NORMAL
        } else {
            PROB_REVIEW_PHRASE_NORMAL
        };

        if rand_val < normal_threshold {
            return LessonCardView::Normal(card.clone());
        }

        generation::generate_phrase_quiz(card.clone(), same_type_cards, &DEFAULT_LANG)
            .unwrap_or_else(|| match card {
                Card::Phrase(phrase_card) => {
                    let audio_file = phrase_card.audio_file().unwrap_or_default();
                    LessonCardView::PhraseListen {
                        card: card.clone(),
                        audio_file,
                        options: vec![],
                    }
                },
                Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => {
                    LessonCardView::Normal(card.clone())
                },
            })
    }
}
