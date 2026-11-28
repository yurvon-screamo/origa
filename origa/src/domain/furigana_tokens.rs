//! Opt-in furigana source for surfaces whose counterpart renders
//! `PrecomputedEntry::tokens` (the phrase question side). Keeping this
//! helper separate preserves the spans-first contract of `furiganize_segments`
//! and the small-module discipline of the `domain` layer.

use std::collections::HashSet;

use crate::domain::{FuriganaSegment, JapaneseChar, katakana_to_hiragana, lookup_precomputed};

/// Furigana for surfaces whose answer side renders `PrecomputedEntry::tokens`.
/// Deriving both sides from the same token array keeps one phrase at one
/// reading (uwuwu#1705). Deliberately opt-in: the general furiganize paths
/// keep their spans-first contract from `precomputed.rs` and stay untouched.
pub fn furiganize_segments_from_precomputed_tokens(
    text: &str,
    known_kanji: &HashSet<char>,
) -> Option<Vec<FuriganaSegment>> {
    let entry = lookup_precomputed(text)?;
    if entry.tokens.is_empty() {
        return None;
    }

    // Token surfaces must rebuild the exact render string: a partial
    // tokenization falls back to the caller's spans/dictionary path instead
    // of silently dropping characters from the question.
    let rebuilt: String = entry.tokens.iter().map(|t| t.surface.as_str()).collect();
    if rebuilt != text {
        return None;
    }

    Some(
        entry
            .tokens
            .iter()
            .map(|token| {
                if !token.surface.chars().any(|c| c.is_kanji()) {
                    return FuriganaSegment::new(token.surface.clone(), None, false);
                }
                let reading = if token.reading.is_empty() {
                    None
                } else {
                    Some(katakana_to_hiragana(&token.reading))
                };
                let is_known = reading.is_some()
                    && token
                        .surface
                        .chars()
                        .filter(|c| c.is_kanji())
                        .all(|c| known_kanji.contains(&c));
                FuriganaSegment::new(token.surface.clone(), reading, is_known)
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::furigana_dict::ReadingSpan;
    use crate::domain::{
        AnnotatedSpan, PartOfSpeech, PrecomputedEntry, PrecomputedToken,
        furiganize_text_precomputed, install_precomputed_entry, reset_precomputed_store,
    };

    /// The phrase question surface must read the same contextual token
    /// array as the answer surface: the JmdictFurigana lemma for 酷い is
    /// むご, but this phrase is ひどく (uwuwu#1705).
    #[test]
    fn phrase_question_prefers_precomputed_token_readings() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        install_precomputed_entry(
            "酷くない！",
            PrecomputedEntry {
                furigana_spans: vec![AnnotatedSpan {
                    text: "酷く".to_string(),
                    reading: Some("むごい".to_string()),
                    reading_spans: vec![ReadingSpan {
                        start_index: 0,
                        end_index: 0,
                        text: "むご".to_string(),
                    }],
                }],
                tokens: vec![
                    PrecomputedToken {
                        surface: "酷く".to_string(),
                        base: "酷い".to_string(),
                        reading: "ヒドク".to_string(),
                        pos: PartOfSpeech::PreNounAdjectival,
                    },
                    PrecomputedToken {
                        surface: "ない".to_string(),
                        base: "無い".to_string(),
                        reading: "ナイ".to_string(),
                        pos: PartOfSpeech::IAdjective,
                    },
                    PrecomputedToken {
                        surface: "！".to_string(),
                        base: "！".to_string(),
                        reading: "！".to_string(),
                        pos: PartOfSpeech::Symbol,
                    },
                ],
            },
        );

        let segments = furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new())
            .expect("the phrase entry carries tokens");

        assert_eq!(
            segments.iter().map(|s| s.text()).collect::<String>(),
            "酷くない！"
        );
        let kanji = segments
            .iter()
            .find(|s| s.text() == "酷く")
            .expect("token surface must be preserved");
        assert_eq!(kanji.reading(), Some("ひどく"));
        reset_precomputed_store();
    }

    /// The new helper must never change the global furiganize contract:
    /// an entry with empty spans stays a miss even when it carries tokens.
    #[test]
    fn empty_spans_with_tokens_keep_the_general_furiganize_miss() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        install_precomputed_entry(
            "酷くない！",
            PrecomputedEntry {
                furigana_spans: vec![],
                tokens: vec![PrecomputedToken {
                    surface: "酷くない！".to_string(),
                    base: "酷くない！".to_string(),
                    reading: "ヒドクナイ".to_string(),
                    pos: PartOfSpeech::PreNounAdjectival,
                }],
            },
        );

        assert!(furiganize_text_precomputed("酷くない！", &HashSet::new()).is_none());
        assert!(
            furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new()).is_some()
        );
        reset_precomputed_store();
    }

    #[test]
    fn precomputed_tokens_helper_misses_when_entry_has_no_tokens() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        install_precomputed_entry(
            "酷くない！",
            PrecomputedEntry {
                furigana_spans: vec![AnnotatedSpan {
                    text: "酷く".to_string(),
                    reading: Some("むごい".to_string()),
                    reading_spans: vec![],
                }],
                tokens: vec![],
            },
        );

        assert!(
            furiganize_segments_from_precomputed_tokens("酷くない！", &HashSet::new()).is_none()
        );
        assert!(
            furiganize_text_precomputed("酷くない！", &HashSet::new())
                .unwrap_or_default()
                .contains("むごい")
        );
        reset_precomputed_store();
    }
}
