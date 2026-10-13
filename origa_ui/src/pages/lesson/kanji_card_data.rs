use crate::ui_components::ReadingItem;
use origa::domain::{Card as DomainCard, NativeLanguage};
use tracing::warn;

use super::kanji_card_details::RadicalDisplay;

/// Kanji-side data shared by every kanji-rendering lesson surface: the
/// symbol, frequency-ordered readings, radicals and example words.
/// Previously each view repeated the same four extraction blocks.
pub(crate) struct KanjiCardData {
    pub symbol: String,
    pub on_readings: Option<Vec<ReadingItem>>,
    pub kun_readings: Option<Vec<ReadingItem>>,
    pub radicals: Option<Vec<RadicalDisplay>>,
    pub example_words: Option<Vec<(String, String)>>,
}

/// Extracts kanji-side data. Returns `None` for non-kanji cards; lookup
/// failures inside a kanji card degrade the individual field to `None`
/// with a `warn!` (log-only degradation, the card stays renderable).
pub(crate) fn extract_kanji_card_data(
    card: &DomainCard,
    native_language: NativeLanguage,
) -> Option<KanjiCardData> {
    let DomainCard::Kanji(kanji) = card else {
        return None;
    };

    let symbol = kanji.kanji().text().to_string();

    let collect_readings = |readings: Vec<(String, u32, bool)>| {
        (!readings.is_empty()).then(|| {
            readings
                .into_iter()
                .map(|(reading, freq, is_rare)| ReadingItem {
                    reading,
                    freq,
                    is_rare,
                })
                .collect::<Vec<_>>()
        })
    };
    let on_readings = collect_readings(kanji.on_readings_with_freq());
    let kun_readings = collect_readings(kanji.kun_readings_with_freq());

    let radicals = match kanji.radicals_info() {
        Ok(infos) => Some(
            infos
                .iter()
                .map(|info| RadicalDisplay {
                    symbol: info.radical(),
                    name: info.name(&native_language).to_string(),
                    description: info.description(&native_language).to_string(),
                })
                .collect::<Vec<_>>(),
        ),
        Err(e) => {
            warn!(kanji = %symbol, error = ?e, "Failed to get radicals for kanji");
            None
        },
    };

    let example_words = {
        let examples: Vec<_> = kanji
            .example_words(&native_language)
            .iter()
            .map(|entry| (entry.word().to_string(), entry.meaning().to_string()))
            .collect();
        (!examples.is_empty()).then_some(examples)
    };

    Some(KanjiCardData {
        symbol,
        on_readings,
        kun_readings,
        radicals,
        example_words,
    })
}
