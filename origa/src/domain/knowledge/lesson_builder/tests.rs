use super::interleave::*;
use super::phrases::*;
use super::slots::*;
use super::spacing::*;
use super::*;
use crate::domain::DailyBudget;
use crate::domain::RateMode;
use crate::domain::RatingContext;
use crate::domain::knowledge::LessonCardView;
use crate::domain::knowledge::{GrammarRuleCard, KanjiCard, PhraseCard, VocabularyCard};
use crate::domain::memory::Rating;
use crate::domain::value_objects::Question;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rstest::rstest;

fn vocab_card(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ))
}

fn phrase_card(phrase_id: Ulid) -> Card {
    Card::Phrase(PhraseCard::new_test_with_id(phrase_id))
}

fn make_study_card(card: Card) -> (Ulid, StudyCard) {
    (Ulid::new(), StudyCard::new(card))
}

// --- Tail phrase selection (Slice-3) ---
//
// The phrase index is a process-wide `OnceLock`; only one index can live in
// a test binary. These tests reuse the exact 8-phrase fixture also used by
// `journeys/phrase.rs` and `seed_ready_phrases.rs` so they hold regardless
// of which module wins the initialization race.

static PHRASE_INDEX_INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

fn phrase_id_hello() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
}

fn phrase_id_bye() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HK").expect("valid ULID")
}

fn phrase_id_morning() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HN").expect("valid ULID")
}

fn phrase_id_thanks() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HM").expect("valid ULID")
}

fn phrase_id_particle() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HP").expect("valid ULID")
}

fn phrase_id_extra1() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HQ").expect("valid ULID")
}

fn phrase_id_extra2() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HR").expect("valid ULID")
}

fn phrase_id_w1_a() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HT").expect("valid ULID")
}

fn phrase_id_w1_b() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HV").expect("valid ULID")
}

fn phrase_id_w2_a() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HZ").expect("valid ULID")
}

fn phrase_id_w2_b() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03J0").expect("valid ULID")
}

fn phrase_id_hello_anchor() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HW").expect("valid ULID")
}

fn phrase_id_bye_anchor() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HX").expect("valid ULID")
}

fn phrase_id_morning_anchor() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HY").expect("valid ULID")
}

fn ensure_test_phrase_index() {
    PHRASE_INDEX_INIT.get_or_init(|| {
            if crate::dictionary::phrase::is_phrases_loaded() {
                return;
            }
            // Anchored-phrase entries carry one deliberately unknown token
            // ("x1"…"x7") so they are NEVER tail-eligible: they can only
            // enter a lesson through the anchored (interleaved) path.
            let index_json = r#"{"v":1,"h":"test","total":15,"phrases":[
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","t":["test","morning"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB","01G00000000000000024000000"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","t":["test","thanks"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HP","t":["test","は"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HQ","t":["hello","extra1"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HR","t":["hello","extra2"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HS","t":["alpha","beta"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HT","t":["w1","x1"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HV","t":["w1","x2"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HW","t":["hello","x3"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HX","t":["bye","x4"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HY","t":["morning","x5"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HZ","t":["w2","x6"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03J0","t":["w2","x7"],"c":0}
            ]}"#;
            crate::dictionary::phrase::init_phrase_index(index_json)
                .expect("Failed to init test phrase index");
        });
}

fn full_known_set() -> HashSet<String> {
    ["test", "hello", "bye", "morning", "thanks"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn phrase_tail_eligible_true_when_all_tokens_known() {
    ensure_test_phrase_index();

    let known = full_known_set();
    assert!(phrase_tail_eligible(&phrase_id_hello(), &known));
    assert!(phrase_tail_eligible(&phrase_id_bye(), &known));
}

#[test]
fn phrase_tail_eligible_false_when_token_missing() {
    ensure_test_phrase_index();

    let partial: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
    assert!(!phrase_tail_eligible(&phrase_id_bye(), &partial));
}

#[test]
fn phrase_tail_eligible_false_missing_index_entry() {
    ensure_test_phrase_index();

    let known = full_known_set();
    assert!(!phrase_tail_eligible(&Ulid::new(), &known));
}

// --- Prove-It test suite (Slice-1, RED) ---
//
// These tests pin the new tail-eligibility and interleaving contracts
// introduced to fix the post-PR-#188 regression. They drive the Slice-2
// fix and must remain green afterwards. The contract:
//   * Tail eligibility ignores grammatical particles and draws the
//     known-pool from the ENTIRE knowledge_set (not just lesson cards).
//   * Tail per-word cap is 1 (down from 2).
//   * Interleaved anchor set is `!is_known_card()` — high-difficulty
//     vocab is intentionally included.
//   * There is no global interleaved cap (only per-word cap).

#[test]
fn phrase_tail_eligible_ignores_grammatical_particle() {
    ensure_test_phrase_index();

    let known: HashSet<String> = ["test"].iter().map(|s| s.to_string()).collect();
    assert!(
        phrase_tail_eligible(&phrase_id_particle(), &known),
        "particle token は must be ignored when judging tail eligibility"
    );
}

#[test]
fn phrase_tail_eligible_rejects_unknown_non_particle_token() {
    ensure_test_phrase_index();

    let known: HashSet<String> = ["hello"].iter().map(|s| s.to_string()).collect();
    assert!(
        !phrase_tail_eligible(&phrase_id_extra1(), &known),
        "unknown non-particle token must disqualify the phrase"
    );
}

#[test]
fn phrase_tail_eligible_all_known_vocab_eligible() {
    ensure_test_phrase_index();

    let known: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
    assert!(
        phrase_tail_eligible(&phrase_id_hello(), &known),
        "phrase whose tokens are all known must be eligible"
    );
}

/// Tail eligibility draws its known-pool from the ENTIRE knowledge_set, not
/// just the lesson cards. A phrase anchored to vocab that lives in the
/// knowledge_set but is NOT in the current lesson's core must still be
/// tail-eligible. This is the core regression fix.
#[test]
fn phrase_tail_eligible_uses_entire_knowledge_set() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    let test_sc = ks.create_card(vocab_card("test")).expect("create test");
    ks.mark_card_as_known(*test_sc.card_id())
        .expect("mark test known");
    let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
    ks.mark_card_as_known(*hello_sc.card_id())
        .expect("mark hello known");
    ks.create_card(Card::Phrase(
        PhraseCard::new_test_with_id(phrase_id_hello()),
    ))
    .expect("create phrase");

    let lesson = LessonData {
        cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
        core_count: 1,
    };

    let mut budget = 5;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
    let phrases = lesson_phrase_ids(&result);

    assert!(
        phrases.contains(&phrase_id_hello()),
        "phrase anchored to known vocab outside the lesson core must still enter the lesson"
    );
}

#[test]
fn tail_phrases_cap_one_per_word() {
    ensure_test_phrase_index();

    let owned = make_phrase_study_cards(&[
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ]);
    let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

    let known = full_known_set();
    let empty_used = HashSet::new();
    let mut selection = tail_selection(&all_cards, &known, &empty_used);
    let result = collect_phrase_cards(&mut selection);

    assert_eq!(
        result.len(),
        1,
        "Tail phrases sharing a word should be capped at 1 (MAX_PHRASES_PER_WORD_IN_TAIL=1), got {}",
        result.len()
    );
}

/// The no-anchor section ignores `phrase_new_budget`: a depleted budget
/// must NOT starve no-anchor phrases, whose count is bounded per lesson
/// by `TAIL_PHRASE_PER_LESSON` (plus the per-word cap) instead. With
/// `budget = 0` the no-anchor phrases still appear.
#[test]
fn no_anchor_phrases_ignore_new_phrase_budget() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    for word in ["test", "hello", "bye", "morning", "thanks"] {
        let sc = ks.create_card(vocab_card(word)).expect("create vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ] {
        ks.create_card(Card::Phrase(PhraseCard::new_test_with_id(pid)))
            .expect("create phrase");
    }

    // Lesson core holds a word no phrase anchors to, so every selected
    // phrase is no-anchor. Per-word cap (`MAX_PHRASES_PER_WORD_IN_TAIL=1`)
    // admits exactly one (all four share the "test" token).
    let lesson = LessonData {
        cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
        core_count: 1,
    };

    let mut zero_budget = 0;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut zero_budget);
    let count = lesson_phrase_ids(&result).len();

    assert_eq!(
        zero_budget, 0,
        "no-anchor phrases must not decrement phrase_new_budget"
    );
    assert!(
        count >= 1,
        "no-anchor phrases must appear even with budget=0, got {count}"
    );
    assert!(
        count <= TAIL_PHRASE_PER_LESSON,
        "no-anchor phrases must respect TAIL_PHRASE_PER_LESSON={TAIL_PHRASE_PER_LESSON}, got {count}"
    );
}

// Note: `tail_phrases_preserve_due_then_new_order` previously pinned that
// due phrases preceded new phrases inside the dedicated tail zone. That zone
// is gone (Slice 3 merges all phrases into constraint-aware placement), so
// the due/new ordering no longer exists as a separate contract — phrase
// order is now driven by the anchor-first-showing constraint, not by due/new
// scheduling. The test was removed together with the tail zone.

/// High-difficulty vocab is now a valid interleaving target (the original
/// purpose of interleaving): a phrase anchored to an HD word must appear.
#[test]
fn interleaved_phrases_target_high_difficulty() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    let bye_sc = ks.create_card(vocab_card("bye")).expect("create bye");
    for word in ["test", "hello"] {
        ks.create_card(vocab_card(word))
            .expect("create known vocab");
    }
    ks.create_card(phrase_card(phrase_id_bye()))
        .expect("create phrase");

    for _ in 0..3 {
        ks.rate_card(
            *bye_sc.card_id(),
            Rating::Again,
            RateMode::ShortTerm,
            RatingContext::Explicit,
        )
        .expect("rate bye");
    }

    let bye_id = *bye_sc.card_id();
    assert!(
        ks.get_card(bye_id).unwrap().memory().is_high_difficulty(),
        "fixture sanity: bye should be high difficulty"
    );

    let lesson = LessonData {
        cards: vec![(bye_id, lesson_card_for(vocab_card("bye")))],
        core_count: 1,
    };

    let mut budget = 5;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
    let phrases = lesson_phrase_ids(&result);

    assert!(
        phrases.contains(&phrase_id_bye()),
        "high-difficulty anchor must receive an interleaved phrase"
    );
}

/// No global interleaved cap exists: two anchor words each yield
/// `INTERLEAVED_PHRASES_PER_WORD` phrases, summing to 4 — proving the
/// absence of a hidden total ceiling.
#[test]
fn interleaved_phrases_no_total_cap_multi_anchor() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    let test_sc = ks.create_card(vocab_card("test")).expect("create test");
    let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_extra1(),
        phrase_id_extra2(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let test_id = *test_sc.card_id();
    let hello_id = *hello_sc.card_id();
    let lesson = LessonData {
        cards: vec![
            (test_id, lesson_card_for(vocab_card("test"))),
            (hello_id, lesson_card_for(vocab_card("hello"))),
        ],
        core_count: 2,
    };

    let mut budget = 10;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
    let phrases = lesson_phrase_ids(&result);

    assert!(
        phrases.len() >= INTERLEAVED_PHRASES_PER_WORD * 2,
        "two anchors × {} phrases each (no total cap) should yield ≥ {}, got {}",
        INTERLEAVED_PHRASES_PER_WORD,
        INTERLEAVED_PHRASES_PER_WORD * 2,
        phrases.len()
    );
}

fn make_phrase_study_cards(phrase_ids: &[Ulid]) -> Vec<(Ulid, StudyCard)> {
    phrase_ids
        .iter()
        .map(|pid| make_study_card(phrase_card(*pid)))
        .collect()
}

fn selected_phrase_ids(cards: &[(&Ulid, &StudyCard)]) -> Vec<Ulid> {
    cards
        .iter()
        .filter_map(|(_, sc)| match sc.card() {
            Card::Phrase(p) => Some(*p.phrase_id()),
            _ => None,
        })
        .collect()
}

fn empty_ulid_set() -> &'static HashSet<Ulid> {
    static EMPTY: std::sync::OnceLock<HashSet<Ulid>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(HashSet::new)
}

