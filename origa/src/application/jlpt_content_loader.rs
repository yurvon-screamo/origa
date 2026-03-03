use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::JapaneseLevel;

#[derive(Debug, Clone)]
pub struct JlptContent {
    pub kanji_by_level: HashMap<JapaneseLevel, HashSet<String>>,
    pub words_by_level: HashMap<JapaneseLevel, HashSet<String>>,
    pub grammar_by_level: HashMap<JapaneseLevel, HashSet<String>>,
}

impl JlptContent {
    pub fn new() -> Self {
        Self {
            kanji_by_level: HashMap::new(),
            words_by_level: HashMap::new(),
            grammar_by_level: HashMap::new(),
        }
    }

    pub fn total_kanji(&self, level: JapaneseLevel) -> usize {
        self.kanji_by_level.get(&level).map_or(0, HashSet::len)
    }

    pub fn total_words(&self, level: JapaneseLevel) -> usize {
        self.words_by_level.get(&level).map_or(0, HashSet::len)
    }

    pub fn total_grammar(&self, level: JapaneseLevel) -> usize {
        self.grammar_by_level.get(&level).map_or(0, HashSet::len)
    }
}

impl Default for JlptContent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JlptContentError {
    #[error("Failed to load kanji data: {0}")]
    KanjiLoadError(String),

    #[error("Failed to load words data for level {0}: {1}")]
    WordsLoadError(String, String),

    #[error("Failed to load grammar data: {0}")]
    GrammarLoadError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct KanjiDictionary {
    kanji: Vec<KanjiEntry>,
}

#[derive(Debug, Deserialize)]
struct KanjiEntry {
    kanji: String,
    jlpt: String,
}

#[derive(Debug, Deserialize)]
struct JlptWordsFile {
    #[allow(dead_code)]
    level: String,
    words: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GrammarDictionary {
    grammar: Vec<GrammarEntry>,
}

#[derive(Debug, Deserialize)]
struct GrammarEntry {
    rule_id: String,
    level: String,
}

pub struct JlptContentLoader;

impl JlptContentLoader {
    pub fn load() -> Result<JlptContent, JlptContentError> {
        let mut content = JlptContent::new();

        Self::load_kanji(&mut content)?;
        Self::load_words(&mut content)?;
        Self::load_grammar(&mut content)?;

        Ok(content)
    }

    fn load_kanji(content: &mut JlptContent) -> Result<(), JlptContentError> {
        let path = Self::get_json_path("origa_ui/public/domain/dictionary/kanji.json");

        let data: KanjiDictionary = match Self::load_json_file(&path) {
            Ok(d) => d,
            Err(e) => {
                return Err(JlptContentError::KanjiLoadError(format!(
                    "{}: {}",
                    path.display(),
                    e
                )));
            }
        };

        for entry in data.kanji {
            if let Ok(level) = entry.jlpt.parse::<JapaneseLevel>() {
                content
                    .kanji_by_level
                    .entry(level)
                    .or_default()
                    .insert(entry.kanji);
            }
        }

        Ok(())
    }

    fn load_words(content: &mut JlptContent) -> Result<(), JlptContentError> {
        let levels = [
            (JapaneseLevel::N5, "jltp_n5.json"),
            (JapaneseLevel::N4, "jltp_n4.json"),
            (JapaneseLevel::N3, "jltp_n3.json"),
            (JapaneseLevel::N2, "jltp_n2.json"),
            (JapaneseLevel::N1, "jltp_n1.json"),
        ];

        for (level, filename) in levels {
            let path = Self::get_json_path(&format!(
                "origa_ui/public/domain/well_known_set/{}",
                filename
            ));

            match Self::load_json_file::<JlptWordsFile>(&path) {
                Ok(data) => {
                    content
                        .words_by_level
                        .entry(level)
                        .or_default()
                        .extend(data.words);
                }
                Err(e) => {
                    eprintln!("Warning: skipping {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    fn load_grammar(content: &mut JlptContent) -> Result<(), JlptContentError> {
        let path = Self::get_json_path("origa_ui/public/domain/grammar/grammar.json");

        let data: GrammarDictionary = match Self::load_json_file(&path) {
            Ok(d) => d,
            Err(e) => {
                return Err(JlptContentError::GrammarLoadError(format!(
                    "{}: {}",
                    path.display(),
                    e
                )));
            }
        };

        for entry in data.grammar {
            if let Ok(level) = entry.level.parse::<JapaneseLevel>() {
                content
                    .grammar_by_level
                    .entry(level)
                    .or_default()
                    .insert(entry.rule_id);
            }
        }

        Ok(())
    }

    fn get_json_path(relative_path: &str) -> PathBuf {
        let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
            .map(|p| PathBuf::from(p).parent().unwrap().to_path_buf())
            .unwrap_or_else(|_| PathBuf::from("."));

        workspace_root.join(relative_path)
    }

    fn load_json_file<T: serde::de::DeserializeOwned>(
        path: &PathBuf,
    ) -> Result<T, JlptContentError> {
        let content = std::fs::read_to_string(path)?;
        let data: T = serde_json::from_str(&content)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_jlpt_content() {
        let content = JlptContentLoader::load().expect("Failed to load JLPT content");

        assert!(
            content.total_kanji(JapaneseLevel::N5) > 0,
            "N5 should have kanji"
        );
        assert!(
            content.total_words(JapaneseLevel::N5) > 0,
            "N5 should have words"
        );
        assert!(
            content.total_kanji(JapaneseLevel::N1) > 0,
            "N1 should have kanji"
        );

        println!("N5 kanji: {}", content.total_kanji(JapaneseLevel::N5));
        println!("N5 words: {}", content.total_words(JapaneseLevel::N5));
        println!("N5 grammar: {}", content.total_grammar(JapaneseLevel::N5));
    }

    #[test]
    fn test_jlpt_content_totals() {
        let content = JlptContent::new();
        assert_eq!(content.total_kanji(JapaneseLevel::N5), 0);
        assert_eq!(content.total_words(JapaneseLevel::N5), 0);
        assert_eq!(content.total_grammar(JapaneseLevel::N5), 0);
    }
}
