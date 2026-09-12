use std::collections::HashSet;

use crate::dictionary::furigana_dict::{
    FuriganaStore, ReadingSpan, get_furigana_dict, is_furigana_dict_loaded,
};
use crate::domain::JapaneseText;
use crate::domain::furigana_annotator::AnnotatedSpan;
use crate::domain::hiragana_to_katakana;
use crate::domain::{
    OrigaError, japanese::JapaneseChar, lookup_precomputed, tokenizer::tokenize_text,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuriganaSegment {
    text: String,
    reading: Option<String>,
    is_known: bool,
}

impl FuriganaSegment {
    pub fn new(text: String, reading: Option<String>, is_known: bool) -> Self {
        Self {
            text,
            reading,
            is_known,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn reading(&self) -> Option<&str> {
        self.reading.as_deref()
    }

    pub fn has_reading(&self) -> bool {
        self.reading.is_some()
    }

    pub fn is_known(&self) -> bool {
        self.is_known
    }
}

pub fn furiganize_segments(
    text: &str,
    known_kanji: &HashSet<char>,
) -> Result<Vec<FuriganaSegment>, OrigaError> {
    // Kanji-free fast path: furigana attaches only to kanji, so texts
    // without kanji (phrase translations, Latin fragments) render as one
    // plain segment without touching any dictionary.
    if !text.contains_kanji() {
        return if text.trim().is_empty() {
            Ok(Vec::new())
        } else {
            Ok(vec![FuriganaSegment::new(text.to_string(), None, false)])
        };
    }

    // Precomputed fast path: the store is keyed by this exact render
    // string. Entries with empty spans serve token translations only —
    // see the `precomputed` module contract — and must not shadow the
    // live paths below.
    if let Some(entry) = lookup_precomputed(text)
        && !entry.furigana_spans.is_empty()
    {
        return Ok(spans_to_segments(entry.furigana_spans, known_kanji));
    }

    // Single-word fast path: card words are dictionary lemmas, so the
    // furigana dictionary answers them directly — no tokenizer needed.
    if let Some(dict) = get_furigana_dict()
        && let Some(span) = annotate_single_word(text, dict)
    {
        return Ok(spans_to_segments(vec![span], known_kanji));
    }

    if is_furigana_dict_loaded() {
        let spans = crate::domain::furigana_annotator::annotate_text(text)?;
        return Ok(spans_to_segments(spans, known_kanji));
    }

    let tokens = tokenize_text(text).map_err(|e| OrigaError::FuriganaError {
        reason: e.to_string(),
    })?;
    Ok(tokens
        .into_iter()
        .map(|token| token_to_furigana_segment(token, known_kanji))
        .collect())
}

/// Resolves the furigana of a whole string treated as one dictionary
/// word. Mirrors the `Single` branch of the annotator's
/// `resolve_annotation` (katakana-normalized reading, span layout), but
/// skips tokenization: the caller guarantees the text is already a lemma.
fn annotate_single_word(text: &str, dict: &FuriganaStore) -> Option<AnnotatedSpan> {
    let best = dict.lookup_word(text).into_iter().next()?;
    Some(AnnotatedSpan {
        text: text.to_string(),
        reading: Some(hiragana_to_katakana(&best.reading)),
        reading_spans: best
            .reading_spans
            .iter()
            .map(|span| ReadingSpan {
                start_index: span.start_index,
                end_index: span.end_index,
                text: hiragana_to_katakana(&span.text),
            })
            .collect(),
    })
}

fn token_to_furigana_segment(
    token: crate::domain::tokenizer::TokenInfo,
    known_kanji: &HashSet<char>,
) -> FuriganaSegment {
    let surface = token.orthographic_surface_form().to_string();
    let contains_kanji = surface.chars().any(|c| c.is_kanji());

    if contains_kanji {
        let all_kanji_known = surface
            .chars()
            .filter(|c| c.is_kanji())
            .all(|c| known_kanji.contains(&c));

        let reading = token.phonological_surface_form();
        let reading = if reading.is_empty() {
            None
        } else {
            Some(reading.to_string())
        };
        FuriganaSegment::new(surface, reading, all_kanji_known)
    } else {
        FuriganaSegment::new(surface, None, false)
    }
}

fn spans_to_segments(
    spans: Vec<AnnotatedSpan>,
    known_kanji: &HashSet<char>,
) -> Vec<FuriganaSegment> {
    spans
        .into_iter()
        .flat_map(|span| {
            let has_kanji = span.text.chars().any(|c| c.is_kanji());
            if span.reading_spans.is_empty() || !has_kanji {
                let reading = if has_kanji { span.reading } else { None };
                let is_known = reading.is_some()
                    && span
                        .text
                        .chars()
                        .filter(|c| c.is_kanji())
                        .all(|c| known_kanji.contains(&c));
                vec![FuriganaSegment::new(span.text, reading, is_known)]
            } else {
                apply_reading_spans(&span.text, &span.reading_spans, known_kanji)
            }
        })
        .collect()
}

fn apply_reading_spans(
    text: &str,
    spans: &[ReadingSpan],
    known_kanji: &HashSet<char>,
) -> Vec<FuriganaSegment> {
    let chars: Vec<char> = text.chars().collect();

    // Reading spans come from the JmdictFurigana entry matched by lemma, but
    // they are laid over the token surface. With SudachiDict the surface can
    // be shorter than the dictionary word (and indices are char-based there),
    // so an out-of-range span used to panic mid-poll and kill the wasm
    // executor. Any span that does not fit the surface exactly is dropped and
    // the reading is attached to the whole text instead.
    let spans_fit = !spans.is_empty()
        && spans
            .iter()
            .all(|s| s.end_index < chars.len() && s.start_index <= s.end_index);
    if !spans_fit {
        if let Some(first) = spans.first() {
            return vec![FuriganaSegment::new(
                text.to_string(),
                Some(first.text.clone()),
                false,
            )];
        }
        return vec![FuriganaSegment::new(text.to_string(), None, false)];
    }

    let mut segments = Vec::new();
    let mut last_end: usize = 0;

    for span in spans {
        let start = span.start_index;
        let end = span.end_index;

        if start > last_end {
            let gap: String = chars[last_end..start].iter().collect();
            if !gap.is_empty() {
                segments.push(FuriganaSegment::new(gap, None, false));
            }
        }

        let base_text: String = chars[start..=end].iter().collect();
        let is_known = base_text
            .chars()
            .filter(|c| c.is_kanji())
            .all(|c| known_kanji.contains(&c));
        segments.push(FuriganaSegment::new(
            base_text,
            Some(span.text.clone()),
            is_known,
        ));

        last_end = end + 1;
    }

    if last_end < chars.len() {
        let tail: String = chars[last_end..].iter().collect();
        if !tail.is_empty() {
            segments.push(FuriganaSegment::new(tail, None, false));
        }
    }

    segments
}

fn html_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

pub fn furiganize_text_html(segments: &[FuriganaSegment]) -> String {
    segments
        .iter()
        .map(|seg| match &seg.reading {
            Some(reading) => {
                let class = if seg.is_known { "furigana-hidden furigana-ruby" } else { "furigana-ruby" };
                format!(
                    "<ruby class=\"{}\">{}<rp>(</rp><rt class=\"furigana-rt\">{}</rt><rp>)</rp></ruby>",
                    class, html_escape(&seg.text), html_escape(reading)
                )
            }
            None => html_escape(&seg.text),
        })
        .collect()
}

pub fn furiganize_text(text: &str, known_kanji: &HashSet<char>) -> Result<String, OrigaError> {
    let segments = furiganize_segments(text, known_kanji)?;
    Ok(furiganize_text_html(&segments))
}

/// Precompute-path twin of [`furiganize_text`]: renders the store's
/// furigana spans for exactly this string. Returns `None` when the store
/// has no usable entry (no entry, or empty spans per the `precomputed`
/// contract) — the caller then falls back to the live path.
pub fn furiganize_text_precomputed(text: &str, known_kanji: &HashSet<char>) -> Option<String> {
    let entry = lookup_precomputed(text)?;
    if entry.furigana_spans.is_empty() {
        return None;
    }
    Some(furiganize_text_html(&spans_to_segments(
        entry.furigana_spans,
        known_kanji,
    )))
}

#[cfg(test)]
mod tests {
    use std::{env, fs, io::Read, path::PathBuf};

    use flate2::read::DeflateDecoder;

    use super::*;
    use crate::domain::{
        DictionaryData, PrecomputedEntry, init_dictionary, install_precomputed_entry,
        is_dictionary_loaded, reset_precomputed_store,
    };

    fn decompress(data: Vec<u8>) -> Vec<u8> {
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    }

    fn ensure_dictionary() {
        if is_dictionary_loaded() {
            return;
        }

        let base = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionaries");
        let dict_dir = base.join(crate::domain::SUDACHIDICT_DIR);

        let read_file = |name: &str| fs::read(dict_dir.join(name)).unwrap();

        let data = DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_trie: decompress(read_file("dict.trie")),
            dict_vals_idx: decompress(read_file("dict.valsidx")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        };

        init_dictionary(data).unwrap();
    }

    #[test]
    fn should_create_segment_with_reading() {
        let segment = FuriganaSegment::new("食べ".to_string(), Some("タベ".to_string()), false);
        assert_eq!(segment.text(), "食べ");
        assert_eq!(segment.reading(), Some("タベ"));
        assert!(segment.has_reading());
        assert!(!segment.is_known());
    }

    #[test]
    fn should_create_segment_without_reading() {
        let segment = FuriganaSegment::new("たべ".to_string(), None, false);
        assert_eq!(segment.text(), "たべ");
        assert_eq!(segment.reading(), None);
        assert!(!segment.has_reading());
    }

    #[test]
    fn should_create_segment_with_known_kanji() {
        let segment = FuriganaSegment::new("食".to_string(), Some("ショク".to_string()), true);
        assert!(segment.is_known());
    }

    #[test]
    fn should_furiganize_kanji_word_with_reading() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();
        assert!(!segments.is_empty());
        assert!(segments.iter().any(|s| s.has_reading()));
    }

    #[test]
    fn should_furiganize_hiragana_without_reading() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("たべもの", &known_kanji).unwrap();
        assert!(!segments.is_empty());
        assert!(segments.iter().all(|s| !s.has_reading()));
    }

    // Standalone kanji that Lindera classifies as Unknown (no UniDic lemma on
    // its own — e.g. 璃 in character names) carries no derivable kana reading,
    // so the furigana segment must surface `reading == None` rather than
    // `Some("")`. An empty-string reading would render `<rt></rt>` and lie via
    // `has_reading() == true`.
    #[test]
    fn should_furiganize_unknown_kanji_without_empty_reading() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        // 杏璃 is a single token in SudachiDict now; 蕗 alone is not a lemma,
        // so a name ending in it takes the unknown-kanji path.
        let segments = furiganize_segments("杏蕩", &known_kanji).unwrap();
        let ri = segments
            .iter()
            .find(|s| s.text().contains("蕩"))
            .expect("unknown-kanji segment should exist");
        assert_eq!(ri.reading(), None);
        assert!(
            !ri.has_reading(),
            "unknown kanji must not carry an empty-string reading, got {:?}",
            ri.reading()
        );
    }

    #[test]
    fn should_furiganize_mixed_text() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("食べます", &known_kanji).unwrap();
        assert!(!segments.is_empty());
    }

    #[test]
    fn should_furiganize_non_japanese_text() {
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("hello", &known_kanji).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "hello");
        assert!(!segments[0].has_reading());
    }

    #[test]
    fn should_furiganize_mixed_japanese_and_ascii() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("hello食べ物world", &known_kanji).unwrap();
        assert!(!segments.is_empty());
        assert!(
            segments
                .iter()
                .any(|s| s.text() == "hello" && !s.has_reading())
        );
        assert!(
            segments
                .iter()
                .any(|s| s.text() == "world" && !s.has_reading())
        );
    }

    #[test]
    fn should_generate_html_for_segment_with_reading() {
        let segments = vec![FuriganaSegment::new(
            "食".to_string(),
            Some("ショク".to_string()),
            false,
        )];
        let html = furiganize_text_html(&segments);
        assert_eq!(
            html,
            "<ruby class=\"furigana-ruby\">食<rp>(</rp><rt class=\"furigana-rt\">ショク</rt><rp>)</rp></ruby>"
        );
    }

    #[test]
    fn should_generate_html_for_segment_without_reading() {
        let segments = vec![FuriganaSegment::new("たべ".to_string(), None, false)];
        let html = furiganize_text_html(&segments);
        assert_eq!(html, "たべ");
    }

    #[test]
    fn should_generate_html_for_mixed_segments() {
        let segments = vec![
            FuriganaSegment::new("食".to_string(), Some("ショク".to_string()), false),
            FuriganaSegment::new("べ".to_string(), None, false),
        ];
        let html = furiganize_text_html(&segments);
        assert_eq!(
            html,
            "<ruby class=\"furigana-ruby\">食<rp>(</rp><rt class=\"furigana-rt\">ショク</rt><rp>)</rp></ruby>べ"
        );
    }

    #[test]
    fn should_generate_html_with_hidden_furigana_for_known_kanji() {
        let segments = vec![FuriganaSegment::new(
            "食".to_string(),
            Some("ショク".to_string()),
            true,
        )];
        let html = furiganize_text_html(&segments);
        assert_eq!(
            html,
            "<ruby class=\"furigana-hidden furigana-ruby\">食<rp>(</rp><rt class=\"furigana-rt\">ショク</rt><rp>)</rp></ruby>"
        );
    }

    #[test]
    fn should_furiganize_text_backwards_compatible() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let result = furiganize_text("食べ物", &known_kanji).unwrap();
        assert!(result.contains("<ruby"));
        assert!(result.contains("<rt class=\"furigana-rt\">"));
    }

    #[test]
    fn kanji_free_text_renders_one_plain_segment_without_dictionaries() {
        let known_kanji: HashSet<char> = HashSet::new();

        let segments = furiganize_segments("hello world", &known_kanji).unwrap();
        let reading = furiganize_segments("Привет мир", &known_kanji).unwrap();

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "hello world");
        assert!(!segments[0].has_reading());
        assert_eq!(reading.len(), 1);
        assert_eq!(reading[0].text(), "Привет мир");
    }

    #[test]
    fn empty_text_returns_no_segments_without_dictionaries() {
        let known_kanji: HashSet<char> = HashSet::new();
        assert!(furiganize_segments("", &known_kanji).unwrap().is_empty());
        assert!(furiganize_segments("   ", &known_kanji).unwrap().is_empty());
    }

    #[test]
    fn precomputed_hit_answers_without_any_dictionary_loaded() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Arrange: no lindera dictionary and no furigana dictionary are
        // guaranteed here — the only way this call can return Ok with a
        // reading is the precomputed store hit.
        let known_kanji: HashSet<char> = HashSet::new();
        install_precomputed_entry(
            "食べ物",
            PrecomputedEntry {
                furigana_spans: vec![AnnotatedSpan {
                    text: "食べ物".to_string(),
                    reading: Some("テストヨミ".to_string()),
                    reading_spans: vec![],
                }],
                tokens: vec![],
            },
        );

        // Act
        let segments = furiganize_segments("食べ物", &known_kanji);

        // Assert
        let segments = segments.expect("precompute must answer without dictionaries");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].reading(), Some("テストヨミ"));
        reset_precomputed_store();
    }

    #[test]
    fn precomputed_hit_hides_furigana_for_known_kanji() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Arrange
        install_precomputed_entry(
            "食べ物",
            PrecomputedEntry {
                furigana_spans: vec![AnnotatedSpan {
                    text: "食べ物".to_string(),
                    reading: Some("タベモノ".to_string()),
                    reading_spans: vec![],
                }],
                tokens: vec![],
            },
        );
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        known_kanji.insert('物');

        // Act
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        // Assert: is_known is resolved at render time from known_kanji,
        // so the same precomputed entry keeps tracking the user's progress.
        assert!(segments[0].is_known());
        reset_precomputed_store();
    }

    /// Serializes tests that mutate the global precomputed store: parallel
    /// install/reset of the same slot is a race otherwise. The lock lives
    /// in the `precomputed` module so every suite shares one mutex.
    #[test]
    fn precomputed_entry_without_spans_does_not_shadow_live_paths() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Arrange: entries with empty spans serve token translations only
        // and must fall through to the live furigana paths.
        install_precomputed_entry(
            "食べ物",
            PrecomputedEntry {
                furigana_spans: vec![],
                tokens: vec![],
            },
        );
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();

        // Act
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        // Assert
        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty(), "render must not be empty");
        assert!(kanji_segments.iter().all(|s| s.has_reading()));
        assert!(furiganize_text_precomputed("食べ物", &known_kanji).is_none());
        reset_precomputed_store();
    }

    #[test]
    fn furiganize_text_precomputed_renders_ruby_from_store() {
        let _guard = crate::domain::tokenizer::precomputed::STORE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Arrange
        install_precomputed_entry(
            "食べ物",
            PrecomputedEntry {
                furigana_spans: vec![AnnotatedSpan {
                    text: "食べ物".to_string(),
                    reading: Some("タベモノ".to_string()),
                    reading_spans: vec![],
                }],
                tokens: vec![],
            },
        );
        let known_kanji: HashSet<char> = HashSet::new();

        // Act
        let html = furiganize_text_precomputed("食べ物", &known_kanji);

        // Assert
        assert_eq!(
            html.as_deref(),
            Some(
                "<ruby class=\"furigana-ruby\">食べ物<rp>(</rp><rt class=\"furigana-rt\">タベモノ</rt><rp>)</rp></ruby>"
            )
        );
        assert!(furiganize_text_precomputed("missing", &known_kanji).is_none());
        reset_precomputed_store();
    }

    #[test]
    fn should_show_furigana_when_only_partial_kanji_known() {
        ensure_dictionary();
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');

        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty());
        assert!(kanji_segments.iter().any(|s| s.has_reading()));
        assert!(kanji_segments.iter().any(|s| !s.is_known()));
    }

    #[test]
    fn should_show_furigana_when_no_kanji_known() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();

        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty());
        assert!(kanji_segments.iter().all(|s| s.has_reading()));
        assert!(kanji_segments.iter().all(|s| !s.is_known()));
    }

    #[test]
    fn should_hide_furigana_when_all_kanji_known() {
        ensure_dictionary();
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        known_kanji.insert('物');

        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty());
        assert!(kanji_segments.iter().all(|s| s.has_reading()));
        assert!(kanji_segments.iter().all(|s| s.is_known()));
    }

    #[test]
    fn apply_reading_spans_span_beyond_surface_degrades_to_full_reading() {
        // Regression (grammar drawer N3 crash): a JmdictFurigana entry matched
        // by lemma can carry reading spans indexed against the dictionary
        // word, while the SudachiDict token surface is shorter. Indexing
        // chars[start..=end] then panics with "range end index out of range"
        // inside a poll, taking down the wasm executor and the whole page.
        let spans = vec![ReadingSpan {
            start_index: 0,
            end_index: 2, // "触" entry word 触れ合い (4 chars), surface 触 (1 char)
            text: "ふれあ".to_string(),
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("触", &spans, &known_kanji);

        // The span cannot be laid over the surface: degrade to one segment
        // carrying the whole reading instead of panicking.
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "触");
        assert_eq!(segments[0].reading(), Some("ふれあ"));
    }

    #[test]
    fn apply_reading_spans_single_kanji_per_span() {
        let spans = vec![
            ReadingSpan {
                start_index: 0,
                end_index: 0,
                text: "たべ".to_string(),
            },
            ReadingSpan {
                start_index: 2,
                end_index: 2,
                text: "もの".to_string(),
            },
        ];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("食べ物", &spans, &known_kanji);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text(), "食");
        assert_eq!(segments[0].reading(), Some("たべ"));
        assert_eq!(segments[1].text(), "べ");
        assert_eq!(segments[1].reading(), None);
        assert_eq!(segments[2].text(), "物");
        assert_eq!(segments[2].reading(), Some("もの"));
    }

    #[test]
    fn apply_reading_spans_multi_char_span() {
        let spans = vec![ReadingSpan {
            start_index: 0,
            end_index: 1,
            text: "おとな".to_string(),
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("大人", &spans, &known_kanji);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "大人");
        assert_eq!(segments[0].reading(), Some("おとな"));
    }

    #[test]
    fn apply_reading_spans_known_kanji_marks_segment() {
        let spans = vec![
            ReadingSpan {
                start_index: 0,
                end_index: 0,
                text: "たべ".to_string(),
            },
            ReadingSpan {
                start_index: 2,
                end_index: 2,
                text: "もの".to_string(),
            },
        ];
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        let segments = apply_reading_spans("食べ物", &spans, &known_kanji);

        assert!(segments[0].is_known());
        assert!(!segments[2].is_known());
    }

    #[test]
    fn apply_reading_spans_tail_after_last_span() {
        let spans = vec![ReadingSpan {
            start_index: 0,
            end_index: 0,
            text: "ほう".to_string(),
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("方程式", &spans, &known_kanji);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text(), "方");
        assert_eq!(segments[0].reading(), Some("ほう"));
        assert_eq!(segments[1].text(), "程式");
        assert_eq!(segments[1].reading(), None);
    }

    #[test]
    fn spans_to_segments_without_reading_spans() {
        let annotated = vec![AnnotatedSpan {
            text: "食べ物".to_string(),
            reading: Some("たべもの".to_string()),
            reading_spans: vec![],
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = spans_to_segments(annotated, &known_kanji);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "食べ物");
        assert_eq!(segments[0].reading(), Some("たべもの"));
    }

    #[test]
    fn spans_to_segments_with_reading_spans() {
        let annotated = vec![AnnotatedSpan {
            text: "食べ物".to_string(),
            reading: Some("たべもの".to_string()),
            reading_spans: vec![
                ReadingSpan {
                    start_index: 0,
                    end_index: 0,
                    text: "たべ".to_string(),
                },
                ReadingSpan {
                    start_index: 2,
                    end_index: 2,
                    text: "もの".to_string(),
                },
            ],
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = spans_to_segments(annotated, &known_kanji);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text(), "食");
        assert_eq!(segments[0].reading(), Some("たべ"));
        assert_eq!(segments[1].text(), "べ");
        assert_eq!(segments[2].text(), "物");
        assert_eq!(segments[2].reading(), Some("もの"));
    }

    fn setup_dictionaries_for_integration() {
        ensure_dictionary();
        if !is_furigana_dict_loaded() {
            let content = "\
食べる|たべる|0:た
食べ物|たべもの|0:たべ;2:もの
大人|おとな|0-1:おとな
指|ゆび|0:ゆび
";
            let _ = crate::dictionary::furigana_dict::init_furigana_dict(content);
        }
    }

    #[test]
    fn furiganize_segments_uses_annotator_when_dict_loaded() {
        setup_dictionaries_for_integration();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let ta = segments
            .iter()
            .find(|s| s.text() == "食" && s.reading() == Some("タベ"));
        assert!(
            ta.is_some(),
            "expected segment '食' with reading 'タベ', got: {segments:?}"
        );

        let mo = segments
            .iter()
            .find(|s| s.text() == "物" && s.reading() == Some("モノ"));
        assert!(
            mo.is_some(),
            "expected segment '物' with reading 'モノ', got: {segments:?}"
        );
    }

    #[test]
    fn furiganize_segments_annotator_known_kanji() {
        setup_dictionaries_for_integration();
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let ta = segments.iter().find(|s| s.text() == "食").unwrap();
        assert!(ta.is_known());
        let mo = segments.iter().find(|s| s.text() == "物").unwrap();
        assert!(!mo.is_known());
    }
}