fn tail_selection<'a>(
    all_cards: &'a [(&'a Ulid, &'a StudyCard)],
    known: &'a HashSet<String>,
    used: &'a HashSet<Ulid>,
) -> TailPhraseSelection<'a> {
    TailPhraseSelection {
        all_cards,
        excluded_card_ids: empty_ulid_set(),
        used_phrase_ids: used,
        known_pool: known,
    }
}

#[test]
fn tail_phrases_contain_only_known_words() {
    ensure_test_phrase_index();

    // One eligible phrase plus one unknown phrase id. The eligible phrase
    // has all-known tokens; the unknown phrase id is absent from the index,
    // so `phrase_tail_eligible` rejects it via `get_index_entry` returning
    // None.
    let owned = make_phrase_study_cards(&[phrase_id_hello(), Ulid::new()]);
    let unknown_phrase = match owned[1].1.card() {
        Card::Phrase(p) => *p.phrase_id(),
        _ => unreachable!("second card is a phrase card"),
    };
    let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

    let known = full_known_set();
    let empty_used = HashSet::new();
    let mut selection = tail_selection(&all_cards, &known, &empty_used);
    let result = collect_phrase_cards(&mut selection);

    let selected = selected_phrase_ids(&result);
    assert!(
        selected.contains(&phrase_id_hello()),
        "phrase with all-known tokens should be selected"
    );
    assert!(
        !selected.contains(&unknown_phrase),
        "phrase whose phrase_id is absent from the index must be excluded"
    );
}

#[test]
fn tail_phrases_exclude_used_phrase_ids() {
    ensure_test_phrase_index();

    let owned = make_phrase_study_cards(&[phrase_id_hello(), phrase_id_bye()]);
    let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

    let known = full_known_set();
    let mut used = HashSet::new();
    used.insert(phrase_id_hello());

    let mut selection = tail_selection(&all_cards, &known, &used);
    let result = collect_phrase_cards(&mut selection);

    let selected = selected_phrase_ids(&result);
    assert!(
        !selected.contains(&phrase_id_hello()),
        "phrase already used by interleaving must not reappear in the tail"
    );
    assert!(selected.contains(&phrase_id_bye()));
}

#[test]
fn init_phrase_index_loads_entries() {
    // The CDN loader (`init_phrase_index_from_cdn`) is process-wide and
    // mutually exclusive with the 8-phrase fixture used across the lib test
    // binary, so loading the real 156k-entry index here would win the
    // OnceLock race and break fixture-based tests. We instead verify the
    // helper is exported and callable, and assert the index contract
    // (entries become retrievable) via the safe fixture path.
    let _helper: fn() = crate::use_cases::init_phrase_index_from_cdn;

    ensure_test_phrase_index();

    assert!(
        crate::dictionary::phrase::is_phrases_loaded(),
        "phrase index should be loaded after init"
    );
    assert!(
        crate::dictionary::phrase::iter_index_entries().is_some(),
        "iter_index_entries should yield entries once the index is loaded"
    );
    assert!(
        crate::dictionary::phrase::get_phrases_by_token("test")
            .iter()
            .any(|e| !e.tokens().is_empty()),
        "get_phrases_by_token should resolve known fixture tokens to entries"
    );
}

// --- Interleaving algorithm (H1) ---
//
// These tests pin the invariants of the phrase-interleaving pipeline. They
// intentionally span three levels: the pure layout primitive
// `interleave_with_gap`, the mid-level orchestrator `add_phrases`
// / `collect_interleaved_phrases_for_word`, and the full `cards_to_lesson`
// pipeline. Lower levels isolate a single invariant; the pipeline tests
// guard the integration of the shared budget.

fn lesson_card_for(card: Card) -> LessonCard {
    LessonCard::new(Ulid::new(), LessonCardView::Normal(card), false)
}

// --- interleave_core_by_type (Slice 2) ---
//
// Vocab acts as the separator spine; kanji/grammar are dealt round-robin
// into the V+1 gaps between vocab cards. The longest kanji-only run is
// therefore bounded by ⌈K/(V+1)⌉ deterministically (counts only).

fn slot(card: Card) -> (Ulid, LessonCard) {
    let id = Ulid::new();
    (id, LessonCard::new(id, LessonCardView::Normal(card), false))
}

fn build_core_with(vocab_n: usize, kanji_n: usize, grammar_n: usize) -> LessonData {
    let mut cards: Vec<(Ulid, LessonCard)> = Vec::new();
    for i in 0..vocab_n {
        cards.push(slot(vocab_card(&format!("v{i}"))));
    }
    for i in 0..kanji_n {
        cards.push(slot(Card::Kanji(KanjiCard::new_test(format!("k{i}")))));
    }
    for _ in 0..grammar_n {
        cards.push(slot(Card::Grammar(GrammarRuleCard::new_test())));
    }
    let core_count = cards.len();
    LessonData { cards, core_count }
}

fn longest_type_run(cards: &[(Ulid, LessonCard)], target: CardType) -> usize {
    let mut best = 0usize;
    let mut current = 0usize;
    for (_, lc) in cards {
        if CardType::from(lc.card()) == target {
            current += 1;
            best = best.max(current);
        } else {
            current = 0;
        }
    }
    best
}

#[rstest]
#[case::vocab_dominant(8, 5, 0)]
#[case::kanji_heavy(5, 10, 0)]
#[case::with_grammar(6, 7, 3)]
#[case::single_kanji(10, 1, 0)]
#[case::kanji_slightly_more_than_vocab(4, 6, 0)]
fn interleave_core_bounds_kanji_run_within_ceiling(
    #[case] vocab_n: usize,
    #[case] kanji_n: usize,
    #[case] grammar_n: usize,
) {
    let lesson = build_core_with(vocab_n, kanji_n, grammar_n);
    let result = interleave_core_by_type(lesson);

    let ceiling = (kanji_n + vocab_n) / (vocab_n + 1);
    let run = longest_type_run(&result.cards, CardType::Kanji);
    assert!(
        run <= ceiling,
        "V={vocab_n},K={kanji_n},G={grammar_n}: max kanji run {run} exceeds ceiling ⌈K/(V+1)⌉={ceiling}"
    );
}

#[test]
fn interleave_core_grammar_run_also_bounded() {
    let lesson = build_core_with(6, 2, 9);
    let result = interleave_core_by_type(lesson);

    let ceiling = (9 + 6) / (6 + 1);
    let run = longest_type_run(&result.cards, CardType::Grammar);
    assert!(
        run <= ceiling,
        "grammar run {run} exceeds ceiling ⌈G/(V+1)⌉={ceiling}"
    );
}

#[test]
fn interleave_core_preserves_card_set() {
    let lesson = build_core_with(5, 4, 2);
    let before: HashSet<Ulid> = lesson.cards.iter().map(|(id, _)| *id).collect();
    let result = interleave_core_by_type(lesson);
    let after: HashSet<Ulid> = result.cards.iter().map(|(id, _)| *id).collect();

    assert_eq!(
        before, after,
        "interleaving must not add, drop or duplicate cards"
    );
    assert_eq!(result.core_count, before.len());
}

#[test]
fn interleave_core_empty_is_noop() {
    let lesson = LessonData {
        cards: vec![],
        core_count: 0,
    };
    let result = interleave_core_by_type(lesson);
    assert!(result.cards.is_empty());
    assert_eq!(result.core_count, 0);
}

