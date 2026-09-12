//! Shared helpers for the tokenization precompute builders (#521):
//! source hashing, furigana-dictionary bootstrap and the entry
//! derivation itself. Kept in one place so the phrase and grammar
//! builders cannot drift apart as the pipeline evolves.

use std::fs;
use std::path::Path;

use origa::dictionary::furigana_dict::{build_furigana_dict_from_text, set_furigana_dict};
use origa::domain::{
    JapaneseText, OrigaError, PrecomputedEntry, PrecomputedToken, SUDACHIDICT_DIR, annotate_text,
    tokenize_text,
};
use sha2::{Digest, Sha256};

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    let digest: [u8; 32] = Sha256::digest(data).into();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

pub(crate) fn sha256_raw(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Derives the precompute entry for one exact render string. Kanji-free
/// strings are skipped: the runtime fast path answers them without any
/// dictionary, so precomputing them would only bloat the blob.
pub(crate) fn precompute_entry(text: &str) -> Option<PrecomputedEntry> {
    if !text.contains_kanji() {
        return None;
    }
    let tokens: Vec<PrecomputedToken> = tokenize_text(text)
        .ok()?
        .iter()
        .map(PrecomputedToken::from_token_info)
        .collect();
    let furigana_spans = annotate_text(text).ok()?;
    Some(PrecomputedEntry {
        furigana_spans,
        tokens,
    })
}

/// Installs the furigana dictionary the annotator needs and returns the
/// tokenizer "ingredients" bytes that blob freshness binds to:
/// `|sudachidict:<dir>|furigana:<sha256(JmdictFurigana.txt)>`.
///
/// The SudachiDict ingredient is the directory NAME, not a content hash:
/// the project convention is that every dictionary rebuild ships under a
/// bumped versioned directory (see `SUDACHIDICT_DIR` sync notes), so a
/// name change is the rebuild signal. A rebuild that keeps the name
/// silently leaves blobs stale — the asymmetry with the honest furigana
/// content hash is deliberate and documented here.
pub(crate) fn init_furigana_and_dictionary_ingredients(
    furigana_source_path: &Path,
) -> Result<Vec<u8>, OrigaError> {
    let furigana_source =
        fs::read(furigana_source_path).map_err(|e| OrigaError::RepositoryError {
            reason: format!("failed to read {}: {e}", furigana_source_path.display()),
        })?;
    let furigana_text =
        String::from_utf8(furigana_source.clone()).map_err(|e| OrigaError::RepositoryError {
            reason: format!("{} is not valid UTF-8: {e}", furigana_source_path.display()),
        })?;
    set_furigana_dict(build_furigana_dict_from_text(&furigana_text)?).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("furigana dictionary already loaded: {e:?}"),
        }
    })?;

    let mut ingredients = Vec::new();
    ingredients.extend_from_slice(b"|sudachidict:");
    ingredients.extend_from_slice(SUDACHIDICT_DIR.as_bytes());
    ingredients.extend_from_slice(b"|furigana:");
    ingredients.extend_from_slice(&sha256_raw(&furigana_source));
    Ok(ingredients)
}
