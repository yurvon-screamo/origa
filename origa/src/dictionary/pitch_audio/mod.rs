//! Pitch audio index: storage (owned JSON-built form or the zero-copy
//! archived view of the CDN blob), initialization entry points and the
//! lookup API.

mod index;

use std::sync::OnceLock;

pub use index::{
    ArchivedPitchAudioIndexBlob, PitchAudioEntry, PitchAudioIndex, PitchAudioIndexBlob,
    access_pitch_index_blob, serialize_pitch_index_blob_to_rkyv,
};

use crate::domain::OrigaError;

/// Loaded pitch audio index storage. The CDN rkyv blob installs a
/// zero-copy archived view; the JSON fallback builds the owned index.
enum PitchStore {
    Owned(PitchAudioIndex),
    Archived(&'static ArchivedPitchAudioIndexBlob),
}

static PITCH_AUDIO_INDEX: OnceLock<PitchStore> = OnceLock::new();

pub fn init_pitch_audio_index(json: &str) -> Result<(), OrigaError> {
    let index = PitchAudioIndex::from_json(json)?;
    PITCH_AUDIO_INDEX
        .set(PitchStore::Owned(index))
        .map_err(|_| OrigaError::PitchAudioParseError {
            reason: "Pitch audio index already initialized".to_string(),
        })
}

/// Install the zero-copy archived view of a CDN blob payload.
pub fn install_archived_pitch_index(
    view: &'static ArchivedPitchAudioIndexBlob,
) -> Result<(), OrigaError> {
    PITCH_AUDIO_INDEX
        .set(PitchStore::Archived(view))
        .map_err(|_| OrigaError::PitchAudioParseError {
            reason: "Pitch audio index already initialized".to_string(),
        })
}

pub fn is_pitch_audio_loaded() -> bool {
    PITCH_AUDIO_INDEX.get().is_some()
}

/// Simple word lookup. Prefer [`get_audio_for_reading`] which uses tokenizer
/// reading for correct pitch matching.
pub fn get_audio_for_word(word: &str) -> Option<PitchAudioEntry> {
    match PITCH_AUDIO_INDEX.get() {
        Some(PitchStore::Owned(index)) => index.get_entry(word).cloned(),
        Some(PitchStore::Archived(view)) => view.get_entry(word),
        None => None,
    }
}

/// Lookup pitch audio by word and reading (from tokenizer).
/// Tries: "word|reading" → "reading" → "word".
pub fn get_audio_for_reading(word: &str, reading: &str) -> Option<PitchAudioEntry> {
    match PITCH_AUDIO_INDEX.get() {
        Some(PitchStore::Owned(index)) => index.find_audio_for_reading(word, reading).cloned(),
        Some(PitchStore::Archived(view)) => view.find_audio_for_reading(word, reading),
        None => None,
    }
}

pub fn pitch_audio_version() -> u32 {
    match PITCH_AUDIO_INDEX.get() {
        Some(PitchStore::Owned(index)) => index.version,
        Some(PitchStore::Archived(view)) => view.version(),
        None => 0,
    }
}

pub fn get_audio_entry_count() -> usize {
    match PITCH_AUDIO_INDEX.get() {
        Some(PitchStore::Owned(index)) => index.len(),
        Some(PitchStore::Archived(view)) => view.len(),
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These run before any test populates the global slot in this binary;
    // the fallback semantics of the free functions are the contract.
    #[test]
    fn lookups_before_initialization_yield_none_and_zero() {
        if is_pitch_audio_loaded() {
            return; // another test won the race — fallback semantics N/A
        }
        assert!(get_audio_for_word("猫").is_none());
        assert_eq!(pitch_audio_version(), 0);
        assert_eq!(get_audio_entry_count(), 0);
    }
}

#[cfg(test)]
mod owned_branch_tests {
    use super::*;

    /// The v3 composite-key form exercises the full lookup fallback chain
    /// ("word|reading" → "reading" → "word") through the global API.
    fn index_v3_json() -> &'static str {
        r#"{"v":3,"total":5,"entries":{"役|やく":{"f":"yaku.opus","p":1},"役|えき":{"f":"eki.opus","p":0},"やく":{"f":"yaku_kana.opus","p":1},"えき":{"f":"eki_kana.opus","p":0},"役":{"f":"fallback.opus","p":0}}}"#
    }

    /// Parsing happens before the global slot write, so a broken payload
    /// errors deterministically regardless of which test won the slot.
    #[test]
    fn init_rejects_broken_json_before_touching_the_slot() {
        let Err(OrigaError::PitchAudioParseError { reason }) = init_pitch_audio_index("not json")
        else {
            panic!("expected a parse error");
        };
        assert!(reason.contains("Failed to parse"), "reason: {reason}");
    }

    /// Single test function for the Owned branch: the process-global slot is
    /// one-shot, so install and lookups must share one test to stay
    /// race-free (see the bail-out guard of the fallback test above).
    #[test]
    fn init_installs_the_owned_branch_for_global_lookups() {
        if is_pitch_audio_loaded() {
            return; // another test already installed an index
        }

        // Act
        init_pitch_audio_index(index_v3_json()).expect("valid index installs");

        // Assert — every free function routes through the Owned branch.
        assert!(is_pitch_audio_loaded());
        assert_eq!(pitch_audio_version(), 3);
        assert_eq!(get_audio_entry_count(), 5);

        let composite = get_audio_for_word("役|やく").expect("composite key entry");
        assert_eq!(composite.file(), "yaku.opus");

        let by_reading = get_audio_for_reading("役", "やく").expect("composite fallback");
        assert_eq!(by_reading.file(), "yaku.opus");

        let kana_only = get_audio_for_reading("NoSuchWord", "えき").expect("reading fallback");
        assert_eq!(kana_only.file(), "eki_kana.opus");

        let word_only = get_audio_for_reading("役", "NoSuchReading").expect("word fallback");
        assert_eq!(word_only.file(), "fallback.opus");

        assert!(get_audio_for_word("NoSuchEntry").is_none());
    }

    #[test]
    fn install_archived_pitch_index_reports_duplicate_install() {
        // Arrange — a slot already populated by init cannot take an archived
        // view; the error documents the one-shot contract.
        if !is_pitch_audio_loaded() {
            return; // fallback test won the race — no slot to collide with
        }
        let Err(OrigaError::PitchAudioParseError { reason }) =
            install_archived_pitch_index(dummy_archived_view())
        else {
            panic!("expected an already-initialized error");
        };
        assert!(reason.contains("already initialized"), "reason: {reason}");
    }

    /// Stands in for an archived view: the install fails on the slot guard
    /// before the reference is ever dereferenced.
    fn dummy_archived_view() -> &'static ArchivedPitchAudioIndexBlob {
        // SAFETY-free stand-in: an empty-aligned slice cannot be produced
        // without rkyv; instead reuse any leaked payload via the accessor.
        let index = crate::dictionary::pitch_audio::PitchAudioIndex::from_json(
            r#"{"v":1,"total":0,"entries":{}}"#,
        )
        .expect("valid index");
        let payload =
            crate::dictionary::pitch_audio::serialize_pitch_index_blob_to_rkyv(&index.to_blob())
                .expect("serialize");
        crate::dictionary::pitch_audio::access_pitch_index_blob(&payload).expect("access")
    }
}