#[test]
fn interleave_core_single_card_is_noop() {
    let lesson = build_core_with(1, 0, 0);
    let only_id = lesson.cards[0].0;
    let result = interleave_core_by_type(lesson);
    assert_eq!(result.cards.len(), 1);
    assert_eq!(result.cards[0].0, only_id);
}

#[test]
fn interleave_core_no_vocab_leaves_layout_untouched() {
    // With no separator spine the core cannot be spread; the original
    // order must be preserved byte-for-byte.
    let lesson = build_core_with(0, 6, 0);
    let original_order: Vec<Ulid> = lesson.cards.iter().map(|(id, _)| *id).collect();
    let result = interleave_core_by_type(lesson);
    let result_order: Vec<Ulid> = result.cards.iter().map(|(id, _)| *id).collect();

    assert_eq!(
        original_order, result_order,
        "an all-kanji core must be left untouched (no separator available)"
    );
}

fn lesson_phrase_id(lc: &LessonCard) -> Option<Ulid> {
    match lc.card() {
        Card::Phrase(p) => Some(*p.phrase_id()),
        _ => None,
    }
}

fn phrase_cards_map(owned: &[(Ulid, StudyCard)]) -> HashMap<Ulid, (&Ulid, &StudyCard)> {
    owned
        .iter()
        .filter_map(|(id, sc)| match sc.card() {
            Card::Phrase(p) => Some((*p.phrase_id(), (id, sc))),
            _ => None,
        })
        .collect()
}

fn lesson_phrase_ids(lesson: &LessonData) -> HashSet<Ulid> {
    lesson
        .cards
        .iter()
        .filter_map(|(_, lc)| lesson_phrase_id(lc))
        .collect()
}

/// A phrase anchored to a word must land strictly after it, with at least
/// `gap` intervening cards when the core is long enough to honour the gap.
#[test]
fn interleaved_phrases_placed_after_word_with_gap() {
    let word_id = Ulid::new();
    let phrase_card_id = Ulid::new();

    let mut core_cards: Vec<(Ulid, LessonCard)> = (0..5)
        .map(|_| (Ulid::new(), lesson_card_for(vocab_card("filler"))))
        .collect();
    core_cards[2] = (word_id, lesson_card_for(vocab_card("anchor")));

    let assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = [(
        word_id,
        vec![(phrase_card_id, lesson_card_for(phrase_card(Ulid::new())))],
    )]
    .into_iter()
    .collect();

    let result = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);

    let word_pos = result
        .iter()
        .position(|(id, _)| *id == word_id)
        .expect("anchor word present in output");
    let phrase_pos = result
        .iter()
        .position(|(id, _)| *id == phrase_card_id)
        .expect("interleaved phrase present in output");

    assert!(
        phrase_pos > word_pos,
        "phrase must follow its anchor word: {phrase_pos} <= {word_pos}"
    );
    assert!(
        phrase_pos > word_pos + INTERLEAVING_GAP,
        "core had room for the gap: phrase_pos {phrase_pos} should be > {}",
        word_pos + INTERLEAVING_GAP
    );
}

/// No single anchor word may pull more than `INTERLEAVED_PHRASES_PER_WORD`
/// phrases into the core, even when many eligible phrases share its token.
#[test]
fn interleaved_phrases_max_two_per_word() {
    ensure_test_phrase_index();

    let owned = make_phrase_study_cards(&[
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ]);
    let map = phrase_cards_map(&owned);

    let in_lesson = HashSet::new();
    let mut used = HashSet::new();
    let mut budget = 10;

    let picked =
        collect_interleaved_phrases_for_word("test", &map, &in_lesson, &mut used, &mut budget);

    assert!(
        picked.len() <= INTERLEAVED_PHRASES_PER_WORD,
        "per-word cap violated: {} > {INTERLEAVED_PHRASES_PER_WORD}",
        picked.len()
    );
    assert_eq!(
        picked.len(),
        INTERLEAVED_PHRASES_PER_WORD,
        "four eligible phrases with a deep budget should fill the cap exactly"
    );
}

/// Interleaving falls back to known vocab ONLY when no non-known (new /
/// in-progress / high-difficulty) anchor yields a phrase. The fallback
/// exists to keep phrases flowing on a fully-mastered lesson — the target
/// filter `!is_known_card()` covers new, in-progress and high-difficulty
/// vocab alike (see `interleaved_phrases_target_high_difficulty`), so this
/// test exercises the residual branch where every core vocab is known.
#[test]
fn interleaved_phrases_fallback_to_known() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
    let test_sc = ks.create_card(vocab_card("test")).expect("create test");
    for pid in [phrase_id_hello(), phrase_id_bye()] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    ks.mark_card_as_known(*hello_sc.card_id())
        .expect("mark hello known");
    ks.mark_card_as_known(*test_sc.card_id())
        .expect("mark test known");

    let hello_id = *hello_sc.card_id();
    let test_id = *test_sc.card_id();
    assert!(ks.get_card(hello_id).unwrap().memory().is_known_card());
    assert!(ks.get_card(test_id).unwrap().memory().is_known_card());

    let lesson = LessonData {
        cards: vec![
            (hello_id, lesson_card_for(vocab_card("hello"))),
            (test_id, lesson_card_for(vocab_card("test"))),
        ],
        core_count: 2,
    };

    let mut budget = 5;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
    let phrases = lesson_phrase_ids(&result);

    assert!(
        phrases.contains(&phrase_id_hello()),
        "fallback should anchor a phrase to a known word when no target vocab yields phrases"
    );
}

