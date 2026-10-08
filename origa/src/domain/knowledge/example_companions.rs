//! #528: example companions — a textbook-example recall slot attached to
//! its word with a random gap: `word → …other cards… → example` (owner
//! design). Eligible words are the ones actively studied
//! (`is_high_difficulty || is_in_progress`, a snapshot at lesson build
//! time — the same source `kanji_companions` uses), covered by CDN
//! examples.
//!
//! Insertion runs as the FINAL layout pass of
//! `cards_to_lesson_with_policy` (after `redistribute_core_for_spacing`),
//! so no later pass moves the slots apart. Companions are inserted
//! right-to-left: earlier targets never shift. The gap is capped by the
//! core section boundary (a companion never lands among the trailing
//! phrases) and every insertion bumps `core_count` by the same invariant
//! the kanji companions keep (`LessonProgress` stays consistent).

use rand::Rng;
use ulid::Ulid;

use super::lesson::{LessonCard, LessonCardView, LessonData};
use super::{Card, KnowledgeSet};
use crate::dictionary::example::get_word_example_refs;

/// Cap per lesson — parity with `MAX_COMPANION_CARDS_PER_LESSON`
/// (kanji companions). Also the single lever against lesson bloat in
/// ghost-heavy decks (ghost words are high-difficulty by definition).
pub(crate) const MAX_EXAMPLE_COMPANIONS_PER_LESSON: usize = 15;

/// How many OTHER core cards may sit between the word and its example.
const MAX_EXAMPLE_GAP: usize = 3;

pub(crate) fn attach_example_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    rng: &mut impl Rng,
) -> LessonData {
    let LessonData {
        mut cards,
        core_count,
    } = lesson_data;

    // Words that already own an Example SRS card (#528 v3) get NO
    // premiere: the card's own due schedule will bring the sentence back
    // («one example card per word» — the rng pick is made once, at the
    // first «didn't understand»).
    let words_with_example_card: std::collections::HashSet<&str> = knowledge_set
        .study_cards()
        .values()
        .filter_map(|sc| match sc.card() {
            Card::Example(ec) => Some(ec.word()),
            _ => None,
        })
        .collect();

    // Original slots only: companions inserted below are never visited,
    // so a companion cannot produce another companion.
    let mut planned: Vec<(usize, LessonCard)> = Vec::new();
    for (index, (slot_id, lesson_card)) in cards.iter().enumerate() {
        if planned.len() >= MAX_EXAMPLE_COMPANIONS_PER_LESSON {
            break;
        }
        // Core section only: the gap must not push a companion among the
        // trailing phrase slots (the core/tail boundary is `core_count`).
        if index >= core_count {
            break;
        }
        let Card::Vocabulary(vocab) = lesson_card.card() else {
            continue;
        };
        if words_with_example_card.contains(vocab.word().text()) {
            continue;
        }
        let Some(study_card) = knowledge_set.get_card(*slot_id) else {
            continue;
        };
        let memory = study_card.memory();
        if !(memory.is_high_difficulty() || memory.is_in_progress()) {
            continue;
        }
        let refs = get_word_example_refs(vocab.word().text());
        if refs.is_empty() {
            continue;
        }
        let pick = &refs[rng.random_range(0..refs.len())];
        let companion = LessonCard::new(
            Ulid::new(),
            LessonCardView::Example {
                card: lesson_card.card().clone(),
                sentence_id: pick.sentence_id(),
                start: pick.start(),
                end: pick.end(),
                // A companion slot IS the premiere: the first showing of
                // this sentence, attached to its word with a gap.
                premiere: true,
            },
            false,
        );
        // `word → …other cards… → example`: at least one other card
        // between, clamped by the core boundary (a word at the very end
        // of the core degrades to "right after" — better than dropping).
        let gap = rng.random_range(1..=MAX_EXAMPLE_GAP);
        let target = (index + 1 + gap).min(core_count);
        planned.push((target, companion));
    }

    let inserted = planned.len();
    if inserted == 0 {
        return LessonData { cards, core_count };
    }

    // Right-to-left insertion keeps every earlier target valid.
    planned.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));
    for (pos, companion) in planned {
        let pos = pos.min(cards.len());
        cards.insert(pos, (companion.card_id(), companion));
    }

    LessonData {
        cards,
        core_count: core_count + inserted,
    }
}

