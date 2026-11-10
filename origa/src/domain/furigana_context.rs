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
use crate::domain::{FuriganaSegment, OrigaError, furiganize_segments};

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
/// skipped. Uncovered spans — dictionary path as before.
pub fn furiganize_segments_with_ruby(
    text: &str,
    ruby: &[RubyPair],
    known_kanji: &HashSet<char>,
) -> Result<Vec<FuriganaSegment>, OrigaError> {
    let ranges = match_ruby_ranges(text, ruby);
    if ranges.is_empty() {
        return furiganize_segments(text, known_kanji);
    }
    Ok(build_segments(text, &ranges, known_kanji))
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

    fn pair(surface: &str, reading: &str) -> RubyPair {
        RubyPair {
            surface: surface.to_string(),
            reading: reading.to_string(),
        }
    }

    fn readings(segments: &[FuriganaSegment]) -> Vec<(String, Option<String>)> {
        segments
            .iter()
            .map(|s| (s.text().to_string(), s.reading().map(str::to_string)))
            .collect()
    }

    #[test]
    fn context_pair_overrides_dictionary_reading_for_the_word() {
        // The screenshot case: 頭 must render あたま (context), not とう
        // (dictionary first entry) — 冷やせる gap still gets dictionary help.
        let ruby = vec![pair("頭", "あたま")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &ruby, &known).unwrap();

        let head = segments
            .iter()
            .find(|s| s.text() == "頭")
            .expect("頭 segment");
        assert_eq!(head.reading(), Some("あたま"));
    }

    #[test]
    fn pairs_match_in_sentence_order_for_repeated_surfaces() {
        let ruby = vec![pair("今日", "きょう"), pair("今日", "こんにち")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("今日は今日の運命。", &ruby, &known).unwrap();

        assert_eq!(
            readings(&segments)[0],
            ("今日".into(), Some("きょう".into()))
        );
        let second = segments
            .iter()
            .filter(|s| s.text() == "今日")
            .nth(1)
            .expect("second 今日");
        assert_eq!(second.reading(), Some("こんにち"));
    }

    #[test]
    fn absent_surface_pair_is_skipped_and_gaps_render_via_dictionary() {
        // 下った is a conjugated surface; a lemma-recorded pair (下る) does
        // not match the text and the gap falls back to the dictionary.
        let ruby = vec![pair("下る", "くだる")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("坂を下った。", &ruby, &known).unwrap();

        assert!(
            segments.iter().all(|s| s.reading() != Some("くだる")),
            "non-matching pair must not attach anywhere: {segments:?}"
        );
    }

    #[test]
    fn empty_ruby_degrades_to_plain_dictionary_path() {
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &[], &known).unwrap();
        assert!(!segments.is_empty());
    }

    #[test]
    fn kana_gaps_render_without_reading() {
        let ruby = vec![pair("冷やせ", "ひやせ")];
        let known = HashSet::new();
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &ruby, &known).unwrap();

        let wo = segments
            .iter()
            .find(|s| s.text() == "を")
            .expect("particle segment");
        assert_eq!(wo.reading(), None);
    }

    #[test]
    fn context_segments_track_known_kanji() {
        let ruby = vec![pair("頭", "あたま")];
        let mut known = HashSet::new();
        known.insert('頭');
        let segments = furiganize_segments_with_ruby("頭を冷やせ。", &ruby, &known).unwrap();

        let head = segments
            .iter()
            .find(|s| s.text() == "頭")
            .expect("頭 segment");
        assert!(head.is_known());
    }
}
