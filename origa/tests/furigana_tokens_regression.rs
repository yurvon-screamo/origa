//! Tester pins for uwuwu#1705 at the public domain boundary.
//!
//! The engineer's unit tests cover the helper inside its own module; these
//! tests drive the same behavior through the public API, so a future
//! refactor that changes the export, the store contract, or the opt-in
//! source cannot silently regress the phrase question/answer agreement.

use std::collections::HashSet;
use std::sync::Mutex;

use origa::dictionary::furigana_dict::ReadingSpan;
use origa::domain::{
    AnnotatedSpan, PartOfSpeech, PrecomputedEntry, PrecomputedToken, furiganize_segments,
    furiganize_segments_from_precomputed_tokens, furiganize_text_precomputed,
    install_precomputed_entry, reset_precomputed_store,
};

// The precomputed store is process-global. Each integration test binary is
// its own process, so this lock only has to serialize the tests in THIS
// file; it must not reach across to the unit tests in `origa/src/domain`.
static STORE_LOCK: Mutex<()> = Mutex::new(());

struct StoreGuard;

impl Drop for StoreGuard {
    fn drop(&mut self) {
        reset_precomputed_store();
    }
}

fn token(surface: &str, reading: &str) -> PrecomputedToken {
    PrecomputedToken {
        surface: surface.to_string(),
        base: surface.to_string(),
        reading: reading.to_string(),
        pos: PartOfSpeech::PreNounAdjectival,
    }
}

fn conflicting_entry() -> PrecomputedEntry {
    PrecomputedEntry {
        furigana_spans: vec![AnnotatedSpan {
            text: "酷く".to_string(),
            reading: Some("むごい".to_string()),
            reading_spans: vec![ReadingSpan {
                start_index: 0,
                end_index: 0,
                text: "むご".to_string(),
            }],
        }],
        tokens: vec![
            token("酷く", "ヒドク"),
            token("ない", "ナイ"),
            token("！", "！"),
        ],
    }
}

/// The bug's exact shape: two readings for one kanji in one entry. Only
/// the opt-in helper may pick the token reading; the general path stays
/// on the JmdictFurigana lemma span (uwuwu#1705, ADR-084 §5).
#[test]
fn phrase_helper_reads_tokens_while_general_path_keeps_spans() {
    let _lock = STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = StoreGuard;
    install_precomputed_entry("酷くない！", conflicting_entry());

    let helper = furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new())
        .expect("conflicting entry carries tokens");
    assert_eq!(
        helper.iter().map(|s| s.text()).collect::<String>(),
        "酷くない！",
        "token surfaces must rebuild the rendered text exactly"
    );
    assert_eq!(helper[0].reading(), Some("ひどく"));

    let general = furiganize_segments("酷くない！", &HashSet::new())
        .expect("spans are present for the general path");
    assert_eq!(
        general[0].reading(),
        Some("むご"),
        "the general furiganize path must keep its spans-first contract"
    );
}

/// A partial tokenization must be a miss, not a question that silently
/// drops characters — the caller then falls back to spans/dictionary.
#[test]
fn partial_token_coverage_is_a_miss() {
    let _lock = STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = StoreGuard;
    install_precomputed_entry(
        "酷くない！",
        PrecomputedEntry {
            furigana_spans: vec![AnnotatedSpan {
                text: "酷くない！".to_string(),
                reading: Some("ひどくない".to_string()),
                reading_spans: vec![],
            }],
            tokens: vec![token("酷く", "ヒドク")],
        },
    );

    assert!(
        furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new()).is_none(),
        "surfaces that do not rebuild the text must not be used for the question"
    );
}

/// Known-kanji flags are computed from the token surfaces, not copied from
/// the span entry: the same entry yields different flags for different
/// learner knowledge sets.
#[test]
fn known_kanji_flags_follow_the_learner_knowledge_set() {
    let _lock = STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = StoreGuard;
    install_precomputed_entry("酷くない！", conflicting_entry());

    let known = HashSet::from(['酷']);
    let flagged = furiganize_segments_from_precomputed_tokens("酷くない！", &known)
        .expect("conflicting entry carries tokens");
    assert_eq!(flagged[0].reading(), Some("ひどく"));
    assert!(flagged[0].is_known(), "known kanji must stay known");

    let unknown = furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new())
        .expect("conflicting entry carries tokens");
    assert!(
        !unknown[0].is_known(),
        "unknown kanji must not be reported as known"
    );
}

/// A token with a kanji surface but no reading must render plain text
/// rather than an empty ruby annotation.
#[test]
fn kanji_token_without_reading_has_no_ruby() {
    let _lock = STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = StoreGuard;
    install_precomputed_entry(
        "酷",
        PrecomputedEntry {
            furigana_spans: vec![],
            tokens: vec![token("酷", "")],
        },
    );

    let segments = furiganize_segments_from_precomputed_tokens("酷", &HashSet::new())
        .expect("a complete tokenization is a hit even without a reading");
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].reading(), None);
    assert!(!segments[0].is_known());
}

/// The precomputed contract (`precomputed.rs`): empty spans are a miss for
/// the general furiganize path regardless of tokens. The opt-in helper
/// deliberately does not change that.
#[test]
fn empty_spans_with_tokens_stay_a_general_furiganize_miss() {
    let _lock = STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = StoreGuard;
    install_precomputed_entry(
        "酷くない！",
        PrecomputedEntry {
            furigana_spans: vec![],
            tokens: vec![token("酷くない！", "ヒドクナイ")],
        },
    );

    assert!(
        furiganize_text_precomputed("酷くない！", &HashSet::new()).is_none(),
        "card words must keep their dictionary/live reading"
    );
    assert!(
        furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new()).is_some(),
        "the opt-in helper is allowed to read tokens for the phrase question"
    );
}
