//! Example index: owned JSON-built form — word -> sentence refs.

use std::collections::HashMap;

use serde::Deserialize;

use crate::domain::OrigaError;

/// One example reference: the sentence holding the word and the character
/// offsets of the word occurrence inside the sentence text. Negative offsets
/// mean the surface form could not be located (kana variant of a kanji
/// word) — the UI then shows the sentence without highlight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleRef {
    sentence_id: u32,
    start: i32,
    end: i32,
}

impl ExampleRef {
    pub fn sentence_id(&self) -> u32 {
        self.sentence_id
    }

    /// Character offset of the word start, or -1 when unlocatable.
    pub fn start(&self) -> i32 {
        self.start
    }

    /// Exclusive character offset of the word end, or -1 when unlocatable.
    pub fn end(&self) -> i32 {
        self.end
    }
}

pub struct ExampleIndex {
    words: HashMap<String, Vec<ExampleRef>>,
    sentence_count: u32,
}

#[derive(Deserialize)]
struct IndexFile {
    #[serde(rename = "words")]
    words: HashMap<String, WordRefs>,
    #[serde(rename = "s", default)]
    sentence_count: u32,
}

#[derive(Deserialize)]
struct WordRefs {
    #[serde(rename = "refs")]
    refs: Vec<(u32, i32, i32)>,
}

impl ExampleIndex {
    pub fn from_json(json: &str) -> Result<Self, OrigaError> {
        let file: IndexFile =
            serde_json::from_str(json).map_err(|e| OrigaError::ExampleParseError {
                reason: format!("example index: {e}"),
            })?;
        let words = file
            .words
            .into_iter()
            .map(|(w, wr)| {
                let refs = wr
                    .refs
                    .into_iter()
                    .map(|(sentence_id, start, end)| ExampleRef {
                        sentence_id,
                        start,
                        end,
                    })
                    .collect();
                (w, refs)
            })
            .collect();
        Ok(Self {
            words,
            sentence_count: file.sentence_count,
        })
    }

    pub fn get_refs(&self, word: &str) -> Vec<ExampleRef> {
        self.words.get(word).cloned().unwrap_or_default()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    pub fn sentence_count(&self) -> u32 {
        self.sentence_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_refs_and_metadata() {
        let json = r#"{"v":1,"h":"x","s":5,"words":{"本":{"refs":[[0,0,1],[3,7,9]]}}}"#;
        let idx = ExampleIndex::from_json(json).expect("parse");
        assert_eq!(idx.word_count(), 1);
        assert_eq!(idx.sentence_count(), 5);
        let refs = idx.get_refs("本");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[1].sentence_id(), 3);
        assert_eq!(refs[1].start(), 7);
        assert_eq!(refs[1].end(), 9);
    }

    #[test]
    fn empty_words_object_is_valid() {
        let json = r#"{"v":1,"h":"","s":0,"words":{}}"#;
        let idx = ExampleIndex::from_json(json).expect("parse");
        assert_eq!(idx.word_count(), 0);
        assert!(idx.get_refs("何").is_empty());
    }

    #[test]
    fn missing_refs_field_is_an_error() {
        let json = r#"{"v":1,"h":"","s":0,"words":{"本":{}}}"#;
        assert!(ExampleIndex::from_json(json).is_err());
    }
}