/// #528 v3: mix due example SRS cards into the lesson («like phrases»):
/// sorted by next_review_date, capped, deduped against every slot the
/// lesson already has (companions included — a card enters at most
/// once). Runs as the FINAL pass after `attach_example_companions`.
pub(crate) fn mix_due_example_cards(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    now: chrono::DateTime<chrono::Utc>,
) -> LessonData {
    const MAX_DUE_EXAMPLES_PER_LESSON: usize = 5;

    let LessonData {
        mut cards,
        core_count,
    } = lesson_data;

    let in_lesson: std::collections::HashSet<ulid::Ulid> =
        cards.iter().map(|(id, _)| *id).collect();

    let mut due: Vec<(ulid::Ulid, &super::StudyCard)> = knowledge_set
        .study_cards()
        .iter()
        .filter(|(id, sc)| {
            !in_lesson.contains(*id)
                && matches!(sc.card(), Card::Example(_))
                && !sc.memory().is_new()
                && sc.memory().next_review_date().is_some_and(|d| *d <= now)
        })
        .map(|(id, sc)| (*id, sc))
        .collect();
    due.sort_by_key(|(_, sc)| sc.memory().next_review_date());
    due.truncate(MAX_DUE_EXAMPLES_PER_LESSON);

    let inserted = due.len();
    if inserted == 0 {
        return LessonData { cards, core_count };
    }

    // No placement constraint (owner: «просто подмешиваются»): append
    // into the core section, core_count grows by the insertions.
    for (card_id, _) in due {
        let Some(sc) = knowledge_set.get_card(card_id) else {
            continue;
        };
        // premiere=false: a due review, counted in the lesson progress.
        // Offsets resolve from the CDN index entry (view mirrors the
        // generator's CardType::Example branch).
        let (word, sentence_id) = match sc.card() {
            Card::Example(ec) => (ec.word().to_string(), ec.sentence_id()),
            _ => continue,
        };
        let refs = get_word_example_refs(&word);
        let (start, end) = refs
            .iter()
            .find(|r| r.sentence_id() == sentence_id)
            .map(|r| (r.start(), r.end()))
            .unwrap_or((-1, -1));
        let view = LessonCardView::Example {
            card: sc.card().clone(),
            sentence_id,
            start,
            end,
            premiere: false,
        };
        let pos = core_count.min(cards.len());
        cards.insert(pos, (card_id, LessonCard::new(card_id, view, false)));
    }

    LessonData {
        cards,
        core_count: core_count + inserted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::example::{
        EXAMPLE_INDEX_TEST_LOCK, init_example_index, reset_example_index_for_test,
    };
    use crate::domain::VocabularyCard;
    use crate::domain::memory::{Difficulty, MemoryState, Rating, Stability};
    use crate::domain::value_objects::Question;
    use chrono::Utc;
    use rand::SeedableRng;

    pub(super) const FIXTURE_INDEX: &str = r#"{ "v": 1, "h": "", "s": 2, "words": {
        "たべる": { "refs": [[0, 0, 3]] },
        "よむ":   { "refs": [[1, 0, 2]] }
    } }"#;

    pub(super) fn lock() -> std::sync::MutexGuard<'static, ()> {
        EXAMPLE_INDEX_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    pub(super) fn seeded() -> rand::rngs::StdRng {
        rand::rngs::StdRng::seed_from_u64(42)
    }

    /// `(word, Some((stability, difficulty, rating)))` — a reviewed card;
    /// `None` — a brand-new card.
    type Review = (f64, f64, Rating);

    fn set_with_words(words: &[(&str, Option<Review>)]) -> (KnowledgeSet, Vec<Ulid>) {
        let mut ks = KnowledgeSet::new();
        let mut ids = Vec::new();
        for (word, review) in words {
            let card = Card::Vocabulary(VocabularyCard::new(
                Question::new(word.to_string()).unwrap(),
            ));
            let sc = ks.create_card(card).expect("create card");
            if let Some((stability, difficulty, rating)) = *review {
                ks.study_cards.get_mut(sc.card_id()).unwrap().apply_review(
                    MemoryState::new(
                        Stability::new(stability).unwrap(),
                        Difficulty::new(difficulty).unwrap(),
                        Utc::now(),
                    ),
                    rating,
                );
            }
            ids.push(*sc.card_id());
        }
        (ks, ids)
    }

    /// Core section of `core_len` slots + an optional tail slot; the tail
    /// stays ineligible so a companion must never cross into it.
    fn lesson_data(ks: &KnowledgeSet, ids: &[Ulid], core_len: usize, tail: usize) -> LessonData {
        let mut cards: Vec<(Ulid, LessonCard)> = ids
            .iter()
            .chain(std::iter::repeat(&ids[0]))
            .take(core_len + tail)
            .map(|id| {
                let sc = ks.get_card(*id).expect("card");
                (
                    *id,
                    LessonCard::new(*id, LessonCardView::Normal(sc.card().clone()), false),
                )
            })
            .collect();
        // Tail slots get fresh ids so they are distinct entities.
        for slot in cards.iter_mut().skip(core_len) {
            *slot = (Ulid::new(), slot.1.clone());
        }
        LessonData {
            cards,
            core_count: core_len,
        }
    }

    #[test]
    fn eligible_word_gets_a_companion_after_a_gap() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        let (ks, ids) = set_with_words(&[
            ("たべる", Some((5.0, 3.0, Rating::Good))), // in_progress + covered
            ("はしる", Some((25.0, 3.0, Rating::Easy))), // known, uncovered fillers
            ("かく", Some((25.0, 3.0, Rating::Easy))),
            ("はなす", Some((25.0, 3.0, Rating::Easy))),
        ]);
        let data = lesson_data(&ks, &ids, 4, 0);
        let out = attach_example_companions(data, &ks, &mut seeded());

        assert_eq!(out.cards.len(), 5, "exactly one companion inserted");
        let pos = out
            .cards
            .iter()
            .position(|(_, c)| matches!(c.view(), LessonCardView::Example { .. }))
            .expect("companion present");
        assert!(
            (2..=4).contains(&pos),
            "companion must sit after a gap of 1..=3 other cards (word at 0), got {pos}"
        );
        assert_eq!(out.core_count, 5, "core_count grows by the insertion");
    }

    #[test]
    fn new_and_known_words_get_no_companion() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        let (ks, ids) = set_with_words(&[
            ("たべる", None),                          // new
            ("よむ", Some((25.0, 3.0, Rating::Easy))), // known
        ]);
        let data = lesson_data(&ks, &ids, 2, 0);
        let out = attach_example_companions(data, &ks, &mut seeded());
        assert_eq!(out.cards.len(), 2, "no companions for new/known");
    }

    #[test]
    fn uncovered_words_get_no_companion() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        let (ks, ids) = set_with_words(&[("はしる", Some((5.0, 3.0, Rating::Good)))]);
        let data = lesson_data(&ks, &ids, 1, 0);
        let out = attach_example_companions(data, &ks, &mut seeded());
        assert_eq!(out.cards.len(), 1, "no CDN coverage — no companion");
    }

    #[test]
    fn companion_never_crosses_into_the_tail() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        // Covered in-progress word LAST in the core + one tail slot: the
        // clamp must keep the companion inside the core section.
        let (ks, ids) = set_with_words(&[("たべる", Some((5.0, 3.0, Rating::Good)))]);
        let data = lesson_data(&ks, &ids, 1, 1);
        let out = attach_example_companions(data, &ks, &mut seeded());

        let pos = out
            .cards
            .iter()
            .position(|(_, c)| matches!(c.view(), LessonCardView::Example { .. }))
            .expect("companion inserted");
        assert!(
            pos < out.core_count,
            "companion must stay in the core (pos {pos}, core {})",
            out.core_count
        );
    }

    #[test]
    fn cap_limits_companions_per_lesson() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        // 20 UNIQUE covered in-progress words (KnowledgeSet rejects
        // duplicate words, so the fixture index is generated).
        let words: Vec<String> = (0..20).map(|i| format!("言葉{i}")).collect();
        let index_json = format!(
            r#"{{ "v": 1, "h": "", "s": 20, "words": {{ {} }} }}"#,
            words
                .iter()
                .enumerate()
                .map(|(i, w)| format!(r#""{w}": {{ "refs": [[{i}, 0, 2]] }}"#))
                .collect::<Vec<_>>()
                .join(", ")
        );
        // init_example_index is idempotent — reset first or the generated
        // words stay invisible behind the shared fixture loaded above.
        reset_example_index_for_test();
        init_example_index(&index_json).expect("generated fixture index");
        let words: Vec<(&str, Option<Review>)> = words
            .iter()
            .map(|w| (w.as_str(), Some((5.0, 3.0, Rating::Good))))
            .collect();
        let (ks, ids) = set_with_words(&words);
        let data = lesson_data(&ks, &ids, 20, 0);
        let out = attach_example_companions(data, &ks, &mut seeded());

        let companions = out
            .cards
            .iter()
            .filter(|(_, c)| matches!(c.view(), LessonCardView::Example { .. }))
            .count();
        assert_eq!(companions, MAX_EXAMPLE_COMPANIONS_PER_LESSON);
        assert_eq!(
            out.core_count,
            20 + MAX_EXAMPLE_COMPANIONS_PER_LESSON,
            "core_count grows by exactly the inserted companions"
        );
    }
}

