//! CDN blob of precomputed tokenization entries.
//!
//! Payload is a `BTreeMap<String, PrecomputedEntry>` keyed by the exact
//! render strings (phrase sentences, grammar markdown text nodes). The
//! builder (`utils build-cdn-rkyv`) serializes it deterministically;
//! clients access it zero-copy and keep the archived view installed in a
//! process-global registry that [`crate::domain::tokenizer::precomputed`]
//! consults on lookup.

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::domain::OrigaError;
use crate::domain::{PartOfSpeech, PrecomputedEntry, PrecomputedToken};

use super::cdn_blob::split_blob;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PrecomputeBlob {
    pub entries: BTreeMap<String, PrecomputedEntry>,
}

/// Installed zero-copy views, one per loaded blob (a phrase chunk or the
/// grammar set).
static INSTALLED_VIEWS: RwLock<Vec<&'static ArchivedPrecomputeBlob>> = RwLock::new(Vec::new());

/// Serialize the deterministic blob form (payload of a CDN blob).
pub fn serialize_precompute_blob_to_rkyv(blob: &PrecomputeBlob) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(blob)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("failed to serialize precompute blob: {e}"),
        })
}

/// Deflates a built blob for CDN storage. The precompute payloads are
/// string-heavy (token surfaces, readings, spans) and deflate to roughly
/// a quarter of the raw size, keeping the lazy per-chunk fetch cheap.
pub fn deflate_blob(blob: &[u8]) -> Vec<u8> {
    use std::io::Write;

    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(6));
    // write_all on Vec-backed DeflateEncoder cannot fail except for OOM,
    // which aborts anyway.
    let _ = encoder.write_all(blob);
    encoder.finish().unwrap_or_default()
}

/// Inflates a deflated CDN blob (the mirror of [`deflate_blob`]).
pub fn inflate_blob(deflated: &[u8]) -> Result<Vec<u8>, OrigaError> {
    use std::io::Read;

    let mut decoder = flate2::read::DeflateDecoder::new(deflated);
    let mut blob = Vec::new();
    decoder
        .read_to_end(&mut blob)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("failed to inflate precompute blob: {e}"),
        })?;
    Ok(blob)
}

/// Inflates a deflated CDN blob and splits it: the payload is accessed
/// zero-copy, the header is returned for the caller's manifest guard.
pub fn access_precompute_deflated(
    deflated: &[u8],
) -> Result<
    (
        &'static ArchivedPrecomputeBlob,
        crate::dictionary::cdn_blob::BlobHeader,
    ),
    OrigaError,
> {
    let blob = inflate_blob(deflated)?;
    let (header, payload) = split_blob(&blob).map_err(|e| OrigaError::TokenizerError {
        reason: format!("precompute blob header invalid: {e}"),
    })?;
    let view = access_precompute_blob(payload)?;
    Ok((view, header))
}

/// Zero-copy archived view of a blob payload. The payload is copied once
/// into an immortal `AlignedVec` (checked `access` requires properly
/// aligned bytes); a validation failure leaves the leaked bytes
/// unreclaimed until reload while callers fall back to live tokenization.
///
/// Pass the payload slice from `split_blob`, not the full blob.
/// rkyv 0.8 resolves the archive root from the buffer END
/// (`root_position = size - size_of::<T>()`), so passing a header-prefixed
/// buffer accidentally works — but that is an implementation detail of
/// the current rkyv major, not a contract. The payload slice is correct
/// by construction and stays correct across rkyv bumps.
///
/// Validation depth (accepted trade-off): `access` runs `CheckBytes`,
/// which does not traverse the whole string tree, and raw deflate carries
/// no checksum — a corrupted payload may install and serve garbage
/// strings instead of failing over to live tokenization. The
/// `manifest_guard` binds the blob to its manifest SOURCES (not the
/// payload bytes), and the transport (TLS + S3) is integrity-checked end
/// to end in practice, which is why this depth is considered sufficient.
pub fn access_precompute_blob(
    payload: &[u8],
) -> Result<&'static ArchivedPrecomputeBlob, OrigaError> {
    crate::dictionary::cdn_blob::access_leaked::<ArchivedPrecomputeBlob>(payload).map_err(|e| {
        OrigaError::TokenizerError {
            reason: format!("failed to access precompute blob: {e}"),
        }
    })
}

/// Installs an accessed view into the lookup registry. Cheap duplicate
/// suppression: re-installing an already-registered view is a no-op.
pub fn install_precompute_view(view: &'static ArchivedPrecomputeBlob) {
    let mut guard = INSTALLED_VIEWS.write().unwrap_or_else(|e| e.into_inner());
    if !guard.iter().any(|existing| std::ptr::eq(*existing, view)) {
        guard.push(view);
    }
}

pub(crate) fn lookup_in_installed_views(text: &str) -> Option<PrecomputedEntry> {
    let guard = INSTALLED_VIEWS.read().unwrap_or_else(|e| e.into_inner());
    for view in guard.iter() {
        if let Some(archived) = view.entries.get(text) {
            return Some(archived_entry_to_owned(archived));
        }
    }
    None
}

