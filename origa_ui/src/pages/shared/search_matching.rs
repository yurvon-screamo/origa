//! Search matching of the card list pages (#837).
//!
//! [`default_card_matches`] is the shared question/answer matching every
//! list page falls back to; page matchers compose it with page-specific
//! arms. The words page adds kana readings (resolved through the furigana
//! dictionary — the same source the card furigana renders from, so what
//! the user sees above the kanji is what the search matches), the kanji
//! page adds on/kun readings.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use leptos::prelude::{Get, RwSignal};
use origa::dictionary::furigana_dict::get_furigana_dict;
use origa::domain::{
    Card, CardAnswer, JapaneseChar, NativeLanguage, StudyCard, katakana_to_hiragana,
};

use super::card_list_page::SearchMatcher;

/// Folds katakana to hiragana so a query typed in either kana script
/// matches readings and surfaces written in the other (タベル finds 食べる).
fn normalize_kana(text: &str) -> String {
    katakana_to_hiragana(text)
}

fn query_has_kana(query: &str) -> bool {
    query.chars().any(|c| c.is_hiragana() || c.is_katakana())
}

/// The default matching of `card_list_view`: the question surface and the
/// answer texts (translations, description). Extracted from the memo so
/// page matchers compose it instead of forking the logic. `query` arrives
/// lowercased and trimmed.
pub(crate) fn default_card_matches(card: &StudyCard, query: &str, lang: NativeLanguage) -> bool {
    let card_inner = card.card();
    let matches_question = card_inner
        .question(&lang)
        .ok()
        .is_some_and(|q| q.text().to_lowercase().contains(query));

    matches_question || answer_matches(card_inner.answer(&lang).ok().as_ref(), query)
}

pub(crate) fn answer_matches(answer: Option<&CardAnswer>, query: &str) -> bool {
    match answer {
        Some(CardAnswer::Vocabulary {
            translations,
            description,
        }) => {
            translations
                .iter()
                .any(|t| t.to_lowercase().contains(query))
                || description
                    .as_ref()
                    .is_some_and(|d| d.to_lowercase().contains(query))
        },
        Some(CardAnswer::Text(s)) => s.to_lowercase().contains(query),
        // Structured nuances carry no flat text projection; same behavior
        // the inline memo matcher had before the extraction.
        Some(CardAnswer::GrammarNuances { .. }) | None => false,
    }
}

/// Resolved readings of a word, cached per word for the session. The
/// furigana dictionary is immutable once loaded, so an empty result is a
/// stable miss and safe to cache too.
static READING_CACHE: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();

fn reading_cache() -> &'static Mutex<HashMap<String, Vec<String>>> {
    READING_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Hiragana-normalized readings of a vocabulary word via the furigana
/// dictionary's exact-word lookup (cheap lookups over the immutable rkyv
/// store — no tokenization on any path). Words the dictionary does not
/// know degrade to surface/translation matching. While the dictionary is
/// still loading the result is empty and NOT cached, so a later keystroke
/// retries.
pub(crate) fn vocabulary_readings(word: &str) -> Vec<String> {
    let mut cache = reading_cache().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(readings) = cache.get(word) {
        return readings.clone();
    }
    let Some(dict) = get_furigana_dict() else {
        return Vec::new();
    };
    let readings: Vec<String> = dict
        .lookup_word(word)
        .iter()
        .map(|entry| normalize_kana(&entry.reading))
        .collect();
    cache.insert(word.to_string(), readings.clone());
    readings
}

/// Words-page matcher (#837): the default arms plus the word's surface
/// and readings matched kana-insensitively (hiragana query finds 食べる by
/// たべる, katakana words by their hiragana spelling).
///
/// `furigana_ready` is the store's dictionary-readiness signal: reading
/// it inside the matcher registers a reactive dependency, so a query
/// typed before the furigana dictionary finished loading recomputes
/// (and starts matching) the moment the dictionary lands.
pub(crate) fn vocabulary_search_matcher(furigana_ready: RwSignal<bool>) -> SearchMatcher {
    Arc::new(move |card: &StudyCard, query: &str, lang: NativeLanguage| {
        if default_card_matches(card, query, lang) {
            return true;
        }
        let Card::Vocabulary(vocab) = card.card() else {
            return false;
        };
        // Reading search only makes sense for kana queries; Latin input
        // never pays for the reading arm.
        if !query_has_kana(query) {
            return false;
        }
        let normalized_query = normalize_kana(query);
        // The normalized surface is dictionary-independent: katakana
        // words answer hiragana queries right away.
        if normalize_kana(vocab.word().text()).contains(&normalized_query) {
            return true;
        }
        // Readings need the furigana dictionary; the readiness read
        // registers a reactive dependency, so a query typed while it
        // still loads re-runs once the dictionary lands.
        if !furigana_ready.get() {
            return false;
        }
        vocabulary_readings(vocab.word().text())
            .iter()
            .any(|reading| reading.contains(&normalized_query))
    })
}

/// Strips JMdict reading separators — the okurigana dot (`た.べる`) and
/// the compound dash (`ひと-り`) — so a plain kana query matches
/// dictionary-form readings.
fn strip_reading_separators(reading: &str) -> String {
    reading.chars().filter(|c| *c != '.' && *c != '-').collect()
}

/// Kana-insensitive match of a query against JMdict-style readings.
/// Shared by the kanji list page matcher and the add-kanji drawer.
pub(crate) fn readings_match(readings: &[String], query: &str) -> bool {
    let normalized_query = normalize_kana(query);
    if normalized_query.is_empty() {
        return false;
    }
    readings.iter().any(|reading| {
        strip_reading_separators(&normalize_kana(reading)).contains(&normalized_query)
    })
}

/// Kanji-page matcher (#837): the default arms plus on/kun readings, so
/// げん finds 厳 (on ゲン) and きび finds its kun readings.
///
/// `kanji_ready` mirrors [`vocabulary_search_matcher`]'s readiness
/// parameter: readings resolve through the kanji dictionary, and the
/// reactive read re-runs the search once that dictionary lands.
pub(crate) fn kanji_search_matcher(kanji_ready: RwSignal<bool>) -> SearchMatcher {
    Arc::new(move |card: &StudyCard, query: &str, lang: NativeLanguage| {
        if default_card_matches(card, query, lang) {
            return true;
        }
        let Card::Kanji(kanji) = card.card() else {
            return false;
        };
        if !query_has_kana(query) || !kanji_ready.get() {
            return false;
        }
        readings_match(&kanji.on_readings(), query) || readings_match(&kanji.kun_readings(), query)
    })
}
