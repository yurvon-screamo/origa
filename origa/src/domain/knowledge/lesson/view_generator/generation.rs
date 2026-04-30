use crate::dictionary::grammar::{FormatAction, FormatActionGroup, get_rule_by_id};
use crate::dictionary::kanji::get_kanji_info;
use crate::domain::grammar::apply_format_actions;
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::NativeLanguage;
use crate::domain::{Card, OrigaError, PartOfSpeech};
use rand::{Rng, prelude::IndexedRandom, seq::SliceRandom};

use super::super::types::{
    GrammarInfo, GrammarQuizCard, LessonCardView, QuizCard, QuizOption, YesNoCard,
};
use super::QUIZ_OPTIONS_COUNT;

pub(crate) fn generate_quiz(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
) -> Result<LessonCardView, OrigaError> {
    match &original_card {
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) | Card::Phrase(_) => {},
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
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) | Card::Phrase(_) => {},
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

pub(crate) fn generate_phrase_quiz(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
) -> Option<LessonCardView> {
    let phrase_card = match &original_card {
        Card::Phrase(pc) => pc,
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => return None,
    };

    let audio_file = format!("{}.opus", phrase_card.phrase_id());
    let correct_text = original_card.answer(lang).ok()?.text().to_string();

    let mut distractors: Vec<String> = same_type_cards
        .iter()
        .filter_map(|c| match c {
            Card::Phrase(_) => c
                .answer(lang)
                .ok()
                .map(|a| a.text().to_string())
                .filter(|text| text != &correct_text),
            Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => None,
        })
        .collect();

    if distractors.is_empty() {
        return None;
    }

    distractors.shuffle(&mut rand::rng());

    let max_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
    let selected: Vec<String> = distractors.into_iter().take(max_distractors).collect();

    let mut options: Vec<QuizOption> = selected
        .into_iter()
        .map(|text| QuizOption::new(text, false))
        .collect();

    options.push(QuizOption::new(correct_text, true));
    options.shuffle(&mut rand::rng());

    Some(LessonCardView::PhraseListen {
        card: original_card,
        audio_file,
        options,
    })
}

pub(crate) fn generate_kanji_reading_quiz(
    original_card: Card,
    same_type_cards: &[Card],
) -> Result<LessonCardView, OrigaError> {
    let kanji_card = match &original_card {
        Card::Kanji(kc) => kc,
        Card::Vocabulary(_) | Card::Grammar(_) | Card::Phrase(_) => {
            return Ok(LessonCardView::Normal(original_card));
        },
    };

    let kanji_str = kanji_card.kanji().text();
    let info = match get_kanji_info(kanji_str) {
        Ok(info) => info,
        Err(_) => return Ok(LessonCardView::Normal(original_card)),
    };

    let mut all_readings: Vec<String> = info.on_readings().to_vec();
    all_readings.extend(info.kun_readings().iter().cloned());

    if all_readings.is_empty() {
        return Ok(LessonCardView::Normal(original_card));
    }

    let target_readings: std::collections::HashSet<String> = all_readings.iter().cloned().collect();

    let correct_reading = all_readings
        .choose(&mut rand::rng())
        .expect("all_readings guaranteed non-empty after is_empty check")
        .clone();

    let mut distractors: Vec<String> = same_type_cards
        .iter()
        .filter_map(|c| match c {
            Card::Kanji(kc) => {
                let other_info = get_kanji_info(kc.kanji().text()).ok()?;
                let mut readings = other_info.on_readings().to_vec();
                readings.extend(other_info.kun_readings().iter().cloned());
                Some(readings)
            },
            Card::Vocabulary(_) | Card::Grammar(_) | Card::Phrase(_) => None,
        })
        .flatten()
        .filter(|r| !target_readings.contains(r))
        .collect();

    let mut seen = std::collections::HashSet::new();
    distractors.retain(|r| seen.insert(r.clone()));

    distractors.shuffle(&mut rand::rng());

    if distractors.len() < 3 {
        return Ok(LessonCardView::Normal(original_card));
    }

    let selected_distractors: Vec<String> = distractors.into_iter().take(3).collect();

    let mut options: Vec<QuizOption> = selected_distractors
        .into_iter()
        .map(|text| QuizOption::new(text, false))
        .collect();

    options.push(QuizOption::new(correct_reading, true));
    options.shuffle(&mut rand::rng());

    let quiz = QuizCard::new(original_card, options);
    Ok(LessonCardView::KanjiReadingQuiz(quiz))
}

