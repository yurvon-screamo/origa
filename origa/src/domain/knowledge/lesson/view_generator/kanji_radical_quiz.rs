use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::dictionary::kanji::KanjiInfo;
use crate::domain::knowledge::lesson::types::{LessonCardView, QuizCard, QuizMode, QuizOption};
use crate::domain::{Card, OrigaError};

use super::generation::cached_kanji_info;

pub(crate) fn generate_kanji_radical_quiz(
    original_card: Card,
    kanji_cache: &mut HashMap<String, &'static KanjiInfo>,
) -> Result<LessonCardView, OrigaError> {
    let kanji_card = match &original_card {
        Card::Kanji(kc) => kc,
        Card::Vocabulary(_) | Card::Grammar(_) | Card::Phrase(_) => {
            return Ok(LessonCardView::Normal(original_card));
        },
    };

    let kanji_str = kanji_card.kanji().text();
    let info = match cached_kanji_info(kanji_str, kanji_cache) {
        Ok(info) => info,
        Err(_) => return Ok(LessonCardView::Normal(original_card)),
    };

    let target_chars = info.radicals_chars();
    if target_chars.is_empty() {
        return Ok(LessonCardView::Normal(original_card));
    }

    if !crate::dictionary::radical::is_radicals_loaded() {
        return Ok(LessonCardView::Normal(original_card));
    }

    let radical_list = crate::dictionary::radical::get_radical_list();

    let mut options: Vec<QuizOption> = target_chars
        .iter()
        .filter_map(|&radical_char| {
            let radical_info = crate::dictionary::radical::get_radical_info(radical_char).ok()?;
            Some(QuizOption::new(
                radical_char.to_string(),
                true,
                Some(radical_info.name().to_string()),
            ))
        })
        .collect();

    let correct_count = options.len();
    if correct_count == 0 {
        return Ok(LessonCardView::Normal(original_card));
    }

    let target_set: std::collections::HashSet<char> = target_chars.iter().copied().collect();

    let mut distractors: Vec<QuizOption> = radical_list
        .iter()
        .filter(|r| !target_set.contains(&r.radical()))
        .map(|r| QuizOption::new_simple(r.radical().to_string(), false))
        .collect();

    distractors.shuffle(&mut rand::rng());
    let max_distractors = 8usize.saturating_sub(correct_count);
    distractors.truncate(max_distractors);

    if distractors.len() < 2 {
        return Ok(LessonCardView::Normal(original_card));
    }

    options.extend(distractors);
    options.shuffle(&mut rand::rng());

    let quiz = QuizCard::new(original_card, options, QuizMode::Multi);
    Ok(LessonCardView::KanjiRadicalQuiz(quiz))
}
