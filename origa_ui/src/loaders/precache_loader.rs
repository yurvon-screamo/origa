use std::sync::Arc;

use futures::stream::{self, StreamExt};
use origa::dictionary::kanji::get_all_kanji;
use origa::dictionary::phrase::index_version;
use origa::domain::OrigaError;

use crate::repository::cdn_provider;

const BUNDLE_DOWNLOADED_KEY: &str = "/__origa_bundle_downloaded__";
const CONCURRENCY: usize = 20;

#[derive(Clone, Default)]
pub struct PreCacheProgress {
    pub completed: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Clone, Default)]
pub struct DownloadResult {
    pub total: usize,
    pub succeeded: usize,
}

pub fn get_base_bundle_resources() -> Vec<String> {
    let mut resources: Vec<String> = Vec::new();

    resources.extend([
        "dictionaries/char_def.bin".to_string(),
        "dictionaries/matrix.mtx".to_string(),
        "dictionaries/dict.da".to_string(),
        "dictionaries/dict.vals".to_string(),
        "dictionaries/unk.bin".to_string(),
        "dictionaries/dict.wordsidx".to_string(),
        "dictionaries/dict.words".to_string(),
        "dictionaries/metadata.json".to_string(),
        "dictionaries/JmdictFurigana.txt".to_string(),
    ]);

    for i in 1..=11 {
        resources.push(format!("dictionary/chunk_{:02}.json", i));
    }

    resources.extend([
        "dictionary/kanji.json".to_string(),
        "dictionary/radicals.json".to_string(),
    ]);

    resources.push("grammar/grammar.json".to_string());
    resources.push("phrases/phrase_index.json".to_string());

    let (_, hash) = index_version();
    for i in 0..=197 {
        resources.push(format!("phrases/data/p{:04}.json?v={}", i, hash));
    }

    resources.push("pitch/index.json".to_string());

    resources.extend([
        "well_known_set/well_known_sets_meta.json".to_string(),
        "well_known_set/well_known_types_meta.json".to_string(),
        "well_known_set/jlpt_n1.json".to_string(),
        "well_known_set/jlpt_n2.json".to_string(),
        "well_known_set/jlpt_n3.json".to_string(),
        "well_known_set/jlpt_n4.json".to_string(),
        "well_known_set/jlpt_n5.json".to_string(),
    ]);

    for kanji in get_all_kanji() {
        let kanji_str = kanji.to_string();
        let encoded = urlencoding::encode(&kanji_str);
        resources.push(format!("kanji_animations/{}.svg", encoded));
        resources.push(format!("kanji_frames/{}.svg", encoded));
    }

    resources
}

pub async fn batch_download(
    paths: Vec<String>,
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let total = paths.len();
    let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let succeeded = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    stream::iter(paths)
        .map(|path| {
            let completed = completed.clone();
            let succeeded = succeeded.clone();
            let on_progress = on_progress.clone();
            async move {
                match cdn_provider::prefetch_to_cache(&path).await {
                    Ok(()) => {
                        succeeded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    },
                    Err(e) => {
                        tracing::warn!(path = %path, error = ?e, "Failed to prefetch");
                    },
                }
                let done = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                on_progress(PreCacheProgress {
                    completed: done,
                    total,
                    current_file: path,
                });
            }
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<()>>()
        .await;

    Ok(DownloadResult {
        total,
        succeeded: succeeded.load(std::sync::atomic::Ordering::Relaxed),
    })
}

async fn mark_bundle_downloaded() -> Result<(), OrigaError> {
    cdn_provider::store_cache_marker(BUNDLE_DOWNLOADED_KEY, "ok").await
}

pub async fn is_bundle_downloaded() -> bool {
    cdn_provider::is_cached(BUNDLE_DOWNLOADED_KEY).await
}

pub async fn precache_base_bundle(
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let resources = get_base_bundle_resources();
    tracing::info!("Starting base bundle download: {} files", resources.len());
    let result = batch_download(resources, on_progress).await?;

    let success_rate = if result.total > 0 {
        result.succeeded as f64 / result.total as f64
    } else {
        0.0
    };

    if success_rate >= 0.95 {
        mark_bundle_downloaded().await?;
        tracing::info!(
            "Bundle downloaded: {}/{} succeeded",
            result.succeeded,
            result.total
        );
    } else {
        tracing::warn!(
            "Bundle download incomplete: {}/{} succeeded",
            result.succeeded,
            result.total
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_bundle_includes_dictionaries() {
        let resources = get_base_bundle_resources();

        assert!(resources.contains(&"dictionaries/char_def.bin".to_string()));
        assert!(resources.contains(&"dictionaries/matrix.mtx".to_string()));
        assert!(resources.contains(&"dictionaries/JmdictFurigana.txt".to_string()));
        assert!(resources.contains(&"dictionary/kanji.json".to_string()));
        assert!(resources.contains(&"dictionary/radicals.json".to_string()));
        assert!(resources.contains(&"grammar/grammar.json".to_string()));
        assert!(resources.contains(&"phrases/phrase_index.json".to_string()));
        assert!(resources.contains(&"pitch/index.json".to_string()));
        assert!(resources.contains(&"well_known_set/jlpt_n5.json".to_string()));
    }

    #[test]
    fn base_bundle_includes_vocabulary_chunks() {
        let resources = get_base_bundle_resources();
        for i in 1..=11 {
            assert!(resources.contains(&format!("dictionary/chunk_{:02}.json", i)));
        }
    }

    #[test]
    fn base_bundle_includes_well_known_sets() {
        let resources = get_base_bundle_resources();
        assert!(resources.contains(&"well_known_set/well_known_sets_meta.json".to_string()));
        assert!(resources.contains(&"well_known_set/well_known_types_meta.json".to_string()));
        for level in ["n1", "n2", "n3", "n4", "n5"] {
            assert!(resources.contains(&format!("well_known_set/jlpt_{}.json", level)));
        }
    }
}