/// A phrase selected for the lesson must never appear more than once: the
/// shared `used_phrase_ids` set dedupes the former interleaved/tail pools,
/// which are now merged into a single placement pass.
#[test]
fn phrases_never_duplicated_in_lesson() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    ks.create_card(vocab_card("hello")).expect("create hello");
    for word in ["test", "bye", "morning", "thanks"] {
        let sc = ks
            .create_card(vocab_card(word))
            .expect("create known vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let lesson = ks.cards_to_lesson(
        DailyBudget::with_daily_cards(1),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let mut all_phrase_ids: Vec<Ulid> = lesson
        .cards
        .iter()
        .filter_map(|(_, lc)| lesson_phrase_id(lc))
        .collect();
    let total = all_phrase_ids.len();
    all_phrase_ids.sort();
    all_phrase_ids.dedup();
    assert!(
        !all_phrase_ids.is_empty(),
        "scenario should place at least one phrase"
    );
    assert_eq!(
        all_phrase_ids.len(),
        total,
        "no phrase id may appear more than once in the lesson"
    );
}

/// After merging, every phrase sits inside the core section and there is no
/// dedicated tail zone: `core_count` equals the lesson length.
#[test]
fn phrases_merged_into_core_no_tail_zone() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    ks.create_card(vocab_card("hello")).expect("create hello");
    for word in ["test", "bye", "morning", "thanks"] {
        let sc = ks
            .create_card(vocab_card(word))
            .expect("create known vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let lesson = ks.cards_to_lesson(
        DailyBudget::with_daily_cards(1),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let phrases_in_lesson: usize = lesson
        .cards
        .iter()
        .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
        .count();
    assert!(
        phrases_in_lesson >= 1,
        "scenario should place at least one phrase"
    );
    assert_eq!(
        lesson.core_count,
        lesson.cards.len(),
        "all cards (phrases included) must be part of the core — no dedicated tail zone"
    );
}

/// On a lesson too short to honour the gap, the phrase is flushed at the
/// end. The only inviolable ordering rule — phrase after its anchor — holds.
#[test]
fn interleaved_phrases_small_lesson_degrades_gap() {
    let word_id = Ulid::new();
    let phrase_card_id = Ulid::new();

    let core_cards: Vec<(Ulid, LessonCard)> =
        [(word_id, lesson_card_for(vocab_card("solo")))].to_vec();
    let assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = [(
        word_id,
        vec![(phrase_card_id, lesson_card_for(phrase_card(Ulid::new())))],
    )]
    .into_iter()
    .collect();

    let result = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);

    let word_pos = result
        .iter()
        .position(|(id, _)| *id == word_id)
        .expect("anchor present");
    let phrase_pos = result
        .iter()
        .position(|(id, _)| *id == phrase_card_id)
        .expect("flushed phrase present");

    assert_eq!(result.len(), 2);
    assert!(phrase_pos > word_pos);
    assert!(
        phrase_pos < word_pos + INTERLEAVING_GAP + 1,
        "gap should degrade on a tiny core, but ordering must survive"
    );
}

/// The new-phrase allowance bounds only NEW ANCHORED phrases; NEW
/// no-anchor phrases are additive (capped per lesson by
/// `TAIL_PHRASE_PER_LESSON`). The combined new-phrase count therefore may
/// reach `allowance + TAIL_PHRASE_PER_LESSON` but must not exceed it. Due
/// phrases are free on both sides.
#[test]
fn new_phrases_respect_anchored_plus_tail_combined_ceiling() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    ks.create_card(vocab_card("hello")).expect("create hello");
    for word in ["test", "bye", "morning", "thanks"] {
        let sc = ks
            .create_card(vocab_card(word))
            .expect("create known vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let budget = DailyBudget::with_daily_cards(1);
    let allowance = budget.new_phrases_per_lesson();
    assert_eq!(allowance, 2, "fixture sanity: PHRASES_PER_NEW_CARD=2");

    let lesson = ks.cards_to_lesson(
        budget,
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let new_phrases_in_lesson = lesson
        .cards
        .iter()
        .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
        .filter(|(id, _)| ks.get_card(*id).is_some_and(|sc| sc.memory().is_new()))
        .count();

    let combined_ceiling = allowance + TAIL_PHRASE_PER_LESSON;
    assert!(
        new_phrases_in_lesson <= combined_ceiling,
        "new phrases (anchored ≤ allowance + no-anchor ≤ TAIL_PHRASE_PER_LESSON) must respect \
             the combined ceiling: {new_phrases_in_lesson} > {combined_ceiling}"
    );
    assert!(
        new_phrases_in_lesson >= 1,
        "scenario should place at least one new phrase"
    );
}

/// Counts NEW phrase cards in the lesson that are ANCHORED — their index
/// tokens intersect the lesson's core vocabulary words. Only anchored
/// phrases consume the new-phrase allowance; no-anchor ("tail") phrases
/// do not, so tests of the allowance must filter with this predicate
/// (a plain "any new phrase" count would also pass under the old code,
/// which kept seeding tail phrases).
fn new_anchored_phrase_count(ks: &KnowledgeSet, lesson: &LessonData) -> usize {
    let core_vocab_words: HashSet<String> = lesson
        .cards
        .iter()
        .filter_map(|(_, lc)| match lc.card() {
            Card::Vocabulary(v) => Some(v.word().text().to_string()),
            _ => None,
        })
        .collect();

    lesson
        .cards
        .iter()
        .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
        .filter(|(id, _)| ks.get_card(*id).is_some_and(|sc| sc.memory().is_new()))
        .filter(|(_, lc)| {
            lesson_phrase_id(lc)
                .and_then(|pid| crate::dictionary::phrase::get_index_entry(&pid))
                .is_some_and(|entry| entry.tokens().iter().any(|t| core_vocab_words.contains(t)))
        })
        .count()
}

/// The per-lesson allowance preserves the historical first-lesson
/// ceiling: with a sufficient pool the FIRST lesson of the day may
/// interleave `daily × 2` new anchored phrases (parity with the former
/// daily budget's unspent first-lesson value).
#[test]
fn first_lesson_new_anchored_phrases_reach_daily_double_bound() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    for word in ["w1", "w2"] {
        ks.create_card(vocab_card(word))
            .expect("create anchor vocab");
    }
    for pid in [
        phrase_id_w1_a(),
        phrase_id_w1_b(),
        phrase_id_w2_a(),
        phrase_id_w2_b(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let budget = DailyBudget::with_daily_cards(2);
    let lesson = ks.cards_to_lesson(
        budget,
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let new_anchored = new_anchored_phrase_count(&ks, &lesson);
    assert_eq!(
        new_anchored, 4,
        "first lesson must reach the daily×2 allowance bound (2 words × 2 phrases)"
    );
}

/// The new-anchored-phrase allowance is per LESSON, not per day: after a
/// morning lesson consumes the full historical daily budget (its phrases
/// rated through `rate_card` — the only path incrementing the studied
/// counter), a second lesson the same day still receives the full
/// allowance. Under the former daily budget the second lesson would
/// receive no new anchored phrases at all.
#[test]
fn second_lesson_same_day_receives_fresh_phrase_allowance() {
    use crate::domain::memory::Rating;

    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    // w1 = lesson-1 new anchor; hello/bye/morning = due-known anchors
    // (mark_card_as_known: stability 22, next_review in the past) that
    // carry the lesson-2 anchored phrases.
    ks.create_card(vocab_card("w1")).expect("create w1");
    for word in ["hello", "bye", "morning"] {
        let sc = ks
            .create_card(vocab_card(word))
            .expect("create known vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }
    for pid in [
        phrase_id_w1_a(),
        phrase_id_w1_b(),
        phrase_id_hello_anchor(),
        phrase_id_bye_anchor(),
        phrase_id_morning_anchor(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let budget = DailyBudget::with_daily_cards(1);
    let lesson1 = ks.cards_to_lesson(
        budget,
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let new_anchored_1 = new_anchored_phrase_count(&ks, &lesson1);
    assert_eq!(
        new_anchored_1, 2,
        "lesson 1 must consume the full allowance (daily×2 = 2)"
    );

    let lesson1_new_phrase_slots: Vec<Ulid> = lesson1
        .cards
        .iter()
        .filter(|(id, lc)| {
            lesson_phrase_id(lc).is_some()
                && ks.get_card(*id).is_some_and(|sc| sc.memory().is_new())
        })
        .map(|(id, _)| *id)
        .collect();
    for slot_id in lesson1_new_phrase_slots {
        ks.rate_card(
            slot_id,
            Rating::Good,
            RateMode::StandardLesson,
            RatingContext::Explicit,
        )
        .expect("rate phrase");
    }

    // Fixture sanity: the studied counter equals the FULL historical
    // daily budget (daily×2 = 2), so the former per-day implementation
    // would grant the second lesson an allowance of exactly zero.
    assert_eq!(
        ks.phrase_cards_studied_today(),
        2,
        "fixture sanity: historical daily phrase budget fully spent"
    );

    let lesson2 = ks.cards_to_lesson(
        budget,
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let lesson2_vocab: HashSet<String> = lesson2
        .cards
        .iter()
        .filter_map(|(_, lc)| match lc.card() {
            Card::Vocabulary(v) => Some(v.word().text().to_string()),
            _ => None,
        })
        .collect();
    for anchor in ["hello", "bye", "morning"] {
        assert!(
            lesson2_vocab.contains(anchor),
            "lesson 2 core must contain the due-known anchor {anchor}"
        );
    }

    let new_anchored_2 = new_anchored_phrase_count(&ks, &lesson2);
    // Deterministic fixture: 3 due-known anchors × 1 phrase each, no
    // per-word contention, allowance 2 — the second lesson must consume
    // the FULL allowance, not merely "some" phrases.
    assert_eq!(
        new_anchored_2, 2,
        "second lesson of the day must receive the full per-lesson \
             allowance (the former daily budget would leave none)"
    );
}

/// Due phrases enter the interleaved section for free: they do not decrement
/// `phrase_new_budget`, which stays reserved for new phrases.
#[test]
fn interleaved_phrases_due_do_not_consume_new_budget() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    let hello_sc = ks
        .create_card(phrase_card(phrase_id_hello()))
        .expect("create hello");
    for pid in [phrase_id_bye(), phrase_id_morning(), phrase_id_thanks()] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    // mark_card_as_known schedules next_review in the past, so the phrase
    // becomes due (and not new) without going through a timed review cycle.
    ks.mark_card_as_known(*hello_sc.card_id())
        .expect("mark hello due");
    let hello_card = ks.get_card(*hello_sc.card_id()).unwrap();
    assert!(hello_card.memory().is_due());
    assert!(!hello_card.memory().is_new());

    let owned: Vec<(Ulid, StudyCard)> = ks
        .study_cards()
        .iter()
        .map(|(id, sc)| (*id, sc.clone()))
        .collect();
    let map = phrase_cards_map(&owned);

    let in_lesson = HashSet::new();
    let mut used = HashSet::new();
    let mut budget = 2;
    let initial_budget = budget;

    let picked =
        collect_interleaved_phrases_for_word("test", &map, &in_lesson, &mut used, &mut budget);

    let picked_due_hello = picked.iter().any(|(id, _)| *id == *hello_sc.card_id());
    assert!(
        picked_due_hello,
        "due phrase must be picked for free by the due pass"
    );

    let new_picked = picked.iter().filter(|(_, sc)| sc.memory().is_new()).count();
    assert_eq!(
        budget,
        initial_budget - new_picked,
        "only new phrases consume budget; the due phrase must be free"
    );
}

// --- Constraint-aware placement (Slice 3) ---
//
// A phrase that references a lesson-vocab word must appear AFTER the first
// showing of every such word, otherwise it leaks the answer into the word's
// standalone FSRS rating. These tests pin that invariant end-to-end.

fn first_showing_positions(lesson: &LessonData) -> HashMap<Ulid, usize> {
    let mut positions: HashMap<Ulid, usize> = HashMap::new();
    for (i, (slot_id, _)) in lesson.cards.iter().enumerate() {
        positions.entry(*slot_id).or_insert(i);
    }
    positions
}

/// Asserts every phrase in `lesson` follows the first showing of each of its
/// anchor words that are present in `anchor_vocab` (word -> slot id).
fn assert_phrases_follow_anchors(
    lesson: &LessonData,
    anchor_vocab: &[(&Ulid, &str)],
    positions: &HashMap<Ulid, usize>,
) {
    let word_to_slot: HashMap<&str, Ulid> = anchor_vocab.iter().map(|(id, w)| (*w, **id)).collect();
    for (phrase_idx, (_, lc)) in lesson.cards.iter().enumerate() {
        let Some(phrase_id) = lesson_phrase_id(lc) else {
            continue;
        };
        let tokens: Vec<String> = crate::dictionary::phrase::get_index_entry(&phrase_id)
            .map(|e| e.tokens().to_vec())
            .unwrap_or_default();
        for token in tokens {
            if let Some(anchor_slot) = word_to_slot.get(token.as_str()) {
                let anchor_pos = positions.get(anchor_slot).copied().unwrap_or(usize::MAX);
                assert!(
                    phrase_idx > anchor_pos,
                    "phrase {phrase_id} at index {phrase_idx} must follow anchor '{token}' \
                         (slot {anchor_slot}) first shown at {anchor_pos}"
                );
            }
        }
    }
}

fn build_vocab_core(words: &[&str]) -> Vec<(Ulid, LessonCard)> {
    words
        .iter()
        .map(|w| {
            let slot_id = Ulid::new();
            (slot_id, lesson_card_for(vocab_card(w)))
        })
        .collect()
}

fn vocab_slots_of(cards: &[(Ulid, LessonCard)]) -> Vec<(Ulid, String)> {
    cards
        .iter()
        .filter_map(|(id, lc)| match lc.card() {
            Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
            _ => None,
        })
        .collect()
}

#[test]
fn phrase_appears_after_its_anchor_word_first_showing() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    ks.create_card(phrase_card(phrase_id_hello()))
        .expect("create phrase hello"); // tokens [test, hello]
    ks.create_card(phrase_card(phrase_id_bye()))
        .expect("create phrase bye"); // tokens [test, bye]

    let core_cards = build_vocab_core(&["test", "fill1", "hello", "fill2"]);
    let core_vocab = vocab_slots_of(&core_cards);
    let lesson = LessonData {
        cards: core_cards,
        core_count: 4,
    };

    let mut budget = 10;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);

    let phrase_ids = lesson_phrase_ids(&result);
    assert!(
        phrase_ids.contains(&phrase_id_hello()),
        "fixture sanity: phrase hello should be selected"
    );

    let positions = first_showing_positions(&result);
    let anchor_vocab: Vec<(&Ulid, &str)> =
        core_vocab.iter().map(|(id, w)| (id, w.as_str())).collect();
    assert_phrases_follow_anchors(&result, &anchor_vocab, &positions);
}

#[test]
fn constraint_holds_across_many_anchors_and_phrases() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
        phrase_id_extra1(),
        phrase_id_extra2(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let core_cards = build_vocab_core(&[
        "test", "hello", "bye", "morning", "thanks", "fill1", "fill2",
    ]);
    let core_vocab = vocab_slots_of(&core_cards);
    let lesson = LessonData {
        cards: core_cards,
        core_count: 7,
    };

    let mut budget = 20;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);

    let positions = first_showing_positions(&result);
    let anchor_vocab: Vec<(&Ulid, &str)> =
        core_vocab.iter().map(|(id, w)| (id, w.as_str())).collect();
    assert_phrases_follow_anchors(&result, &anchor_vocab, &positions);
}

#[test]
fn anchorless_phrases_are_distributed_not_clustered_at_end() {
    ensure_test_phrase_index();

    // ULID of the fixture phrase whose tokens are ["alpha", "beta"] — both
    // deliberately absent from the lesson core, making the phrase anchorless.
    let phrase_independent = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HS").expect("valid ULID");

    let mut ks = KnowledgeSet::new();
    // "alpha"/"beta" are NOT in the lesson core → this phrase has no anchor
    // and must be distributed rather than dumped at the end.
    ks.create_card(phrase_card(phrase_independent))
        .expect("create independent phrase");
    // a second anchorless phrase for a stronger distribution check
    ks.create_card(phrase_card(phrase_id_extra1()))
        .expect("create extra1 phrase");

    // Mark all tokens known so tail eligibility admits them.
    for word in ["alpha", "beta", "hello", "extra1"] {
        let sc = ks.create_card(vocab_card(word)).expect("create vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }

    // Lesson core deliberately contains none of the phrase tokens.
    let core_cards = build_vocab_core(&["v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8"]);
    let core_len = core_cards.len();
    let lesson = LessonData {
        cards: core_cards,
        core_count: core_len,
    };

    let mut budget = 10;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);

    let phrase_indices: Vec<usize> = result
        .cards
        .iter()
        .enumerate()
        .filter(|(_, (_, lc))| lesson_phrase_id(lc).is_some())
        .map(|(i, _)| i)
        .collect();
    assert!(!phrase_indices.is_empty(), "at least one phrase expected");

    // The last third of the lesson must not contain ALL phrases — that would
    // reproduce the dedicated end zone the merge set out to remove.
    let tail_start = result.cards.len() * 2 / 3;
    let phrases_before_tail = phrase_indices.iter().filter(|&&i| i < tail_start).count();
    assert!(
        phrases_before_tail > 0,
        "anchorless phrases must be distributed: found {phrases_before_tail} before the last third, indices {phrase_indices:?}"
    );
}

// --- deal_by_card_id unit contract (Symptom 1, layout primitive) ---
//
// Pins the spacing invariant of the layout primitive directly, at the
// content level (no phrases masking it), including the lone-multi-show-card
// review-lesson shape that defeated the earlier round-robin deal.

/// Builds content slots from a `(card_id, showing_count)` spec, in view
/// order (primary first). Each showing uses a distinct `LessonCardView`
/// variant so view-order preservation is observable.
fn build_content_spec(spec: &[(Ulid, usize)]) -> Vec<(Ulid, LessonCard)> {
    let mut out = Vec::new();
    for (card_id, count) in spec {
        for j in 0..*count {
            let card = vocab_card(&format!("c{card_id}{j}"));
            let view = if j == 0 {
                LessonCardView::Normal(card)
            } else {
                LessonCardView::Reversed(card)
            };
            out.push((Ulid::new(), LessonCard::new(*card_id, view, false)));
        }
    }
    out
}

fn content_min_gap(content: &[(Ulid, LessonCard)], card_id: Ulid) -> Option<usize> {
    let positions: Vec<usize> = content
        .iter()
        .enumerate()
        .filter(|(_, (_, lc))| lc.card_id() == card_id)
        .map(|(i, _)| i)
        .collect();
    if positions.len() < 2 {
        return None;
    }
    positions.windows(2).map(|w| w[1] - w[0] - 1).min()
}

/// The case that beat round-robin: a single multi-show card (3 showings)
/// surrounded by single-show fillers — a normal review lesson with one
/// hard card. Its copies must still stay `MIN_REPEAT_SPACING` apart.
#[test]
fn deal_spreads_lone_multishow_card() {
    let hard = Ulid::new();
    let mut spec: Vec<(Ulid, usize)> = vec![(hard, 3)];
    for _ in 0..14 {
        spec.push((Ulid::new(), 1));
    }
    let content = build_content_spec(&spec);

    let dealt = deal_by_card_id(content);

    let gap = content_min_gap(&dealt, hard).expect("hard has 3 showings");
    assert!(
        gap >= MIN_REPEAT_SPACING,
        "lone multi-show card must stay >= {MIN_REPEAT_SPACING} apart, got gap={gap}"
    );
}

/// Many multi-show cards (kanji-heavy mix): every card_id with >= 2
/// showings keeps `MIN_REPEAT_SPACING` between consecutive showings.
#[test]
fn deal_keeps_min_spacing_across_many_multishow() {
    let mut spec: Vec<(Ulid, usize)> = Vec::new();
    let mut tracked: Vec<Ulid> = Vec::new();
    for _ in 0..5 {
        let id = Ulid::new();
        tracked.push(id);
        spec.push((id, 3));
    }
    for _ in 0..8 {
        let id = Ulid::new();
        tracked.push(id);
        spec.push((id, 2));
    }
    for _ in 0..4 {
        spec.push((Ulid::new(), 1));
    }
    let content = build_content_spec(&spec);
    let dealt = deal_by_card_id(content);

    assert_eq!(dealt.len(), 5 * 3 + 8 * 2 + 4, "deal must place every slot");
    for id in &tracked {
        let gap = content_min_gap(&dealt, *id).expect("tracked card is multi-show");
        assert!(
            gap >= MIN_REPEAT_SPACING,
            "card {id} has gap={gap} < {MIN_REPEAT_SPACING}"
        );
    }
}

/// Within a card the showings keep their original view order: the primary
/// (Normal) variant precedes every copy (Reversed) in the dealt output.
#[test]
fn deal_preserves_within_card_view_order() {
    let a = Ulid::new();
    let b = Ulid::new();
    let content = build_content_spec(&[(a, 3), (b, 2), (Ulid::new(), 1)]);
    let dealt = deal_by_card_id(content);

    for id in [a, b] {
        let views: Vec<&LessonCardView> = dealt
            .iter()
            .filter(|(_, lc)| lc.card_id() == id)
            .map(|(_, lc)| lc.view())
            .collect();
        assert!(views.len() >= 2, "card {id} should have >= 2 showings");
        assert!(
            matches!(views[0], LessonCardView::Normal(_)),
            "primary (Normal) must be the first showing of card {id}"
        );
        assert!(
            views[1..]
                .iter()
                .all(|v| matches!(v, LessonCardView::Reversed(_))),
            "copies must follow the primary in view order for card {id}"
        );
    }
}

/// On a structurally infeasible core (one card with 3 showings and only 2
/// filler slots — needs 1+3+1+3+1=9 slots, has 5), the deal degrades to
/// best-effort without panicking and still emits every slot.
#[test]
fn deal_best_effort_on_infeasible_core() {
    let hard = Ulid::new();
    let content = build_content_spec(&[(hard, 3), (Ulid::new(), 1), (Ulid::new(), 1)]);
    let dealt = deal_by_card_id(content);
    assert_eq!(
        dealt.len(),
        5,
        "best-effort deal must still emit every slot"
    );
}

// --- Phrase no-starvation regression ---
//
// NEW no-anchor phrases must still appear when anchored phrases exhaust
// the new-phrase allowance. The no-anchor slot
// (TAIL_PHRASE_PER_LESSON) is independent of phrase_new_budget, so a
// tail-eligible NEW phrase anchored to no lesson word is admitted even
// with a depleted allowance.

fn phrase_id_independent() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HS").expect("valid ULID")
}

#[test]
fn new_no_anchor_phrase_appears_when_anchored_exhausts_budget() {
    ensure_test_phrase_index();

    let mut ks = KnowledgeSet::new();
    // Lesson-core anchors: "test" and "hello" are NEW (not known) so they
    // drive anchored phrase selection and consume the new-phrase budget.
    let test_sc = ks.create_card(vocab_card("test")).expect("create test");
    let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");

    // Known words that are NOT in the lesson core: they make the
    // [alpha, beta] phrase tail-eligible without turning it into an
    // anchored phrase.
    for word in ["alpha", "beta"] {
        let sc = ks
            .create_card(vocab_card(word))
            .expect("create known vocab");
        ks.mark_card_as_known(*sc.card_id()).expect("mark known");
    }

    // Anchored phrases (token "test"/"hello") + the independent phrase.
    for pid in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
        phrase_id_extra1(),
        phrase_id_extra2(),
        phrase_id_independent(),
    ] {
        ks.create_card(phrase_card(pid)).expect("create phrase");
    }

    let lesson = LessonData {
        cards: vec![
            (*test_sc.card_id(), lesson_card_for(vocab_card("test"))),
            (*hello_sc.card_id(), lesson_card_for(vocab_card("hello"))),
        ],
        core_count: 2,
    };

    // Budget is deliberately tiny: two anchored NEW phrases (one per anchor
    // word, INTERLEAVED_PHRASES_PER_WORD=2) drain it to zero before the
    // no-anchor pass runs.
    let mut budget = 2;
    let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
    let phrases = lesson_phrase_ids(&result);

    assert!(
        phrases.contains(&phrase_id_independent()),
        "NEW no-anchor phrase must appear even when anchored phrases exhaust the budget: \
             got {phrases:?}"
    );
}

// --- Proportional type-slot allocation (grammar-starvation fix) ---
//
// CARD_TYPE_WEIGHTS was originally meant as a PERCENTAGE split (V:K:G ≈
// 80:10:10) but the legacy `weighted_interleave_by_type` treated it as a
// round-robin pattern and sliced the first N positions off the result.
// At `daily_new_limit ≤ 9` this gave Grammar 0 slots every day. The new
// `compute_type_slots` uses largest-remainder + minor-priority tie-break
// so each type gets its proportional share even at small limits.

fn infinite_pool() -> HashMap<CardType, usize> {
    let large = 1_000;
    [
        (CardType::Vocabulary, large),
        (CardType::Kanji, large),
        (CardType::Grammar, large),
    ]
    .into_iter()
    .collect()
}

fn slots_count(slots: &HashMap<CardType, usize>, t: CardType) -> u32 {
    slots.get(&t).copied().unwrap_or(0) as u32
}

/// Expected per-type slot split for a single `allowed` value. Deterministic
/// cases must hold for every RNG seed; random cases must reach BOTH
/// branches across enough seeds.
enum SlotExpectation {
    Deterministic(u32, u32, u32),
    Random {
        a: (u32, u32, u32),
        b: (u32, u32, u32),
    },
}

#[rstest]
#[case::minimal(3, SlotExpectation::Random { a: (2, 1, 0), b: (2, 0, 1) })]
#[case::light(6, SlotExpectation::Deterministic(4, 1, 1))]
#[case::medium(9, SlotExpectation::Deterministic(7, 1, 1))]
#[case::hard(15, SlotExpectation::Random { a: (12, 2, 1), b: (12, 1, 2) })]
#[case::heavy(21, SlotExpectation::Random { a: (16, 3, 2), b: (16, 2, 3) })]
#[case::maximum_unit(30, SlotExpectation::Deterministic(24, 3, 3))]
fn compute_type_slots_reproduces_load_matrix(
    #[case] allowed: usize,
    #[case] expected: SlotExpectation,
) {
    let available = infinite_pool();
    match expected {
        SlotExpectation::Deterministic(v, k, g) => {
            for seed in [0u64, 1, 42, 99, 1234] {
                let mut rng = StdRng::seed_from_u64(seed);
                let slots = compute_type_slots(allowed, &available, &mut rng);
                assert_eq!(
                    slots_count(&slots, CardType::Vocabulary),
                    v,
                    "V wrong, allowed={allowed}, seed={seed}"
                );
                assert_eq!(
                    slots_count(&slots, CardType::Kanji),
                    k,
                    "K wrong, allowed={allowed}, seed={seed}"
                );
                assert_eq!(
                    slots_count(&slots, CardType::Grammar),
                    g,
                    "G wrong, allowed={allowed}, seed={seed}"
                );
            }
        },
        SlotExpectation::Random { a, b } => {
            let (mut found_a, mut found_b) = (false, false);
            for seed in 0..200u64 {
                let mut rng = StdRng::seed_from_u64(seed);
                let slots = compute_type_slots(allowed, &available, &mut rng);
                let triple = (
                    slots_count(&slots, CardType::Vocabulary),
                    slots_count(&slots, CardType::Kanji),
                    slots_count(&slots, CardType::Grammar),
                );
                assert!(
                    triple == a || triple == b,
                    "allowed={allowed}, seed={seed}: unexpected triple {triple:?}"
                );
                if triple == a {
                    found_a = true;
                }
                if triple == b {
                    found_b = true;
                }
            }
            assert!(found_a, "case A never observed for allowed={allowed}");
            assert!(found_b, "case B never observed for allowed={allowed}");
        },
    }
}

#[test]
fn compute_type_slots_returns_empty_when_allowed_zero() {
    let available = infinite_pool();
    let mut rng = StdRng::seed_from_u64(0);
    let slots = compute_type_slots(0, &available, &mut rng);
    assert!(slots.is_empty());
}

#[test]
fn compute_type_slots_returns_empty_when_pool_empty() {
    let available: HashMap<CardType, usize> = HashMap::new();
    let mut rng = StdRng::seed_from_u64(0);
    let slots = compute_type_slots(9, &available, &mut rng);
    assert!(slots.is_empty());
}

#[test]
fn compute_type_slots_normalizes_when_main_type_absent() {
    // V absent from pool → renormalize over K:G = 1:1, sum_w=2.
    let mut available = HashMap::new();
    available.insert(CardType::Kanji, 100);
    available.insert(CardType::Grammar, 100);
    for seed in [0u64, 1, 42] {
        let mut rng = StdRng::seed_from_u64(seed);
        let slots = compute_type_slots(10, &available, &mut rng);
        assert_eq!(slots_count(&slots, CardType::Vocabulary), 0, "seed={seed}");
        assert_eq!(slots_count(&slots, CardType::Kanji), 5, "seed={seed}");
        assert_eq!(slots_count(&slots, CardType::Grammar), 5, "seed={seed}");
    }
}

#[test]
fn compute_type_slots_phase2_fallback_to_vocabulary_when_grammar_capped() {
    // V=∞, K=∞, G=1, allowed=30 → raw G=3 capped to 1, leftover=2 falls to
    // V via Phase 2 (the cap-binding fallback path).
    let mut available = HashMap::new();
    available.insert(CardType::Vocabulary, 100);
    available.insert(CardType::Kanji, 100);
    available.insert(CardType::Grammar, 1);
    for seed in [0u64, 1, 42, 99] {
        let mut rng = StdRng::seed_from_u64(seed);
        let slots = compute_type_slots(30, &available, &mut rng);
        assert_eq!(
            slots_count(&slots, CardType::Vocabulary),
            26,
            "V should absorb the cap-induced leftover (seed={seed})"
        );
        assert_eq!(slots_count(&slots, CardType::Kanji), 3, "seed={seed}");
        assert_eq!(
            slots_count(&slots, CardType::Grammar),
            1,
            "G must be capped at available=1 (seed={seed})"
        );
    }
}

/// Direct unit test for `distribute_new_cards` multi-group overflow:
/// when allowed exceeds the first JLPT group's size, the leftover must
/// roll over to the next group. N5(3V only) is tiny, so the proportional
/// split for N4(∞) yields the expected 7V+1K+1G there.
#[test]
fn distribute_new_cards_overflows_leftover_to_next_jlpt_group() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["n5w1", "n5w2", "n5w3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        (0..30).map(|i| format!("n4w{i}")).collect::<HashSet<_>>(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N4,
        (0..10).map(|i| format!("n4k{i}")).collect::<HashSet<_>>(),
    );
    let gids: Vec<Ulid> = (0..5).map(|_| Ulid::new()).collect();
    jlpt_content.grammar_by_level.insert(
        JapaneseLevel::N4,
        gids.iter().map(|u| u.to_string()).collect::<HashSet<_>>(),
    );

    let mut ks = KnowledgeSet::new();
    // N5 group: 3 vocab only.
    for w in ["n5w1", "n5w2", "n5w3"] {
        ks.create_card(vocab_card(w)).unwrap();
    }
    // N4 group: 30V + 10K + 5G.
    for i in 0..30 {
        ks.create_card(vocab_card(&format!("n4w{i}"))).unwrap();
    }
    for i in 0..10 {
        ks.create_card(Card::Kanji(KanjiCard::new_test(format!("n4k{i}"))))
            .unwrap();
    }
    for gid in &gids {
        ks.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(*gid)))
            .unwrap();
    }

    let all_cards: Vec<(&Ulid, &StudyCard)> = ks.study_cards().iter().collect::<Vec<_>>();
    let mut rng = StdRng::seed_from_u64(0);
    let distributed = distribute_new_cards(all_cards, &jlpt_content, 9, &mut rng);
    assert_eq!(distributed.len(), 9, "must return exactly `allowed` cards");

    // Tally per type using the underlying StudyCard.
    let mut vocab = 0;
    let mut kanji = 0;
    let mut grammar = 0;
    for (_, sc) in &distributed {
        match sc.card() {
            Card::Vocabulary(_) => vocab += 1,
            Card::Kanji(_) => kanji += 1,
            Card::Grammar(_) => grammar += 1,
            _ => {},
        }
    }
    // N5 contributes 3V (its entire group), N4 contributes 4V+1K+1G
    // (matrix row "light" for take=6).
    assert_eq!(vocab, 7, "N5 3V + N4 4V = 7");
    assert_eq!(kanji, 1);
    assert_eq!(grammar, 1);
}

/// Direct unit test for `distribute_new_cards` JLPT priority: when the
/// first group alone can fill `allowed`, the second group is untouched.
#[test]
fn distribute_new_cards_consumes_first_jlpt_group_when_sufficient() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        (0..30).map(|i| format!("n5w{i}")).collect::<HashSet<_>>(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N5,
        (0..10).map(|i| format!("n5k{i}")).collect::<HashSet<_>>(),
    );
    let gids: Vec<Ulid> = (0..5).map(|_| Ulid::new()).collect();
    jlpt_content.grammar_by_level.insert(
        JapaneseLevel::N5,
        gids.iter().map(|u| u.to_string()).collect::<HashSet<_>>(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["n4w1"].into_iter().map(|s| s.to_string()).collect(),
    );

    let mut ks = KnowledgeSet::new();
    for i in 0..30 {
        ks.create_card(vocab_card(&format!("n5w{i}"))).unwrap();
    }
    for i in 0..10 {
        ks.create_card(Card::Kanji(KanjiCard::new_test(format!("n5k{i}"))))
            .unwrap();
    }
    for gid in &gids {
        ks.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(*gid)))
            .unwrap();
    }
    let n4_sc = ks.create_card(vocab_card("n4w1")).unwrap();

    let all_cards: Vec<(&Ulid, &StudyCard)> = ks.study_cards().iter().collect::<Vec<_>>();
    let mut rng = StdRng::seed_from_u64(0);
    let distributed = distribute_new_cards(all_cards, &jlpt_content, 9, &mut rng);
    assert_eq!(distributed.len(), 9);

    // N4 card must NOT appear — N5 alone satisfied `allowed`.
    assert!(
        !distributed.iter().any(|(id, _)| **id == *n4_sc.card_id()),
        "N4 card leaked while N5 group had enough cards"
    );
}

#[test]
fn grammar_appears_in_lesson_at_medium_load() {
    // End-to-end: 30V + 10K + 5G all N5, daily_new_limit=9 (Medium).
    // Before the fix this returned 8V + 1K + 0G (grammar starved).
    // After: 7V + 1K + 1G exactly (deterministic per matrix row "medium").
    let mut ks = KnowledgeSet::new();
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        (0..30).map(|i| format!("vocab{i}")).collect::<HashSet<_>>(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N5,
        (0..10).map(|i| format!("kanji{i}")).collect::<HashSet<_>>(),
    );
    let grammar_ids: Vec<Ulid> = (0..5).map(|_| Ulid::new()).collect();
    jlpt_content.grammar_by_level.insert(
        JapaneseLevel::N5,
        grammar_ids
            .iter()
            .map(|u| u.to_string())
            .collect::<HashSet<_>>(),
    );

    for i in 0..30 {
        ks.create_card(vocab_card(&format!("vocab{i}"))).unwrap();
    }
    for i in 0..10 {
        ks.create_card(Card::Kanji(KanjiCard::new_test(format!("kanji{i}"))))
            .unwrap();
    }
    for gid in &grammar_ids {
        ks.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(*gid)))
            .unwrap();
    }

    let lesson = ks.cards_to_lesson(
        DailyBudget::with_daily_cards(9),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    // Multi-show expansion may add an extra slot for the same grammar
    // card_id, so we count DISTINCT grammar card_ids, not raw slots.
    let grammar_card_ids: HashSet<Ulid> = lesson
        .values()
        .filter_map(|lc| {
            let card = ks.get_card(lc.card_id())?.card();
            matches!(card, Card::Grammar(_)).then_some(lc.card_id())
        })
        .collect();

    assert_eq!(
        grammar_card_ids.len(),
        1,
        "Medium=9 must include exactly 1 distinct grammar card, got {}",
        grammar_card_ids.len()
    );
}

