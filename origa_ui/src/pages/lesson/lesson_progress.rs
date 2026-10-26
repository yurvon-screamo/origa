use leptos::prelude::*;
use origa::domain::LessonCardView;

use super::lesson_state::LessonState;

/// Steps a lesson slot contributes to the common counter. A counter
/// bindings pack (issue #415) is ONE slot of N per-binding showings, and
/// the owner UX decision is that those showings live in the COMMON
/// header counter (no nested «k / N» inside the card) — so the pack
/// counts as N steps while every other view counts as one. The empty
/// pack never reaches a lesson (the view generator degrades it to
/// Normal), `.max(1)` only keeps the counter honest if it ever does.
pub(super) fn slot_step_count(view: &LessonCardView) -> usize {
    match view {
        LessonCardView::CounterBindings { items, .. } => items.len().max(1),
        _ => 1,
    }
}

/// The header progress in step units: `(current, total, core)` — see
/// `slot_step_count`. `binding_step` is the counter-bindings session's
/// published index (0-based) or `None` outside a pack; the pack's own
/// size is derived from the current slot's view (single source), never
/// from the signal.
pub(super) fn display_progress(
    state: &LessonState,
    binding_step: Option<usize>,
) -> (usize, usize, usize) {
    let steps_of = |id: &ulid::Ulid| -> usize {
        state
            .cards
            .get(id)
            .map(|card| slot_step_count(card.view()))
            .unwrap_or(1)
    };
    let total: usize = state.card_ids.iter().map(steps_of).sum();
    // Phrases only ever follow `core_count` (the builder appends them;
    // packs are core cards), so the core section simply sums the same
    // steps over its prefix.
    let core: usize = state
        .card_ids
        .iter()
        .take(state.core_count)
        .map(steps_of)
        .sum();
    let steps_before: usize = state
        .card_ids
        .iter()
        .take(state.current_index)
        .map(steps_of)
        .sum();
    // The step only applies while the CURRENT slot is the pack. The
    // container advances `current_index` before the session unmounts, so
    // one render tick can see a stale `Some(k)` against an already-next
    // card — without the gate the pack's steps would double for that
    // frame («23/13»). The clamp guards the frozen last binding (k is at
    // most pack_size − 1 by construction).
    let pack_steps = state
        .card_ids
        .get(state.current_index)
        .and_then(|id| state.cards.get(id))
        .and_then(|card| match card.view() {
            LessonCardView::CounterBindings { items, .. } => Some(items.len().max(1)),
            _ => None,
        });
    let step_in_pack = binding_step
        .filter(|_| pack_steps.is_some())
        .map(|k| k.min(pack_steps.unwrap_or(1).saturating_sub(1)))
        .unwrap_or(0);
    let current = steps_before + step_in_pack + 1;
    (current, total, core)
}

