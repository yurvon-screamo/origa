//! Context-ruby furigana for sentence examples.
//!
//! Example chunks carry per-token ruby pairs (`ExampleDetail.ruby`, Tanaka
//! B-line readings) that hold the CONTEXT reading: 頭→あたま in
//! 「頭を冷やせ。」. Every dictionary path answers with the lemma's first
//! JmdictFurigana entry instead (頭→とう), which is exactly the
//! wrong-reading bug class. This module lays the context pairs over the
//! sentence first and lets the dictionary annotate only the leftovers.

use std::collections::HashSet;

use crate::dictionary::example::RubyPair;
use crate::domain::JapaneseChar;
use crate::domain::{FuriganaSegment, furiganize_segments};

/// One matched ruby occurrence: char range in the sentence + its reading.
struct RubyRange {
    start: usize,
    end: usize,
    reading: String,
}

/// Furiganize a sentence with the example's context pairs taking priority.
/// Pairs are matched left-to-right in Tanaka order (each claims the first
/// free occurrence of its surface at/after the previous match); pairs whose
/// surface is absent from the text (conjugated forms recorded by lemma) are
/// skipped. Uncovered spans — dictionary path as before, with a graceful
/// degrade: a dictionary miss (not loaded yet, annotate error) renders the
/// span as one plain run rather than failing the whole sentence.
pub fn furiganize_segments_with_ruby(
    text: &str,
    ruby: &[RubyPair],
    known_kanji: &HashSet<char>,
) -> Vec<FuriganaSegment> {
    let ranges = match_ruby_ranges(text, ruby);
    if ranges.is_empty() {
        return furiganize_segments(text, known_kanji)
            .unwrap_or_else(|_| vec![FuriganaSegment::new(text.to_string(), None, false)]);
    }
    build_segments(text, &ranges, known_kanji)
}

fn match_ruby_ranges(text: &str, ruby: &[RubyPair]) -> Vec<RubyRange> {
    let chars: Vec<char> = text.chars().collect();
    let mut ranges: Vec<RubyRange> = Vec::new();
    let mut search_start = 0usize;

    for pair in ruby {
        let surface: Vec<char> = pair.surface.chars().collect();
        if surface.is_empty() {
            continue;
        }
        let Some(start) = find_occurrence(&chars, &surface, search_start) else {
            continue;
        };
        let end = start + surface.len();
        // Defensive: Tanaka spans never overlap, but a corrupt chunk must
        // not push segments out of order.
        if ranges
            .last()
            .is_some_and(|prev: &RubyRange| start < prev.end)
        {
            continue;
        }
        ranges.push(RubyRange {
            start,
            end,
            reading: pair.reading.clone(),
        });
        search_start = end;
    }
    ranges
}

fn find_occurrence(chars: &[char], surface: &[char], from: usize) -> Option<usize> {
    if surface.len() > chars.len() {
        return None;
    }
    (from..=chars.len() - surface.len()).find(|&i| &chars[i..i + surface.len()] == surface)
}

fn build_segments(
    text: &str,
    ranges: &[RubyRange],
    known_kanji: &HashSet<char>,
) -> Vec<FuriganaSegment> {
    let chars: Vec<char> = text.chars().collect();
    let mut segments = Vec::new();
    let mut cursor = 0usize;

    for range in ranges {
        if cursor < range.start {
            segments.extend(gap_segments(&chars[cursor..range.start], known_kanji));
        }
        let span_text: String = chars[range.start..range.end].iter().collect();
        let is_known = span_text
            .chars()
            .filter(|c| c.is_kanji())
            .all(|c| known_kanji.contains(&c));
        segments.push(FuriganaSegment::new(
            span_text,
            Some(range.reading.clone()),
            is_known,
        ));
        cursor = range.end;
    }

    if cursor < chars.len() {
        segments.extend(gap_segments(&chars[cursor..], known_kanji));
    }
    segments
}