// --- Рука добиваний [GhostHand] (ghost relearning) ---
//
// Добивание — параллельная FSRS механика закрепления проваленных карт
// (docs/plans/ghost-relearning.md). Отбор: карты с активным добиванием и
// открытым окном входят в урок с высшим приоритетом (после избранного),
// единственным каналом показа; core-отбор и padding их исключают.

use crate::domain::memory::{Difficulty, GhostRung, GhostState, MemoryState, Stability};
use chrono::{Duration, Utc};

fn seed_memory(ks: &mut KnowledgeSet, card_id: &Ulid, difficulty: f64, due_days_ago: i64) {
    let state = MemoryState::new(
        Stability::new(5.0).unwrap(),
        Difficulty::new(difficulty).unwrap(),
        Utc::now() - Duration::days(due_days_ago),
    );
    let sc = ks.study_cards_mut_for_test().get_mut(card_id).unwrap();
    sc.seed_first_review(state);
}

fn set_ghost(ks: &mut KnowledgeSet, card_id: &Ulid, ghost: GhostState) {
    let sc = ks.study_cards_mut_for_test().get_mut(card_id).unwrap();
    sc.memory_history_mut_for_test()
        .set_ghost_for_test(Some(ghost));
}

/// Активное добивание с открытым окном (due час назад, последний шаг —
/// два часа назад: TTL заведомо не истёк).
fn open_ghost(rung: GhostRung) -> GhostState {
    let now = Utc::now();
    GhostState::active(rung, now - Duration::hours(1), now - Duration::hours(2))
}

