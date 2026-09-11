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