#[component]
pub fn LessonProgress(
    current: Signal<usize>,
    total: Signal<usize>,
    #[prop(optional, into)] core_count: Signal<usize>,
) -> impl IntoView {
    let has_phrases = move || {
        let cc = core_count.get();
        let t = total.get();
        cc > 0 && cc < t
    };

    let label_text = move || {
        let c = current.get();
        let t = total.get();
        let cc = core_count.get();
        if cc > 0 && cc < t {
            let phrase_total = t - cc;
            let core_done = c.min(cc);
            let phrase_done = c.saturating_sub(cc);
            format!("{}/{} + {}/{}", core_done, cc, phrase_done, phrase_total)
        } else {
            format!("{}/{}", c, t)
        }
    };

    let core_fill_width = move || {
        let c = current.get();
        let t = total.get();
        let cc = core_count.get();
        if t == 0 || cc == 0 {
            return 0.0_f64;
        }
        let core_done = (c.min(cc)) as f64;
        let section_ratio = cc as f64 / t as f64;
        (core_done / cc as f64 * section_ratio * 100.0).min(100.0)
    };

    let phrase_fill_width = move || {
        let c = current.get();
        let t = total.get();
        let cc = core_count.get();
        let phrase_total = t.saturating_sub(cc);
        if t == 0 || phrase_total == 0 {
            return 0.0_f64;
        }
        let phrase_done = c.saturating_sub(cc) as f64;
        let section_ratio = phrase_total as f64 / t as f64;
        (phrase_done / phrase_total as f64 * section_ratio * 100.0).min(100.0)
    };

    let divider_position = move || {
        let t = total.get();
        let cc = core_count.get();
        if t == 0 {
            0.0_f64
        } else {
            cc as f64 / t as f64 * 100.0
        }
    };

    let single_fill_width = move || {
        let c = current.get();
        let t = total.get();
        if t == 0 {
            0.0_f64
        } else {
            (c as f64 / t as f64 * 100.0).min(100.0)
        }
    };

    view! {
        <div class="flex items-center gap-2 w-full">
            <Show
                when=has_phrases
                fallback=move || {
                    view! {
                        <div class="flex-1 progress-track-lesson">
                            <div
                                class="progress-fill-core"
                                style=move || format!("width: {}%", single_fill_width() as u32)
                            ></div>
                        </div>
                    }
                }
            >
                <div class="flex-1 progress-container">
                    <div class="progress-track-lesson">
                        <div
                            class="progress-fill-core"
                            style=move || format!("width: {}%", core_fill_width() as u32)
                        ></div>
                        <div
                            class="progress-fill-phrase"
                            style=move || {
                                let divider = divider_position();
                                format!(
                                    "width: {}%; left: {}%",
                                    phrase_fill_width() as u32,
                                    divider as u32,
                                )
                            }
                        ></div>
                    </div>
                    <div
                        class="progress-divider"
                        style=move || format!("left: {}%", divider_position() as u32)
                    ></div>
                </div>
            </Show>

            <span class="font-mono text-[10px] shrink-0" data-testid="lesson-progress-text">
                {label_text}
            </span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{Card, CounterBindingItem, LessonCard};
    use std::collections::HashMap;
    use ulid::Ulid;

    /// A vocabulary card fixture via serde (the translation dictionary
    /// is not loaded in native tests — `from_known_word` would fail).
    fn normal_slot(word: &str) -> (Ulid, LessonCard) {
        let card: Card = serde_json::from_str(&format!(
            r#"{{"Vocabulary":{{"word":{{"text":"{word}"}},"reverse_side":null,"pos":null}}}}"#
        ))
        .expect("deserialize vocab card fixture");
        let id = Ulid::new();
        (id, LessonCard::new(id, LessonCardView::Normal(card), false))
    }

    fn pack_slot(bindings: u8) -> (Ulid, LessonCard) {
        let items: Vec<_> = (1..=bindings).map(CounterBindingItem::new).collect();
        let card = Card::Counter(origa::domain::CounterCard::new("本"));
        let id = Ulid::new();
        (
            id,
            LessonCard::new(id, LessonCardView::CounterBindings { card, items }, false),
        )
    }

    fn state_from(slots: Vec<(Ulid, LessonCard)>, core_count: usize) -> LessonState {
        let cards: HashMap<_, _> = slots.iter().cloned().collect();
        let card_ids: Vec<_> = slots.into_iter().map(|(id, _)| id).collect();
        LessonState {
            cards,
            card_ids,
            core_count,
            ..LessonState::default()
        }
    }

    /// A pack-free lesson must keep the exact legacy numbers — the
    /// formula change is invisible for every existing lesson shape.
    #[test]
    fn pack_free_lesson_matches_legacy_formula() {
        let slots: Vec<_> = (0..4).map(|i| normal_slot(&format!("word{i}"))).collect();
        let state = state_from(slots, 4);
        for index in 0..4 {
            let mut s = state.clone();
            s.current_index = index;
            let (current, total, core) = display_progress(&s, None);
            assert_eq!(current, index + 1);
            assert_eq!(total, 4);
            assert_eq!(core, 4);
        }
    }

    /// One pack slot of 11 bindings: the counter walks 1..=11 inside the
    /// single slot, and the pack total replaces the slot's 1.
    #[test]
    fn pack_counts_each_binding_in_common_counter() {
        let slots = vec![normal_slot("сначала"), pack_slot(11), normal_slot("потом")];
        let state = state_from(slots, 3);
        let mut mid = state.clone();
        mid.current_index = 1;
        assert_eq!(display_progress(&mid, Some(0)), (2, 13, 13));
        assert_eq!(display_progress(&mid, Some(3)), (5, 13, 13));
        // Frozen last binding: the pack tops out at its full size.
        assert_eq!(display_progress(&mid, Some(10)), (12, 13, 13));
    }

    /// Pack finished → next card: the session unmounted (step = None) and
    /// the lesson advanced — the counter lands on steps_before + 1 of the
    /// NEXT slot, never skipping or double-counting the pack.
    #[test]
    fn pack_to_next_card_transition() {
        let slots = vec![pack_slot(11), normal_slot("после")];
        let state = state_from(slots, 2);
        let mut after = state.clone();
        after.current_index = 1;
        assert_eq!(display_progress(&after, None), (12, 12, 12));
    }

    /// The split bar (core + phrases) keeps its contract in step units:
    /// the pack's steps land inside the core section, phrases stay 1:1.
    #[test]
    fn pack_inside_core_with_phrase_tail() {
        let slots = vec![normal_slot("core1"), pack_slot(11), normal_slot("фраза")];
        // LessonState::core_count counts SLOTS (2 core slots + 1 phrase).
        let state = state_from(slots, 2);
        let mut mid = state.clone();
        mid.current_index = 1;
        let (current, total, core) = display_progress(&mid, Some(5));
        assert_eq!(current, 7);
        assert_eq!(total, 13);
        assert_eq!(core, 12);
        // The LessonProgress label contract: core_done/phrase_done split.
        let core_done = current.min(core);
        let phrase_done = current.saturating_sub(core);
        assert_eq!((core_done, phrase_done), (7, 0));
    }

    /// A degenerate empty pack (unreachable in production — the view
    /// generator degrades it to Normal) must not zero the counter.
    #[test]
    fn empty_pack_still_counts_one_step() {
        let slots = vec![pack_slot(0), normal_slot("рядом")];
        let state = state_from(slots, 2);
        let mut mid = state.clone();
        mid.current_index = 0;
        assert_eq!(display_progress(&mid, None), (1, 2, 2));
    }

    /// The advance transition tick: the container has already moved
    /// `current_index` onto the NEXT card while the session's cleanup
    /// has not reset the step yet — the stale `Some(k)` must be ignored
    /// (not folded into the next card's position), otherwise the pack's
    /// steps double for one frame.
    #[test]
    fn stale_step_against_advanced_index_is_ignored() {
        let slots = vec![pack_slot(11), normal_slot("после")];
        let state = state_from(slots, 2);
        let mut transition = state.clone();
        transition.current_index = 1;
        assert_eq!(
            display_progress(&transition, Some(10)),
            (12, 12, 12),
            "a step published by the unmounted pack must not leak into the next card"
        );
    }

    /// The step is clamped to the pack's own size: a hypothetically
    /// out-of-range value cannot push `current` past the pack's last
    /// binding.
    #[test]
    fn step_is_clamped_to_pack_size() {
        let slots = vec![pack_slot(4), normal_slot("после")];
        let state = state_from(slots, 2);
        let mut mid = state.clone();
        mid.current_index = 0;
        assert_eq!(display_progress(&mid, Some(99)), (4, 5, 5));
    }
}