/// Активное добивание с закрытым окном (due через день).
fn closed_ghost(rung: GhostRung) -> GhostState {
    let now = Utc::now();
    GhostState::active(rung, now + Duration::days(1), now - Duration::hours(2))
}

fn lesson_card_ids(lesson: &LessonData) -> Vec<Ulid> {
    lesson.cards.iter().map(|(_, lc)| lc.card_id()).collect()
}

fn build_lesson(ks: &KnowledgeSet, policy: NewCardPolicy) -> LessonData {
    ks.cards_to_lesson_with_policy(
        DailyBudget::with_daily_cards(22),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
        policy,
    )
}

#[test]
fn ghost_cards_fill_lesson_before_new_cards() {
    // Arrange: открытое добивание + много новых карт с дневным лимитом 22
    let mut ks = KnowledgeSet::new();
    let ghost_sc = ks.create_card(vocab_card("鬼")).expect("create ghost card");
    seed_memory(&mut ks, ghost_sc.card_id(), 5.0, 1);
    set_ghost(&mut ks, ghost_sc.card_id(), open_ghost(GhostRung::First));
    for i in 0..30 {
        ks.create_card(vocab_card(&format!("new{i}")))
            .expect("create new card");
    }

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Inject);

    // Assert: добивание в уроке и вытесняет одну новую карту из бюджета
    let ids = lesson_card_ids(&lesson);
    assert!(
        ids.contains(ghost_sc.card_id()),
        "ghost hand card must enter"
    );
    let new_count = ids
        .iter()
        .filter(|id| {
            ks.study_cards()
                .get(id)
                .is_some_and(|sc| sc.memory().is_new())
        })
        .count();
    assert_eq!(new_count, 21, "ghost displaces exactly one new card slot");
    assert!(ids.len() <= 22, "lesson must respect MAX_LESSON_SIZE");
}

