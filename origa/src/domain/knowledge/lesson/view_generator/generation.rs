use crate::domain::value_objects::NativeLanguage;
use crate::domain::{Card, OrigaError};
use rand::{Rng, prelude::IndexedRandom, seq::SliceRandom};

use super::super::types::{LessonCardView, QuizCard, QuizOption, YesNoCard};
use super::QUIZ_OPTIONS_COUNT;

pub(crate) fn generate_quiz(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
) -> Result<LessonCardView, OrigaError> {
    match &original_card {
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => {},
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

pub(crate) fn generate_yesno(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
    rng: &mut impl Rng,
) -> Result<LessonCardView, OrigaError> {
    match &original_card {
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => {},
    }

    let question = original_card.question(lang)?;
    let correct_answer = original_card.answer(lang)?;

    let is_correct = rng.random_bool(0.5);

    let statement_answer = if is_correct {
        correct_answer.text().to_string()
    } else {
        let distractors: Vec<_> = same_type_cards
            .iter()
            .filter_map(|c| c.answer(lang).ok())
            .filter(|a| a.text() != correct_answer.text())
            .map(|a| a.text().to_string())
            .collect();

        if distractors.is_empty() {
            return Ok(LessonCardView::Normal(original_card));
        }

        distractors
            .choose(rng)
            .expect("distractors guaranteed non-empty after is_empty check")
            .clone()
    };

    let statement_text = format!("{} \n {}", question.text(), statement_answer);

    Ok(LessonCardView::YesNo(YesNoCard::new(
        original_card,
        statement_text,
        is_correct,
    )))
}
