use std::collections::HashSet;

use origa::dictionary::phrase::{
    PhraseDetail, cache_phrase_details, get_cached_phrase_detail, get_chunk_id, index_version,
    is_chunk_loaded,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use ulid::Ulid;

use crate::repository::cdn_provider;

#[expect(dead_code, reason = "lazy-load для будущих задач")]
pub async fn load_phrase_detail(phrase_id: Ulid) -> Result<PhraseDetail, OrigaError> {
    if let Some(detail) = get_cached_phrase_detail(&phrase_id) {
        return Ok(detail);
    }

    let chunk_id = get_chunk_id(&phrase_id).ok_or(OrigaError::PhraseNotFound { phrase_id })?;

    if !is_chunk_loaded(chunk_id) {
        let (_, hash) = index_version();
        let path = format!("phrases/data/p{:04}.json?v={}", chunk_id, hash);
        let cdn = cdn_provider();
        let json = cdn.fetch_text(&path).await?;
        cache_phrase_details(chunk_id, &json)?;
    }

    get_cached_phrase_detail(&phrase_id).ok_or(OrigaError::PhraseNotFound { phrase_id })
}

#[expect(dead_code, reason = "batch lazy-load для будущих задач")]
pub async fn load_phrase_details_batch(ids: &[Ulid]) -> Vec<Result<PhraseDetail, OrigaError>> {
    let mut chunks_to_load: HashSet<u32> = HashSet::new();
    for id in ids {
        if get_cached_phrase_detail(id).is_none() {
            if let Some(chunk_id) = get_chunk_id(id) {
                if !is_chunk_loaded(chunk_id) {
                    chunks_to_load.insert(chunk_id);
                }
            }
        }
    }

    let (_, hash) = index_version();
    let cdn = cdn_provider();
    for chunk_id in &chunks_to_load {
        let path = format!("phrases/data/p{:04}.json?v={}", chunk_id, hash);
        match cdn.fetch_text(&path).await {
            Ok(json) => {
                if let Err(e) = cache_phrase_details(*chunk_id, &json) {
                    tracing::warn!("Failed to cache chunk {}: {e}", chunk_id);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to load chunk {}: {e}", chunk_id);
            },
        }
    }

    ids.iter()
        .map(|id| get_cached_phrase_detail(id).ok_or(OrigaError::PhraseNotFound { phrase_id: *id }))
        .collect()
}
