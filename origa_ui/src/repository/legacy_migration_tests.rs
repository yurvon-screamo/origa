use super::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use origa::domain::{
    Card, DailyLoad, JlptProgress, KnowledgeSet, NativeLanguage, PhraseCard, User,
};
use std::collections::HashSet;

const CANONICAL_ID_BYTES: [u8; 16] = [
    0x01, 0x67, 0x4f, 0x3a, 0x4e, 0x32, 0xdf, 0x41, 0x8c, 0x6b, 0x27, 0x4e, 0x73, 0x91, 0x5f, 0xce,
];

fn canonical_id() -> Ulid {
    Ulid::from_bytes(CANONICAL_ID_BYTES)
}

fn build_user_with_id(id: Ulid) -> User {
    User::from_row(
        id,
        "legacy@example.com".to_string(),
        "legacy".to_string(),
        JlptProgress::new(),
        NativeLanguage::Russian,
        None,
        KnowledgeSet::new(),
        Utc::now(),
        HashSet::new(),
        DailyLoad::default(),
        0,
    )
}

/// Build a canonical-row user with profile fields deliberately different from
/// the legacy row, so any accidental clobber during merge is observable in
/// assertions. Identity is `canonical_id()`; email/username/native_language
/// are distinct from `build_user_with_id`.
fn build_canonical_user() -> User {
    User::from_row(
        canonical_id(),
        "canonical@example.com".to_string(),
        "canonical".to_string(),
        JlptProgress::new(),
        NativeLanguage::English,
        Some(99),
        KnowledgeSet::new(),
        Utc::now(),
        HashSet::new(),
        DailyLoad::default(),
        0,
    )
}

// ---------------------------------------------------------------------------
// reseat_user_id
// ---------------------------------------------------------------------------

#[test]
fn reseat_user_id_changes_only_id() {
    let nil_user = build_user_with_id(Ulid::nil());

    let reseated = reseat_user_id(&nil_user, canonical_id());

    assert_eq!(reseated.id(), canonical_id());
    assert_ne!(reseated.id(), Ulid::nil());
    assert_eq!(reseated.email(), nil_user.email());
    assert_eq!(reseated.username(), nil_user.username());
}

#[test]
fn reseat_user_id_preserves_all_non_identity_fields() {
    let original = build_user_with_id(Ulid::nil());

    let reseated = reseat_user_id(&original, Ulid::new());

    assert_eq!(reseated.email(), original.email());
    assert_eq!(reseated.username(), original.username());
    assert_eq!(reseated.native_language(), original.native_language());
    assert_eq!(reseated.known_vocab_hash(), original.known_vocab_hash());
}

// ---------------------------------------------------------------------------
// merge_legacy_progress_into_canonical (Common #1 fix)
// ---------------------------------------------------------------------------

#[test]
fn merge_keeps_canonical_identity_not_legacy() {
    // Regression guard for the discard→merge fix: identity must come from the
    // canonical row, never from the legacy nil-keyed row, otherwise the
    // cross-device split-progress bug returns.
    let canonical = build_canonical_user();
    let legacy = build_user_with_id(Ulid::nil());

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert_eq!(merged.id(), canonical_id());
    assert_eq!(merged.email(), "canonical@example.com");
    assert_eq!(merged.username(), "canonical");
}

#[test]
fn merge_keeps_canonical_profile_fields() {
    // native_language and telegram_user_id are profile fields owned by the
    // remote; the legacy row must not overwrite them even when it was the
    // source of the user's local progress.
    let canonical = build_canonical_user();
    let legacy = build_user_with_id(Ulid::nil());

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert_eq!(*merged.native_language(), NativeLanguage::English);
    assert_eq!(merged.telegram_user_id(), Some(&99));
}

#[test]
fn merge_unions_imported_sets() {
    let mut canonical = build_canonical_user();
    canonical.mark_set_as_imported("set_canonical".to_string());

    let mut legacy = build_user_with_id(Ulid::nil());
    legacy.mark_set_as_imported("set_legacy_a".to_string());
    legacy.mark_set_as_imported("set_legacy_b".to_string());

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    let imported: HashSet<&str> = merged.imported_sets().iter().map(String::as_str).collect();
    assert!(imported.contains("set_canonical"));
    assert!(imported.contains("set_legacy_a"));
    assert!(imported.contains("set_legacy_b"));
    assert_eq!(imported.len(), 3);
}

