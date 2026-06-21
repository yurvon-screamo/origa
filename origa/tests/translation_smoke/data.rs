//! Phrase corpus loader for the smoke test.
//!
//! Reads `cdn/phrases/data/pNNNN.json` files directly via ``serde_json``
//! (already a normal dependency of the crate). The on-disk schema is
//! ``[{ "i": ulid, "x": japanese_text, "ru": .., "en": .. }]`` per the
//! architect's confirmation; only ``x`` is needed for tokenization.

use serde::Deserialize;

use super::bootstrap::cdn_path;

#[derive(Deserialize)]
struct PhraseRaw {
    x: String,
}

/// Loads the ``x`` (japanese text) field of every phrase in ``chunk_filename``.
///
/// Returns ``None`` when the gitignored file is absent so callers can decide
/// whether to skip the test gracefully. A malformed file still panics — bad
/// JSON in a present file is a real error.
pub fn load_phrase_texts_from_chunk(chunk_filename: &str) -> Option<Vec<String>> {
    let path = cdn_path(&["phrases", "data", chunk_filename]);
    let body = std::fs::read_to_string(&path).ok()?;
    let phrases: Vec<PhraseRaw> = serde_json::from_str(&body).unwrap_or_else(|err| {
        panic!(
            "failed to parse {}: {err} — check that the file matches the \
             [{{i, x, ru, en}}, ...] schema",
            path.display()
        )
    });
    Some(phrases.into_iter().map(|p| p.x).collect())
}