#[cfg(test)]
mod v3_tests {
    use super::tests::{FIXTURE_INDEX, lock, seeded};
    use super::*;
    use crate::dictionary::example::{init_example_index, reset_example_index_for_test};
    use crate::domain::ExampleCard;
    use crate::domain::memory::{Difficulty, MemoryState, Rating, Stability};
    use crate::domain::value_objects::Question;
    use chrono::{Duration, Utc};

    fn ks_with_rated_example_card() -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        let card = Card::Vocabulary(crate::domain::VocabularyCard::new(
            Question::new("たべる".to_string()).unwrap(),
        ));
        let sc = ks.create_card(card).unwrap();
        ks.study_cards.get_mut(sc.card_id()).unwrap().apply_review(
            MemoryState::new(
                Stability::new(5.0).unwrap(),
                Difficulty::new(3.0).unwrap(),
                Utc::now() - Duration::days(3),
            ),
            Rating::Good,
        );
        // An example card rated 3 days ago — due now.
        let ex = ks
            .create_card(Card::Example(ExampleCard::new("たべる", 0)))
            .unwrap();
        ks.study_cards.get_mut(ex.card_id()).unwrap().apply_review(
            MemoryState::new(
                Stability::new(0.5).unwrap(),
                Difficulty::new(6.0).unwrap(),
                Utc::now() - Duration::days(3),
            ),
            Rating::Again,
        );
        ks
    }

    fn lesson_with_word(ks: &KnowledgeSet) -> (LessonData, Ulid) {
        let id = *ks
            .study_cards()
            .values()
            .find(|sc| matches!(sc.card(), Card::Vocabulary(_)))
            .unwrap()
            .card_id();
        let sc = ks.get_card(id).unwrap();
        (
            LessonData {
                cards: vec![(
                    id,
                    LessonCard::new(id, LessonCardView::Normal(sc.card().clone()), false),
                )],
                core_count: 1,
            },
            id,
        )
    }

    #[test]
    fn companion_is_excluded_when_the_word_already_owns_an_example_card() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        let ks = ks_with_rated_example_card();
        let (data, _) = lesson_with_word(&ks);
        let out = attach_example_companions(data, &ks, &mut seeded());
        assert_eq!(
            out.cards.len(),
            1,
            "no premiere when an example SRS card exists for the word"
        );
    }

    #[test]
    fn due_example_card_enters_the_lesson_exactly_once() {
        let _guard = lock();
        reset_example_index_for_test();
        init_example_index(FIXTURE_INDEX).expect("fixture index");

        let ks = ks_with_rated_example_card();
        let (data, _) = lesson_with_word(&ks);
        let out = mix_due_example_cards(data, &ks, Utc::now());

        let example_card_id = *ks
            .study_cards()
            .iter()
            .find(|(_, sc)| matches!(sc.card(), Card::Example(_)))
            .unwrap()
            .0;
        let entries = out
            .cards
            .iter()
            .filter(|(id, _)| *id == example_card_id)
            .count();
        assert_eq!(entries, 1, "due example enters exactly once");
        assert_eq!(out.core_count, 2, "core_count grows by the insertion");
        // The inserted slot is a REVIEW (premiere=false).
        let view = out
            .cards
            .iter()
            .find(|(id, _)| *id == example_card_id)
            .unwrap();
        assert!(matches!(
            view.1.view(),
            LessonCardView::Example {
                premiere: false,
                ..
            }
        ));
    }
}