#[test]
fn merge_does_not_drop_legacy_progress_when_canonical_exists() {
    // The core Common #1 regression: previously, when a canonical row existed,
    // the legacy nil-keyed row was deleted outright and its accumulated
    // imported_sets were lost. The merge must preserve every legacy
    // imported_set marker.
    let canonical = build_canonical_user();
    let mut legacy = build_user_with_id(Ulid::nil());
    legacy.mark_set_as_imported("legacy_only".to_string());

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert!(
        merged.is_set_imported("legacy_only"),
        "legacy imported_set must survive the merge, not be discarded"
    );
}

#[test]
fn merge_handles_empty_legacy_without_change() {
    let canonical = build_canonical_user();
    let legacy = build_user_with_id(Ulid::nil());

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert_eq!(merged.id(), canonical.id());
    assert_eq!(merged.email(), canonical.email());
    assert!(merged.imported_sets().is_empty());
}

#[test]
fn merge_touches_updated_at() {
    // The merged row is a fresh write to IndexedDB; its updated_at must move
    // forward so subsequent sync sees it as newer than the canonical source.
    let canonical = build_canonical_user();
    let canonical_updated_at = *canonical.updated_at();
    let legacy = build_user_with_id(Ulid::nil());

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert!(*merged.updated_at() >= canonical_updated_at);
}

#[test]
fn merge_preserves_legacy_knowledge_set_cards() {
    // The main progress payload lives in knowledge_set (SRS state for every
    // card the user studied). A regression that drops the
    // `merged_knowledge.merge(legacy.knowledge_set())` call would pass every
    // other merge test, because the other builders leave knowledge_set empty.
    // PhraseCard::new is dictionary-free, so this test stays hermetic.
    let canonical = build_canonical_user();

    let mut legacy = build_user_with_id(Ulid::nil());
    let legacy_card_id = *legacy
        .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
        .unwrap()
        .card_id();

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert!(
        merged.knowledge_set().get_card(legacy_card_id).is_some(),
        "legacy knowledge_set card must survive the merge, not be discarded"
    );
    assert_eq!(
        merged.knowledge_set().study_cards().len(),
        1,
        "exactly the legacy card should be present (canonical had none)"
    );
}

#[test]
fn merge_unions_canonical_and_legacy_knowledge_set_cards() {
    // Both sides contribute cards; the union must keep every card id from
    // both the canonical and the legacy row.
    let mut canonical = build_canonical_user();
    let canonical_card_id = *canonical
        .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
        .unwrap()
        .card_id();

    let mut legacy = build_user_with_id(Ulid::nil());
    let legacy_card_id = *legacy
        .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
        .unwrap()
        .card_id();

    let merged = merge_legacy_progress_into_canonical(&canonical, &legacy);

    assert!(merged.knowledge_set().get_card(canonical_card_id).is_some());
    assert!(merged.knowledge_set().get_card(legacy_card_id).is_some());
    assert_eq!(merged.knowledge_set().study_cards().len(), 2);
}

// ---------------------------------------------------------------------------
// canonical_id_from_trailbase_id (resolve_canonical_id pure half)
// ---------------------------------------------------------------------------

#[test]
fn canonical_id_returns_none_for_empty_trailbase_id() {
    // No session id means we cannot derive a canonical id; deferring the
    // migration (rather than guessing) is the correct outcome.
    assert!(canonical_id_from_trailbase_id("").is_none());
}

#[test]
fn canonical_id_returns_none_for_undecodable_trailbase_id() {
    // uuid_to_ulid fails safe to nil for garbage input; the canonical-id
    // resolver must treat nil as "no id" and return None so the migration
    // defers instead of re-keying everything under nil again.
    assert!(canonical_id_from_trailbase_id("not-a-valid-id").is_none());
}

#[test]
fn canonical_id_returns_some_for_valid_base64_url_no_pad() {
    let expected = canonical_id();
    let encoded = URL_SAFE_NO_PAD.encode(expected.to_bytes());

    let resolved = canonical_id_from_trailbase_id(&encoded);

    assert_eq!(resolved, Some(expected));
}

#[test]
fn canonical_id_returns_some_for_valid_hex_uuid() {
    let expected = canonical_id();
    let bytes = expected.to_bytes();
    let hex = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    );

    let resolved = canonical_id_from_trailbase_id(&hex);

    assert_eq!(resolved, Some(expected));
}