#[test]
fn ghost_overflow_enters_oldest_first_rest_waits_next_lesson() {
    // Arrange: 3 избранного сужают бюджет до 19; добиваний с открытым
    // окном — 25, окна открывались в разное время
    let mut ks = KnowledgeSet::new();
    for i in 0..3 {
        let sc = ks.create_card(vocab_card(&format!("fav{i}"))).expect("fav");
        seed_memory(&mut ks, sc.card_id(), 5.0, 1);
        ks.toggle_favorite(*sc.card_id()).expect("favorite");
    }
    let mut expected_in: Vec<Ulid> = Vec::new();
    for i in 0..25 {
        let sc = ks
            .create_card(vocab_card(&format!("ghost{i}")))
            .expect("ghost card");
        seed_memory(&mut ks, sc.card_id(), 5.0, 1);
        // Чем больше i — тем «свежее» окно; старейшие (i=0..) должны войти
        let overdue_hours = 48 - i;
        let now = Utc::now();
        set_ghost(
            &mut ks,
            sc.card_id(),
            GhostState::active(
                GhostRung::First,
                now - Duration::hours(overdue_hours),
                now - Duration::hours(overdue_hours + 1),
            ),
        );
        if i < 19 {
            expected_in.push(*sc.card_id());
        }
    }

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: вошли 19 старейших, свежие ждут следующего урока
    let ids: std::collections::HashSet<Ulid> = lesson_card_ids(&lesson).into_iter().collect();
    for id in &expected_in {
        assert!(ids.contains(id), "oldest ghost {id} must enter the lesson");
    }
    assert_eq!(
        ids.len(),
        22,
        "3 favorites + 19 oldest ghosts fill the budget"
    );
}

#[test]
fn ghost_with_closed_window_absent_from_hand_core_and_padding() {
    // Arrange: сложная (high-difficulty) due карта с закрытым окном
    let mut ks = KnowledgeSet::new();
    let ghost_sc = ks.create_card(vocab_card("閉")).expect("create");
    seed_memory(&mut ks, ghost_sc.card_id(), 8.0, 3);
    set_ghost(&mut ks, ghost_sc.card_id(), closed_ghost(GhostRung::Second));
    // Наполнитель, чтобы урок собрался
    for i in 0..10 {
        let sc = ks.create_card(vocab_card(&format!("pad{i}"))).expect("pad");
        seed_memory(&mut ks, sc.card_id(), 8.0, 1);
    }

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: карта отсутствует целиком — ни рука, ни core, ни padding
    let ids = lesson_card_ids(&lesson);
    assert!(
        !ids.contains(ghost_sc.card_id()),
        "closed-window ghost must not appear via hand, core or padding"
    );
}