/// Uncovered span between context pairs: kana stays plain, kanji falls
/// back to the dictionary path (lemma readings are fine there — the wrong
/// reading only bites when context chose a different one).
fn gap_segments(chars: &[char], known_kanji: &HashSet<char>) -> Vec<FuriganaSegment> {
    let gap: String = chars.iter().collect();
    if !gap.chars().any(|c| c.is_kanji()) {
        return vec![FuriganaSegment::new(gap, None, false)];
    }
    furiganize_segments(&gap, known_kanji)
        .unwrap_or_else(|_| vec![FuriganaSegment::new(gap, None, false)])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dictionary setup mirrors furigana.rs tests: the furigana store is a
    // process-global, and the Coverage CI run orders tests differently from
    // a local run — a gap-fallback here once raced another test's half-
    // initialized state. Every test ensures BOTH dictionaries itself.
    fn ensure_lindera_dictionary() {
        use flate2::read::DeflateDecoder;
        use std::io::Read;

        if crate::domain::is_dictionary_loaded() {
            return;
        }
        let dict_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionaries")
            .join(crate::domain::SUDACHIDICT_DIR);

        let decompress = |data: Vec<u8>| -> Vec<u8> {
            let mut decoder = DeflateDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();
            decompressed
        };
        let read_file = |name: &str| std::fs::read(dict_dir.join(name)).expect("dictionary file");

        let data = crate::domain::DictionaryData {
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
        crate::domain::init_dictionary(data).unwrap();
    }

    fn ensure_furigana_dictionary() {
        if crate::dictionary::furigana_dict::is_furigana_dict_loaded() {
            return;
        }
        // Lines shared with the furigana.rs tests (whoever initializes
        // first must not break the other's expectations) plus the 頭 entry
        // this module's assertions rely on NOT being used: the context
        // pair (あたま) must win over the dictionary entry (とう).
        let content = concat!(
            "食べる|たべる|0:た\n",
            "食べ物|たべもの|0:たべ;2:もの\n",
            "大人|おとな|0-1:おとな\n",
            "指|ゆび|0:ゆび\n",
            "頭|とう|0:とう\n",
        );
        crate::dictionary::furigana_dict::init_furigana_dict(content).unwrap();
    }

    fn ensure_dictionaries() {
        ensure_lindera_dictionary();
        ensure_furigana_dictionary();
    }

    fn pair(surface: &str, reading: &str) -> RubyPair {
        RubyPair {
            surface: surface.to_string(),
            reading: reading.to_string(),
        }
    }

    #[test]
    fn context_pair_overrides_dictionary_reading_for_the_word() {
        ensure_dictionaries();
        // The screenshot case: 頭 must render あたま (context), not とう
        // (dictionary first entry) — the gap を。 is kana-only, no
        // dictionary needed for it.
        let ruby = vec![pair("頭", "あたま")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &ruby, &known);

        let head = segments
            .iter()
            .find(|s| s.text() == "頭")
            .expect("頭 segment");
        assert_eq!(head.reading(), Some("あたま"));
    }

    #[test]
    fn pairs_match_in_sentence_order_for_repeated_surfaces() {
        ensure_dictionaries();
        let ruby = vec![pair("今日", "きょう"), pair("今日", "こんにち")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("今日は今日の運命。", &ruby, &known);

        let first = segments
            .iter()
            .find(|s| s.text() == "今日")
            .expect("first 今日");
        assert_eq!(first.reading(), Some("きょう"));
        let second = segments
            .iter()
            .filter(|s| s.text() == "今日")
            .nth(1)
            .expect("second 今日");
        assert_eq!(second.reading(), Some("こんにち"));
    }

    #[test]
    fn absent_surface_pair_is_skipped_and_gaps_render_via_dictionary() {
        ensure_dictionaries();
        // 下った is a conjugated surface; a lemma-recorded pair (下る) does
        // not match the text and the gap falls back to the dictionary.
        let ruby = vec![pair("下る", "くだる")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("坂を下った。", &ruby, &known);

        assert!(
            segments.iter().all(|s| s.reading() != Some("くだる")),
            "non-matching pair must not attach anywhere: {segments:?}"
        );
    }

    #[test]
    fn empty_ruby_degrades_to_plain_dictionary_path() {
        ensure_dictionaries();
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &[], &known);
        assert!(!segments.is_empty());
    }

    #[test]
    fn kana_gaps_render_without_reading() {
        ensure_dictionaries();
        let ruby = vec![pair("冷やせ", "ひやせ")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &ruby, &known);

        let wo = segments
            .iter()
            .find(|s| s.text() == "を")
            .expect("particle segment");
        assert_eq!(wo.reading(), None);
    }

    #[test]
    fn context_segments_track_known_kanji() {
        ensure_dictionaries();
        let ruby = vec![pair("頭", "あたま")];
        let mut known = HashSet::new();
        known.insert('頭');
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &ruby, &known);

        let head = segments
            .iter()
            .find(|s| s.text() == "頭")
            .expect("頭 segment");
        assert!(head.is_known());
    }
}
