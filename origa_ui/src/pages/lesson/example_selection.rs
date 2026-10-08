//! #528 owner fix: pick the EASIEST example for a premiere companion.
//!
//! Tanaka sentences routinely carry rare vocabulary (measured: even a
//! user who knows every shipped JLPT set meets an unknown word in ~1/3
//! of sentences). When a word owns several examples, the companion slot
//! is retargeted to the sentence with the fewest unknown WORD tokens —
//! counted against the user's ACTUAL vocabulary (every vocabulary card
//! in their deck), base or surface form. Ties keep the rng pick made at
//! lesson build time. Runs once, right after the lesson is built, where
//! the CDN sentence cache is reachable.

use std::collections::HashSet;

use origa::dictionary::example::get_word_example_refs;
use origa::domain::{Card, LessonCard, LessonCardView};
use origa::domain::{PartOfSpeech, tokenize_text};
use origa::traits::UserRepository;
use tracing::warn;

use crate::loaders::example_loader::load_example_detail;
use crate::repository::HybridUserRepository;

/// Retarget each PREMIERE example slot to its easiest sentence. Due
/// reviews are fixed at card creation (their sid lives in the SRS card)
/// and are left untouched.
pub async fn pick_easiest_examples(
    lesson_cards: &mut [(ulid::Ulid, LessonCard)],
    repository: &HybridUserRepository,
) {
    let Some(known) = known_vocabulary(repository).await else {
        return;
    };

    for (slot_id, lesson_card) in lesson_cards.iter_mut() {
        let LessonCardView::Example {
            card,
            sentence_id,
            premiere: true,
            ..
        } = lesson_card.view()
        else {
            continue;
        };
        let Card::Vocabulary(vocab) = card else {
            continue;
        };
        let word = vocab.word().text().to_string();
        let refs = get_word_example_refs(&word);
        if refs.len() < 2 {
            continue; // nothing to choose from — keep the rng pick
        }

        let mut best: Option<(u32, i32, i32, usize)> = None;
        for r in &refs {
            let Ok(detail) = load_example_detail(r.sentence_id()).await else {
                continue;
            };
            let score = unknown_word_tokens(&detail.text, &known);
            let better = best.as_ref().map_or(true, |b| score < b.3);
            if better {
                best = Some((r.sentence_id(), r.start(), r.end(), score));
            }
        }

        if let Some((best_sid, best_start, best_end, _)) = best {
            if best_sid != *sentence_id {
                let short_term = lesson_card.is_short_term();
                *lesson_card = LessonCard::new(
                    *slot_id,
                    LessonCardView::Example {
                        card: card.clone(),
                        sentence_id: best_sid,
                        start: best_start,
                        end: best_end,
                        premiere: true,
                        // A premiere is always text (first encounter).
                        audio: false,
                    },
                    short_term,
                );
            }
        }
    }
}

/// Every vocabulary word in the user's deck (studied or studying — the
/// owner complained about words NEVER seen, not merely not memorized).
async fn known_vocabulary(repository: &HybridUserRepository) -> Option<HashSet<String>> {
    match repository.get_current_user().await {
        Ok(Some(user)) => Some(
            user.knowledge_set()
                .study_cards()
                .values()
                .filter_map(|sc| match sc.card() {
                    Card::Vocabulary(v) => Some(v.word().text().to_string()),
                    _ => None,
                })
                .collect(),
        ),
        Ok(None) => None,
        Err(e) => {
            warn!(error = ?e, "Easiest-example pick degraded: no current user");
            None
        },
    }
}

/// Content words (particles/aux/punctuation excluded via
/// `is_vocabulary_word`) whose base AND surface forms are both unknown.
fn unknown_word_tokens(sentence: &str, known: &HashSet<String>) -> usize {
    let Ok(tokens) = tokenize_text(sentence) else {
        return usize::MAX; // untokenizable — treat as the worst pick
    };
    tokens
        .iter()
        .filter(|t| t.part_of_speech().is_vocabulary_word())
        .filter(|t| {
            !known.contains(t.orthographic_base_form())
                && !known.contains(t.orthographic_surface_form())
        })
        .count()
}

/// #528 offline fix: warm the sentence cache for every example slot of
/// the built lesson (premieres retargeted above + due reviews), so card
/// renders during the lesson never wait on the network.
pub fn prefetch_example_sentences(lesson_cards: &[(ulid::Ulid, LessonCard)]) {
    for (_, lesson_card) in lesson_cards {
        if let LessonCardView::Example { sentence_id, .. } = lesson_card.view() {
            let sid = *sentence_id;
            leptos::task::spawn_local(async move {
                if let Err(e) = load_example_detail(sid).await {
                    tracing::debug!(sid, error = ?e, "Example prefetch failed (cache-only)");
                }
            });
        }
    }
}

// Keep the unused-import lint honest if PartOfSpeech is only referenced
// through TokenInfo accessors below.
#[allow(dead_code)]
fn _pos_marker(pos: PartOfSpeech) -> bool {
    pos.is_vocabulary_word()
}
