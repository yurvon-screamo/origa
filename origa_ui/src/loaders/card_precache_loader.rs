use origa::dictionary::pitch_audio::get_audio_for_reading;
use origa::domain::{Card, JapaneseChar, OrigaError, StudyCard};

use leptos::prelude::{GetUntracked, Set};

use crate::loaders::precache_loader::{DownloadResult, PreCacheProgress, batch_download};
use crate::ui_components::get_reading_from_text;

fn kata_to_hira(text: &str) -> String {
    text.chars()
        .map(|c| {
            if ('\u{30A1}'..='\u{30F6}').contains(&c) {
                char::from_u32(c as u32 - 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn kanji_svg_resources(kanji_text: &str) -> Vec<String> {
    kanji_text
        .chars()
        .filter(|c| c.is_kanji())
        .flat_map(|c| {
            let kanji_str = c.to_string();
            let encoded = urlencoding::encode(&kanji_str);
            [
                format!("kanji_animations/{encoded}.svg"),
                format!("kanji_frames/{encoded}.svg"),
            ]
        })
        .collect()
}

fn vocabulary_resources(word: &str) -> Vec<String> {
    let mut resources = Vec::new();

    let reading = get_reading_from_text(word);
    let reading_hira = kata_to_hira(&reading);
    if let Some(entry) = get_audio_for_reading(word, &reading_hira) {
        resources.push(entry.cdn_path());
    }

    resources.extend(kanji_svg_resources(word));

    resources
}

fn phrase_resources(phrase_id: &ulid::Ulid) -> Vec<String> {
    vec![format!("phrases/audio/{phrase_id}.opus")]
}

fn get_card_cdn_resources(card: &StudyCard) -> Vec<String> {
    match card.card() {
        Card::Vocabulary(v) => vocabulary_resources(v.word().text()),
        Card::Phrase(p) => phrase_resources(p.phrase_id()),
        Card::Kanji(k) => kanji_svg_resources(k.kanji().text()),
        Card::Grammar(_) => vec![],
    }
}

pub fn start_card_precache(
    cards: Vec<StudyCard>,
    offline_store: crate::store::offline_bundle_store::OfflineBundleStore,
) {
    use crate::store::offline_bundle_store::CardCacheState;

    let current_state = offline_store.card_cache_state.get_untracked();
    if current_state != CardCacheState::Idle {
        tracing::info!(
            current_state = ?current_state,
            "Card pre-cache already in progress or complete, skipping re-trigger"
        );
        return;
    }

    tracing::info!(card_count = cards.len(), "Starting card pre-cache");
    offline_store.card_cache_state.set(CardCacheState::Running);
    let store_clone = offline_store.clone();

    leptos::task::spawn_local(async move {
        match precache_all_cards(&cards, move |p| store_clone.card_cache_progress.set(p)).await {
            Ok(result) => {
                tracing::info!(
                    succeeded = result.succeeded,
                    total = result.total,
                    "Card pre-cache complete"
                );
                store_clone.card_cache_state.set(CardCacheState::Complete);
            },
            Err(e) => {
                tracing::warn!(error = %e, "Card pre-cache failed, returning to Idle");
                store_clone.card_cache_state.set(CardCacheState::Idle);
            },
        }
    });
}

pub async fn precache_all_cards(
    cards: &[StudyCard],
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let all_paths: Vec<String> = cards.iter().flat_map(get_card_cdn_resources).collect();

    let mut unique_paths = all_paths;
    unique_paths.sort();
    unique_paths.dedup();

    tracing::info!(
        card_count = cards.len(),
        resource_count = unique_paths.len(),
        "Pre-caching card resources"
    );

    batch_download(unique_paths, on_progress).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kata_to_hira_converts_standard_katakana() {
        assert_eq!(kata_to_hira("ヤク"), "やく");
        assert_eq!(kata_to_hira("ア"), "あ");
        assert_eq!(kata_to_hira("ン"), "ん");
    }

    #[test]
    fn kata_to_hira_preserves_hiragana() {
        assert_eq!(kata_to_hira("やく"), "やく");
    }

    #[test]
    fn kata_to_hira_preserves_other() {
        assert_eq!(kata_to_hira("hello"), "hello");
        assert_eq!(kata_to_hira("123"), "123");
    }

    #[test]
    fn kata_to_hira_empty() {
        assert_eq!(kata_to_hira(""), "");
    }

    #[test]
    fn kanji_svg_resources_extracts_kanji_chars() {
        let resources = kanji_svg_resources("日本語");
        assert!(resources.contains(&"kanji_animations/%E6%97%A5.svg".to_string()));
        assert!(resources.contains(&"kanji_frames/%E6%97%A5.svg".to_string()));
        assert!(resources.contains(&"kanji_animations/%E6%9C%AC.svg".to_string()));
        assert!(resources.contains(&"kanji_frames/%E6%9C%AC.svg".to_string()));
        assert!(resources.contains(&"kanji_animations/%E8%AA%9E.svg".to_string()));
        assert!(resources.contains(&"kanji_frames/%E8%AA%9E.svg".to_string()));
    }

    #[test]
    fn kanji_svg_resources_skips_hiragana() {
        let resources = kanji_svg_resources("ねこ");
        assert!(resources.is_empty());
    }

    #[test]
    fn kanji_svg_resources_skips_katakana() {
        let resources = kanji_svg_resources("ネコ");
        assert!(resources.is_empty());
    }
}