#[test]
fn expired_ghost_card_returns_to_normal_core_selection() {
    // Arrange: добивание молчало 31 день (TTL истёк), карта сложная и due
    let mut ks = KnowledgeSet::new();
    let stale = Utc::now() - Duration::days(31);
    let sc = ks.create_card(vocab_card("古")).expect("create");
    seed_memory(&mut ks, sc.card_id(), 8.0, 2);
    set_ghost(
        &mut ks,
        sc.card_id(),
        GhostState::active(GhostRung::Second, stale, stale),
    );

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: TTL-списание снимает исключение — карта снова в core
    let ids = lesson_card_ids(&lesson);
    assert!(
        ids.contains(sc.card_id()),
        "expired ghost card must return to normal core selection"
    );
    assert_eq!(
        ids.iter().filter(|id| **id == *sc.card_id()).count(),
        1,
        "card enters exactly once"
    );
}

#[test]
fn ghost_card_enters_once_via_hand_not_via_core() {
    // Arrange: карта одновременно high-difficulty due И с открытым окном
    let mut ks = KnowledgeSet::new();
    let sc = ks.create_card(vocab_card("一")).expect("create");
    seed_memory(&mut ks, sc.card_id(), 8.0, 2);
    set_ghost(&mut ks, sc.card_id(), open_ghost(GhostRung::First));
    for i in 0..10 {
        let pad = ks.create_card(vocab_card(&format!("p{i}"))).expect("pad");
        seed_memory(&mut ks, pad.card_id(), 8.0, 1);
    }

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: ровно один показ — через руку, core-канал исключён
    let count = lesson_card_ids(&lesson)
        .iter()
        .filter(|id| **id == *sc.card_id())
        .count();
    assert_eq!(count, 1, "ghost card must appear exactly once (via hand)");
}

#[test]
fn favorite_and_ghost_deduped_to_single_showing() {
    // Arrange: избранная карта с открытым окном добивания
    let mut ks = KnowledgeSet::new();
    let sc = ks.create_card(vocab_card("好")).expect("create");
    seed_memory(&mut ks, sc.card_id(), 5.0, 1);
    ks.toggle_favorite(*sc.card_id()).expect("favorite");
    set_ghost(&mut ks, sc.card_id(), open_ghost(GhostRung::First));

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: один показ (избранное pin + дедуп с рукой)
    let count = lesson_card_ids(&lesson)
        .iter()
        .filter(|id| **id == *sc.card_id())
        .count();
    assert_eq!(count, 1, "favorite + ghost must dedupe to a single showing");
}

#[rstest]
#[case::vocabulary(Card::Vocabulary(VocabularyCard::new(
    Question::new("語".to_string()).unwrap(),
)))]
#[case::kanji(Card::Kanji(KanjiCard::new_test("字".to_string())))]
#[case::grammar(Card::Grammar(GrammarRuleCard::new_test()))]
#[case::phrase(phrase_card(phrase_id_hello()))]
fn ghost_selection_applies_to_all_card_types(#[case] card: Card) {
    // Arrange: карта произвольного типа с открытым окном
    let mut ks = KnowledgeSet::new();
    ensure_test_phrase_index();
    let sc = ks.create_card(card).expect("create card");
    seed_memory(&mut ks, sc.card_id(), 5.0, 1);
    set_ghost(&mut ks, sc.card_id(), open_ghost(GhostRung::First));
    // Сосед-наполнитель, чтобы core не был пуст
    let pad = ks.create_card(vocab_card("_neighbor")).expect("pad");
    seed_memory(&mut ks, pad.card_id(), 8.0, 1);

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert
    let ids = lesson_card_ids(&lesson);
    assert!(
        ids.contains(sc.card_id()),
        "ghost card of any type must enter via hand"
    );
}

#[test]
fn after_expansion_removal_no_card_shows_twice() {
    // Arrange: смешанный набор — новые, сложные due, добивания
    let mut ks = KnowledgeSet::new();
    for i in 0..8 {
        ks.create_card(vocab_card(&format!("n{i}"))).expect("new");
    }
    for i in 0..8 {
        let sc = ks.create_card(vocab_card(&format!("hd{i}"))).expect("hard");
        seed_memory(&mut ks, sc.card_id(), 8.0, 2);
    }
    for i in 0..4 {
        let sc = ks.create_card(vocab_card(&format!("g{i}"))).expect("ghost");
        seed_memory(&mut ks, sc.card_id(), 8.0, 1);
        set_ghost(&mut ks, sc.card_id(), open_ghost(GhostRung::First));
    }

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Inject);

    // Assert: дубль-механизм удалён — каждый card_id уникален
    let ids = lesson_card_ids(&lesson);
    let distinct: std::collections::HashSet<Ulid> = ids.iter().copied().collect();
    assert_eq!(
        ids.len(),
        distinct.len(),
        "every card must show at most once per lesson"
    );
}

#[test]
fn ghost_card_shows_via_standard_generator_not_short_term() {
    // Arrange: добивание со стабильной (не сложной) памятью
    let mut ks = KnowledgeSet::new();
    let sc = ks.create_card(vocab_card("標準")).expect("create");
    seed_memory(&mut ks, sc.card_id(), 3.0, 1);
    set_ghost(&mut ks, sc.card_id(), open_ghost(GhostRung::First));

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: обычный показ (не short-term padding) стандартным генератором
    let (_, lc) = lesson
        .cards
        .iter()
        .find(|(_, lc)| lc.card_id() == *sc.card_id())
        .expect("ghost card must be in lesson");
    assert!(!lc.is_short_term(), "hand showing is a normal review slot");
}

#[test]
fn ghost_phrase_card_places_after_content_words() {
    // Arrange: фраза-добивание + слова в core
    ensure_test_phrase_index();
    let mut ks = KnowledgeSet::new();
    let phrase_sc = ks
        .create_card(phrase_card(phrase_id_hello()))
        .expect("phrase");
    seed_memory(&mut ks, phrase_sc.card_id(), 5.0, 1);
    set_ghost(&mut ks, phrase_sc.card_id(), open_ghost(GhostRung::First));
    for word in ["alpha", "beta"] {
        let sc = ks.create_card(vocab_card(word)).expect("word");
        seed_memory(&mut ks, sc.card_id(), 5.0, 1);
    }

    // Act
    let lesson = build_lesson(&ks, NewCardPolicy::Exclude);

    // Assert: фраза в уроке и не стоит первой — после слов
    let position = lesson
        .cards
        .iter()
        .position(|(_, lc)| lc.card_id() == *phrase_sc.card_id())
        .expect("ghost phrase must enter the lesson");
    assert!(position > 0, "phrase must be placed after content words");
}

// ═══════════════════════════════════════════════════════════════════════
// Счётные суффиксы (issue #415): руки знакомства и интерлив
// ═══════════════════════════════════════════════════════════════════════

/// Состав руки знакомства включает counter-карты: пул только из
/// счётчиков N5 распределяется полностью (weights fallback добирает
/// минорные типы) — один тест закрывает weights+distribute+slots.
#[test]
fn counter_cards_enter_the_acquaintance_distribution() {
    crate::dictionary::counters::tests::init_test_counters();
    let mut user = crate::domain::User::new(
        "c@e.st".to_string(),
        crate::domain::NativeLanguage::Russian,
        None,
    );
    for suffix in ["本", "人", "日"] {
        let mut counter = crate::domain::CounterCard::new(suffix);
        counter.ensure_registry_bindings();
        user.create_card(crate::domain::Card::Counter(counter))
            .unwrap();
    }
    let mut content = crate::domain::JlptContent::new();
    content
        .counters_by_level
        .entry(crate::domain::JapaneseLevel::N5)
        .or_default()
        .extend(["本".to_string(), "人".to_string(), "日".to_string()]);

    let pool: Vec<(&ulid::Ulid, &crate::domain::StudyCard)> =
        user.knowledge_set().study_cards().iter().collect();
    let mut rng = rand::rng();
    let selected = super::distribute_new_cards(pool, &content, 3, &mut rng);

    assert_eq!(selected.len(), 3, "все counter-карты распределены в руку");
    for (_, sc) in &selected {
        assert!(matches!(sc.card(), crate::domain::Card::Counter(_)));
    }
}

/// Интерлив: counter-карты разнесены round-robin в vocab-промежутки
/// (собственная ветка, не хвостовой `other` как у фраз).
#[test]
fn interleave_spreads_counter_cards_across_vocab_gaps() {
    crate::dictionary::counters::tests::init_test_counters();
    let mut data = LessonData {
        cards: Vec::new(),
        core_count: 0,
    };
    let vocab_ids: Vec<ulid::Ulid> = (0..2).map(|_| ulid::Ulid::new()).collect();
    let counter_id = ulid::Ulid::new();
    let vocab_view = |id| {
        (
            id,
            crate::domain::LessonCard::new(
                id,
                crate::domain::LessonCardView::Normal(crate::domain::Card::Vocabulary(
                    crate::domain::VocabularyCard::new(
                        crate::domain::value_objects::Question::new("た".to_string()).unwrap(),
                    ),
                )),
                false,
            ),
        )
    };
    let mut counter = crate::domain::CounterCard::new("本");
    counter.ensure_registry_bindings();
    data.cards.push(vocab_view(vocab_ids[0]));
    data.cards.push((
        counter_id,
        crate::domain::LessonCard::new(
            counter_id,
            crate::domain::LessonCardView::Normal(crate::domain::Card::Counter(counter)),
            false,
        ),
    ));
    data.cards.push(vocab_view(vocab_ids[1]));
    data.core_count = data.cards.len();

    let interleaved = super::interleave_core_by_type(data);
    let counter_pos = interleaved
        .cards
        .iter()
        .position(|(id, _)| *id == counter_id)
        .expect("counter card survived interleaving");
    assert_eq!(
        counter_pos, 1,
        "counter lands in a vocab gap, not at the tail (phrase `other` lane)"
    );
}