pub(crate) fn generate_grammar_quiz(
    original_card: Card,
    knowledge_set: &KnowledgeSet,
) -> Result<LessonCardView, OrigaError> {
    let grammar_rule_card = match &original_card {
        Card::Grammar(grc) => grc,
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Phrase(_) => {
            return Ok(LessonCardView::Normal(original_card));
        },
    };

    let rule = get_rule_by_id(grammar_rule_card.rule_id()).ok_or_else(|| {
        OrigaError::GrammarRuleNotFound {
            rule_id: *grammar_rule_card.rule_id(),
        }
    })?;

    if !rule.has_format_map() {
        return Ok(LessonCardView::Normal(original_card));
    }

    let format_map = rule
        .format_map()
        .expect("has_format_map guaranteed Some above");

    let supported_pos: Vec<&PartOfSpeech> = format_map.keys().collect();
    let applicable_pos = grammar_rule_card
        .apply_to()
        .iter()
        .find(|pos| supported_pos.contains(pos))
        .cloned()
        .ok_or_else(|| OrigaError::GrammarFormatError {
            reason: "No supported part of speech".to_string(),
        })?;

    let matching_vocab = find_known_vocab_for_pos(knowledge_set, &applicable_pos);
    if matching_vocab.is_empty() {
        return Ok(LessonCardView::Normal(original_card));
    }

    let mut rng = rand::rng();
    let (_vocab_card, word_text) = matching_vocab
        .choose(&mut rng)
        .expect("matching_vocab guaranteed non-empty")
        .clone();

    let rules = format_map
        .get(&applicable_pos)
        .expect("applicable_pos is from format_map keys");

    let correct_text = apply_format_actions(&word_text, rules, &applicable_pos)?;

    let needed_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
    let distractors = generate_distractors(
        rules,
        &word_text,
        &applicable_pos,
        &correct_text,
        needed_distractors,
        &mut rng,
    );

    if distractors.len() < needed_distractors {
        return Ok(LessonCardView::Normal(original_card));
    }

    let mut options: Vec<QuizOption> = distractors
        .into_iter()
        .map(|text| QuizOption::new(text, false))
        .collect();
    options.push(QuizOption::new(correct_text, true));
    options.shuffle(&mut rng);

    let quiz = QuizCard::new(original_card.clone(), options);

    let grammar_title = grammar_rule_card
        .title(&DEFAULT_LANG)
        .map(|q| q.text().to_string())
        .unwrap_or_else(|_| grammar_rule_card.rule_id().to_string());
    let grammar_desc = grammar_rule_card
        .description(&DEFAULT_LANG)
        .map(|a| a.text().to_string())
        .unwrap_or_default();

    let grammar_info = GrammarInfo::new(
        Some(*grammar_rule_card.rule_id()),
        grammar_title,
        grammar_desc,
    );

    let grammar_quiz = GrammarQuizCard::new(original_card, grammar_info, word_text, quiz);

    Ok(LessonCardView::GrammarQuiz(grammar_quiz))
}

const DEFAULT_LANG: NativeLanguage = NativeLanguage::Russian;

fn find_known_vocab_for_pos(
    knowledge_set: &KnowledgeSet,
    pos: &PartOfSpeech,
) -> Vec<(crate::domain::VocabularyCard, String)> {
    knowledge_set
        .study_cards()
        .values()
        .filter(|sc| sc.memory().is_known_card() || sc.memory().is_in_progress())
        .filter_map(|sc| match sc.card() {
            Card::Vocabulary(v) => {
                let word = v.word().text().to_string();
                let vocab_pos = v.part_of_speech().ok()?;
                if vocab_pos == *pos {
                    Some((v.clone(), word))
                } else {
                    None
                }
            },
            _ => None,
        })
        .collect()
}

fn generate_distractors(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    correct_text: &str,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut distractors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    seen.insert(correct_text.to_string());

    let mut attempts = 0;
    let max_attempts = count * 10;

    while distractors.len() < count && attempts < max_attempts {
        attempts += 1;

        if let Some(distractor) = apply_mutated_pattern(rules, source_word, pos, rng) {
            if !seen.contains(&distractor) {
                seen.insert(distractor.clone());
                distractors.push(distractor);
            }
        }
    }

    distractors
}

fn apply_mutated_pattern(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    rng: &mut impl Rng,
) -> Option<String> {
    let mutable_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, a)| a.group() != FormatActionGroup::Universal)
        .map(|(i, _)| i)
        .collect();

    if mutable_indices.is_empty() {
        return None;
    }

    let idx = mutable_indices.choose(rng)?;
    let original_action = &rules[*idx];
    let alternatives = original_action.mutation_alternatives();

    if alternatives.is_empty() {
        return None;
    }

    let alternative = alternatives.choose(rng)?;
    let mut mutated_rules: Vec<FormatAction> = rules.to_vec();
    mutated_rules[*idx] = (*alternative).clone();

    apply_format_actions(source_word, &mutated_rules, pos).ok()
}