pub(crate) fn clear_installed_views() {
    INSTALLED_VIEWS
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// Number of installed blob views — observability for tests and logs.
pub fn installed_precompute_views_len() -> usize {
    INSTALLED_VIEWS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .len()
}

fn archived_entry_to_owned(entry: &rkyv::Archived<PrecomputedEntry>) -> PrecomputedEntry {
    PrecomputedEntry {
        furigana_spans: entry
            .furigana_spans
            .iter()
            .map(|span| crate::domain::AnnotatedSpan {
                text: span.text.as_str().to_string(),
                reading: span.reading.as_ref().map(|r| r.as_str().to_string()),
                reading_spans: span
                    .reading_spans
                    .iter()
                    .map(|s| crate::dictionary::furigana_dict::ReadingSpan {
                        start_index: u32::from(s.start_index) as usize,
                        end_index: u32::from(s.end_index) as usize,
                        text: s.text.as_str().to_string(),
                    })
                    .collect(),
            })
            .collect(),
        tokens: entry
            .tokens
            .iter()
            .map(|token| PrecomputedToken {
                surface: token.surface.as_str().to_string(),
                base: token.base.as_str().to_string(),
                reading: token.reading.as_str().to_string(),
                pos: PartOfSpeech::from(&token.pos),
            })
            .collect(),
    }
}

/// Archived enum → owned transcription. Every variant is spelled out so
/// adding a `PartOfSpeech` variant breaks this match at compile time and
/// forces the transcription to stay complete.
impl From<&rkyv::Archived<PartOfSpeech>> for PartOfSpeech {
    fn from(archived: &rkyv::Archived<PartOfSpeech>) -> Self {
        use rkyv::Archived;

        match archived {
            Archived::<PartOfSpeech>::Verb => PartOfSpeech::Verb,
            Archived::<PartOfSpeech>::Noun => PartOfSpeech::Noun,
            Archived::<PartOfSpeech>::IAdjective => PartOfSpeech::IAdjective,
            Archived::<PartOfSpeech>::NaAdjective => PartOfSpeech::NaAdjective,
            Archived::<PartOfSpeech>::Adverb => PartOfSpeech::Adverb,
            Archived::<PartOfSpeech>::PreNounAdjectival => PartOfSpeech::PreNounAdjectival,
            Archived::<PartOfSpeech>::Conjunction => PartOfSpeech::Conjunction,
            Archived::<PartOfSpeech>::Interjection => PartOfSpeech::Interjection,
            Archived::<PartOfSpeech>::Prefix => PartOfSpeech::Prefix,
            Archived::<PartOfSpeech>::Suffix => PartOfSpeech::Suffix,
            Archived::<PartOfSpeech>::Particle => PartOfSpeech::Particle,
            Archived::<PartOfSpeech>::AuxiliaryVerb => PartOfSpeech::AuxiliaryVerb,
            Archived::<PartOfSpeech>::Pronoun => PartOfSpeech::Pronoun,
            Archived::<PartOfSpeech>::ProperNoun => PartOfSpeech::ProperNoun,
            Archived::<PartOfSpeech>::Numeral => PartOfSpeech::Numeral,
            Archived::<PartOfSpeech>::Determiner => PartOfSpeech::Determiner,
            Archived::<PartOfSpeech>::Unspecified => PartOfSpeech::Unspecified,
            Archived::<PartOfSpeech>::Other => PartOfSpeech::Other,
            Archived::<PartOfSpeech>::Symbol => PartOfSpeech::Symbol,
            Archived::<PartOfSpeech>::Whitespace => PartOfSpeech::Whitespace,
            Archived::<PartOfSpeech>::AuxiliarySymbol => PartOfSpeech::AuxiliarySymbol,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::lookup_precomputed;

    fn sample_entry() -> PrecomputedEntry {
        PrecomputedEntry {
            furigana_spans: vec![crate::domain::AnnotatedSpan {
                text: "私は学生です。".to_string(),
                reading: Some("ワタシワガクセイデス".to_string()),
                reading_spans: vec![],
            }],
            tokens: vec![PrecomputedToken {
                surface: "私".to_string(),
                base: "私".to_string(),
                reading: "ワタシ".to_string(),
                pos: PartOfSpeech::Pronoun,
            }],
        }
    }

    #[test]
    fn serialize_access_roundtrip_answers_lookups() {
        let _guard = crate::domain::tokenizer::precomputed::precomputed_test_lock_guard();
        // Arrange
        let blob = PrecomputeBlob {
            entries: BTreeMap::from([("私は学生です。".to_string(), sample_entry())]),
        };
        let payload = serialize_precompute_blob_to_rkyv(&blob).unwrap();

        // Act
        // Act: the payload is fed directly (this test serializes the raw
        // payload without a CDN header — `split_blob` is exercised by the
        // deflated loader tests).
        let view = access_precompute_blob(&payload).unwrap();
        install_precompute_view(view);
        let found = lookup_precomputed("私は学生です。");

        // Assert
        assert_eq!(found, Some(sample_entry()));
        assert!(lookup_precomputed("missing").is_none());
        clear_installed_views();
        assert!(lookup_precomputed("私は学生です。").is_none());
    }

    #[test]
    fn access_rejects_truncated_payload() {
        // Arrange
        let blob = PrecomputeBlob {
            entries: BTreeMap::new(),
        };
        let payload = serialize_precompute_blob_to_rkyv(&blob).unwrap();

        // Act
        let result = access_precompute_blob(&payload[..payload.len() / 2]);

        // Assert
        assert!(result.is_err());
    }
}
