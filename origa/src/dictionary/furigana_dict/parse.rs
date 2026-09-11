//! JmdictFurigana text parsing and the lookup implementations for both
//! storage forms (owned `BTreeMap` and its archived zero-copy twin).

use std::collections::BTreeMap;

use super::{FuriganaEntry, ReadingSpan};
use crate::domain::OrigaError;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FuriganaDictionary {
    pub(super) entries: BTreeMap<String, Vec<FuriganaEntry>>,
}

impl FuriganaDictionary {
    pub fn from_text(content: &str) -> Result<Self, OrigaError> {
        let mut entries: BTreeMap<String, Vec<FuriganaEntry>> = BTreeMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(entry) = parse_line(trimmed) {
                entries.entry(entry.text.clone()).or_default().push(entry);
            }
        }

        Ok(Self { entries })
    }

    pub fn lookup_word(&self, word: &str) -> Vec<FuriganaEntry> {
        self.entries
            .get(word)
            .map(|v| v.iter().cloned())
            .unwrap_or_default()
            .collect()
    }

    pub fn lookup_prefixed(&self, prefix: &str) -> Vec<FuriganaEntry> {
        self.entries
            .range(prefix.to_string()..)
            .map_while(|(text, entries)| {
                if text.starts_with(prefix) {
                    Some(entries.iter().cloned())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

impl ArchivedFuriganaDictionary {
    pub fn lookup_word(&self, word: &str) -> Vec<FuriganaEntry> {
        self.entries
            .get(word)
            .map(|v| v.iter().map(clone_entry).collect())
            .unwrap_or_default()
    }

    pub fn lookup_prefixed(&self, prefix: &str) -> Vec<FuriganaEntry> {
        self.entries
            .range::<str, _>((
                std::ops::Bound::Included(prefix),
                std::ops::Bound::Unbounded,
            ))
            .map_while(|(text, entries)| {
                if text.as_str().starts_with(prefix) {
                    Some(entries.iter().map(clone_entry))
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

fn clone_entry(entry: &rkyv::Archived<FuriganaEntry>) -> FuriganaEntry {
    FuriganaEntry {
        text: entry.text.as_str().to_string(),
        reading: entry.reading.as_str().to_string(),
        reading_spans: entry
            .reading_spans
            .iter()
            .map(|span| ReadingSpan {
                start_index: u32::from(span.start_index) as usize,
                end_index: u32::from(span.end_index) as usize,
                text: span.text.as_str().to_string(),
            })
            .collect(),
    }
}

fn parse_line(line: &str) -> Option<FuriganaEntry> {
    let parts: Vec<&str> = line.splitn(3, '|').collect();
    if parts.len() != 3 {
        return None;
    }

    let text = parts[0].to_string();
    let reading = parts[1].to_string();
    let reading_spans = parse_reading_spans(parts[2])?;

    Some(FuriganaEntry {
        text,
        reading,
        reading_spans,
    })
}

fn parse_reading_spans(input: &str) -> Option<Vec<ReadingSpan>> {
    if input.is_empty() {
        return Some(vec![]);
    }

    input
        .split(';')
        .map(|span| {
            let (range, text) = span.split_once(':')?;
            let (start, end) = if let Some((s, e)) = range.split_once('-') {
                (s.parse().ok()?, e.parse().ok()?)
            } else {
                let s = range.parse().ok()?;
                (s, s)
            };
            Some(ReadingSpan {
                start_index: start,
                end_index: end,
                text: text.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::{access_furigana_payload, serialize_furigana_dict_to_rkyv};
    use super::*;

    #[test]
    fn parse_single_span() {
        let entry = parse_line("指|ゆび|0:ゆび").unwrap();
        assert_eq!(entry.text, "指");
        assert_eq!(entry.reading, "ゆび");
        assert_eq!(entry.reading_spans.len(), 1);
        assert_eq!(entry.reading_spans[0].start_index, 0);
        assert_eq!(entry.reading_spans[0].end_index, 0);
        assert_eq!(entry.reading_spans[0].text, "ゆび");
    }

    #[test]
    fn parse_range_span() {
        let entry = parse_line("大人|おとな|0-1:おとな").unwrap();
        assert_eq!(entry.text, "大人");
        assert_eq!(entry.reading, "おとな");
        assert_eq!(entry.reading_spans.len(), 1);
        assert_eq!(entry.reading_spans[0].start_index, 0);
        assert_eq!(entry.reading_spans[0].end_index, 1);
        assert_eq!(entry.reading_spans[0].text, "おとな");
    }

    #[test]
    fn parse_multiple_spans() {
        let entry = parse_line("間に合う|まにあう|0:ま;2:あ").unwrap();
        assert_eq!(entry.text, "間に合う");
        assert_eq!(entry.reading, "まにあう");
        assert_eq!(entry.reading_spans.len(), 2);
        assert_eq!(entry.reading_spans[0].start_index, 0);
        assert_eq!(entry.reading_spans[0].end_index, 0);
        assert_eq!(entry.reading_spans[0].text, "ま");
        assert_eq!(entry.reading_spans[1].start_index, 2);
        assert_eq!(entry.reading_spans[1].end_index, 2);
        assert_eq!(entry.reading_spans[1].text, "あ");
    }

    #[test]
    fn parse_complex_entry() {
        let entry = parse_line("方程式|ほうていしき|0:ほう;1:てい;2:しき").unwrap();
        assert_eq!(entry.text, "方程式");
        assert_eq!(entry.reading_spans.len(), 3);
        assert_eq!(entry.reading_spans[0].text, "ほう");
        assert_eq!(entry.reading_spans[1].text, "てい");
        assert_eq!(entry.reading_spans[2].text, "しき");
    }

    #[test]
    fn parse_partial_reading() {
        let entry = parse_line("食べる|たべる|0:た").unwrap();
        assert_eq!(entry.reading_spans.len(), 1);
        assert_eq!(entry.reading_spans[0].text, "た");
    }

    #[test]
    fn empty_input_produces_empty_dict() {
        let dict = FuriganaDictionary::from_text("").unwrap();
        assert!(dict.lookup_word("指").is_empty());
    }

    #[test]
    fn invalid_lines_are_skipped() {
        let content = "invalid_line_without_pipes\n指|ゆび|0:ゆび\nalso_invalid";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let results = dict.lookup_word("指");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].reading, "ゆび");
    }

    #[test]
    fn lookup_word_returns_all_matches() {
        let content = "大人|おとな|0-1:おとな\n大人|だいじん|0-1:だいじん";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let results = dict.lookup_word("大人");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn lookup_word_returns_empty_for_unknown() {
        let content = "指|ゆび|0:ゆび";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        assert!(dict.lookup_word("手").is_empty());
    }

    #[test]
    fn lookup_prefixed_returns_matching_entries() {
        let content = "食べる|たべる|0:た\n食べ物|たべもの|0-1:たべもの\n飲む|のむ|0:のむ";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let results = dict.lookup_prefixed("食べ");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn lookup_prefixed_returns_empty_for_no_match() {
        let content = "食べる|たべる|0:た";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        assert!(dict.lookup_prefixed("飲").is_empty());
    }

    #[test]
    fn integration_lookup_across_multiple_entries() {
        let content = "\
指|ゆび|0:ゆび
間に合う|まにあう|0:ま;2:あ
大人|おとな|0-1:おとな
方程式|ほうていしき|0:ほう;1:てい;2:しき
食べる|たべる|0:た";
        let dict = FuriganaDictionary::from_text(content).unwrap();

        let yubi = dict.lookup_word("指");
        assert_eq!(yubi.len(), 1);
        assert_eq!(yubi[0].reading_spans[0].text, "ゆび");

        let ok = dict.lookup_word("間に合う");
        assert_eq!(ok.len(), 1);
        assert_eq!(ok[0].reading_spans.len(), 2);

        let prefixed = dict.lookup_prefixed("大");
        assert_eq!(prefixed.len(), 1);
        assert_eq!(prefixed[0].text, "大人");
    }

    fn parity_content() -> &'static str {
        "\
指|ゆび|0:ゆび
間に合う|まにあう|0:ま;2:あ
大人|おとな|0-1:おとな
大人|だいじん|0-1:だいじん
方程式|ほうていしき|0:ほう;1:てい;2:しき"
    }

    fn parity_fixture() -> (FuriganaDictionary, &'static ArchivedFuriganaDictionary) {
        let dict = FuriganaDictionary::from_text(parity_content()).unwrap();
        let payload = serialize_furigana_dict_to_rkyv(&dict).unwrap();
        let view = access_furigana_payload(&payload).unwrap();
        (dict, view)
    }

    /// The zero-copy archived view must answer exactly like the owned
    /// dictionary built from the same text.
    #[test]
    fn archived_word_lookups_match_owned_dictionary() {
        let (dict, view) = parity_fixture();
        for word in ["大人", "指", "手"] {
            assert_eq!(
                view.lookup_word(word),
                dict.lookup_word(word),
                "word lookup mismatch for {word}"
            );
        }
    }

    #[test]
    fn archived_prefix_scans_match_owned_dictionary() {
        let (dict, view) = parity_fixture();
        for prefix in ["大", "食べ"] {
            assert_eq!(
                view.lookup_prefixed(prefix),
                dict.lookup_prefixed(prefix),
                "prefix scan mismatch for {prefix}"
            );
        }
    }

    #[test]
    fn access_rejects_truncated_payload() {
        // Arrange: checked access must refuse an archive claiming structures
        // beyond the buffer. Content-level corruption is the CDN blob
        // header guard's job (`dictionary::cdn_blob`).
        let content = "\
指|ゆび|0:ゆび
間に合う|まにあう|0:ま;2:あ
大人|おとな|0-1:おとな
大人|だいじん|0-1:だいじん
方程式|ほうていしき|0:ほう;1:てい;2:しき";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let payload = serialize_furigana_dict_to_rkyv(&dict).unwrap();
        let truncated = &payload[..payload.len() / 2];

        // Act
        let result = access_furigana_payload(truncated);

        // Assert
        assert!(result.is_err());
    }
}
