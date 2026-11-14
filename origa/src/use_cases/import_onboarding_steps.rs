//! Phase workers of the onboarding sets import: vocabulary, kanji and
//! grammar card creation. Free functions over `&mut User` — the pipeline
//! orchestration (chunk sizes, progress events, timings) lives in
//! `import_onboarding_sets`.

use std::collections::HashSet;
use std::future::Future;

use tracing::{debug, warn};

use crate::dictionary::grammar::get_rules_by_level;
use crate::dictionary::kanji::get_kanji_list;
use crate::domain::{
    Card, GrammarRuleCard, JapaneseLevel, KanjiCard, NativeLanguage, OrigaError, StudyCard, User,
    VocabularyCard, WellKnownSet,
};
use crate::use_cases::import_onboarding_sets::ImportOnboardingResult;
use crate::use_cases::{ImportPhase, ImportProgressEvent};

/// Sets processed per progress tick (and host yield) in the vocabulary
/// phase: ~16 sets ≈ one frame of tokenization work on WASM.
const VOCAB_PROGRESS_CHUNK: usize = 16;
/// Kanji examined per progress tick in the kanji phase.
const KANJI_PROGRESS_CHUNK: usize = 100;

/// Creates vocabulary cards from every set's word list; each set also
/// contributes its own level to `grammar_levels`. Yields to the host (via
/// the progress future) every [`VOCAB_PROGRESS_CHUNK`] sets so a WASM UI
/// keeps rendering during the tokenization-heavy phase.
pub(crate) async fn import_vocabulary_from_sets<F, Fut>(
    user: &mut User,
    sets: Vec<(String, WellKnownSet)>,
    native_language: &NativeLanguage,
    result: &mut ImportOnboardingResult,
    grammar_levels: &mut HashSet<JapaneseLevel>,
    on_progress: &F,
) where
    F: Fn(ImportProgressEvent) -> Fut,
    Fut: Future<Output = ()>,
{
    let total = sets.len();
    for (done, (set_id, set)) in sets.into_iter().enumerate() {
        let done = done + 1;
        debug!(set_id = %set_id, words_count = set.words().len(), "Processing set");

        let set_level = *set.level();
        // Grammar is keyed by JLPT level, so any set contributes its own
        // level — previously gated to "Jlpt"-prefixed ids only, which left
        // Minna / Irodori / Duolingo without grammar.
        grammar_levels.insert(set_level);

        let words_result = VocabularyCard::from_text(&set.words().join(" "), native_language);
        result.skipped_no_translation += words_result.skipped_no_translation.len();

        for vocab_card in words_result.cards {
            if create_vocabulary_card(user, vocab_card, &mut result.skipped_duplicates).is_ok() {
                result.created_vocabulary += 1;
            }
        }
        result.imported_set_ids.push(set_id);

        if done == total || done % VOCAB_PROGRESS_CHUNK == 0 {
            on_progress(ImportProgressEvent {
                phase: ImportPhase::Vocabulary,
                done,
                total,
            })
            .await;
        }
    }
}

/// Imports grammar rules for every accumulated level. Fast dictionary
/// lookups — one progress tick per level is enough.
pub(crate) async fn import_grammar_for_levels<F, Fut>(
    user: &mut User,
    grammar_levels: &HashSet<JapaneseLevel>,
    result: &mut ImportOnboardingResult,
    on_progress: &F,
) where
    F: Fn(ImportProgressEvent) -> Fut,
    Fut: Future<Output = ()>,
{
    debug!(levels = ?grammar_levels, "Importing grammar rules for onboarding levels");

    let levels: Vec<&JapaneseLevel> = grammar_levels.iter().collect();
    let total: usize = levels
        .iter()
        .map(|level| get_rules_by_level(level).len())
        .sum();
    let mut done = 0usize;
    for level in levels {
        let rules = get_rules_by_level(level);
        for rule in rules {
            if let Ok(grammar_card) = GrammarRuleCard::new(*rule.rule_id()) {
                match user.create_card(Card::Grammar(grammar_card)) {
                    Ok(_) => result.created_grammar += 1,
                    Err(OrigaError::DuplicateCard { .. }) => result.skipped_duplicates += 1,
                    Err(e) => warn!(error = ?e, "Failed to create grammar card"),
                }
            }
            done += 1;
        }
        on_progress(ImportProgressEvent {
            phase: ImportPhase::Grammar,
            done,
            total,
        })
        .await;
    }
}

/// Imports kanji directly from the kanji dictionary for every level ≤
/// target — symmetric with grammar. Previously kanji were extracted FROM
/// vocabulary words, which produced wrong results: a word at N4 might
/// contain an N3 kanji, so the user got kanji they never asked for.
/// Kanji→vocab companion cards are still created (`create_kanji_card`).
pub(crate) async fn import_kanji_up_to_level<F, Fut>(
    user: &mut User,
    target_level: JapaneseLevel,
    result: &mut ImportOnboardingResult,
    created_kanji_chars: &mut HashSet<String>,
    on_progress: &F,
) where
    F: Fn(ImportProgressEvent) -> Fut,
    Fut: Future<Output = ()>,
{
    debug!(target_level = ?target_level, "Importing kanji from dictionary");
    let total: usize = JapaneseLevel::ALL
        .iter()
        .filter(|level| **level <= target_level)
        .map(|level| get_kanji_list(level).len())
        .sum();
    let mut done = 0usize;
    for level in JapaneseLevel::ALL {
        if level > target_level {
            continue;
        }
        for info in get_kanji_list(&level) {
            let kanji_char = info.kanji().to_string();
            if created_kanji_chars.contains(&kanji_char) {
                continue;
            }
            if create_kanji_card(user, &kanji_char, result).is_ok() {
                result.created_kanji += 1;
                created_kanji_chars.insert(kanji_char);
            }
            done += 1;
            if done == total || done % KANJI_PROGRESS_CHUNK == 0 {
                on_progress(ImportProgressEvent {
                    phase: ImportPhase::Kanji,
                    done,
                    total,
                })
                .await;
            }
        }
    }
}

pub(crate) fn create_vocabulary_card(
    user: &mut User,
    vocab_card: VocabularyCard,
    skipped_duplicates: &mut usize,
) -> Result<StudyCard, OrigaError> {
    let card = Card::Vocabulary(vocab_card);
    match user.create_card(card) {
        Ok(study_card) => Ok(study_card),
        Err(OrigaError::DuplicateCard { question }) => {
            debug!(word = %question, "Duplicate vocabulary card, skipping");
            *skipped_duplicates += 1;
            Err(OrigaError::DuplicateCard { question })
        },
        Err(e) => Err(e),
    }
}

pub(crate) fn create_kanji_card(
    user: &mut User,
    kanji_char: &str,
    result: &mut ImportOnboardingResult,
) -> Result<StudyCard, OrigaError> {
    let kanji_card = match KanjiCard::new(kanji_char.to_string()) {
        Ok(kanji_card) => kanji_card,
        Err(e) => {
            warn!(kanji = %kanji_char, error = ?e, "Failed to create kanji card");
            return Err(e);
        },
    };
    let card = Card::Kanji(kanji_card);
    match user.create_card(card) {
        Ok(study_card) => {
            let companions = user.create_companion_vocab_cards(kanji_char);
            result.created_vocabulary += companions.len();
            Ok(study_card)
        },
        Err(OrigaError::DuplicateCard { question }) => {
            debug!(kanji = %question, "Duplicate kanji card, skipping");
            result.skipped_duplicates += 1;
            Err(OrigaError::DuplicateCard { question })
        },
        Err(e) => Err(e),
    }
}
