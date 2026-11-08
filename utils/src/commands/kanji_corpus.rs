use origa::dictionary::furigana_dict::FuriganaDictionary;
use std::collections::{HashMap, HashSet};

/// One corpus word of JmdictFurigana: the written form, its full kana
/// reading (hiragana-normalized) and the per-span readings (hiragana-
/// normalized, audit-style: a range span contributes its whole reading to
/// every position it covers — same semantics as
/// `scripts/audit_kanji_readings.py`).
#[derive(Clone)]
pub struct CorpusWord {
    pub text: String,
    pub reading: String,
    pub span_readings: Vec<(usize, usize, String)>,
}

/// Per-kanji view over the corpus: all words containing a given kanji.
pub struct KanjiCorpusIndex {
    words_by_kanji: HashMap<char, Vec<CorpusWord>>,
}

impl KanjiCorpusIndex {
    pub fn build(dictionary: &FuriganaDictionary) -> Self {
        let mut words_by_kanji: HashMap<char, Vec<CorpusWord>> = HashMap::new();
        for entries in dictionary.grouped_entries() {
            for entry in entries {
                let word = CorpusWord {
                    text: entry.text.clone(),
                    reading: katakana_to_hiragana(&entry.reading),
                    span_readings: entry
                        .reading_spans
                        .iter()
                        .map(|span| {
                            (
                                span.start_index,
                                span.end_index,
                                katakana_to_hiragana(&span.text),
                            )
                        })
                        .collect(),
                };
                for kanji in word.text.chars().filter(|c| is_kanji(*c)) {
                    words_by_kanji.entry(kanji).or_default().push(word.clone());
                }
            }
        }
        Self { words_by_kanji }
    }

    /// Body candidates of a kun reading: span readings at the kanji
    /// position for corpus words whose full reading equals the form.
    /// Multiple candidates leave the reading ungrouped (ambiguous body).
    pub fn kun_body_candidates(&self, kanji: char, full_reading: &str) -> Vec<String> {
        let mut bodies = Vec::new();
        for word in self.words(kanji) {
            if !word.text.starts_with(kanji) || word.reading != full_reading {
                continue;
            }
            for &(start, _, ref body) in &word.span_readings {
                if start == 0 && !bodies.contains(body) {
                    bodies.push(body.clone());
                }
            }
        }
        bodies
    }

    /// Full kana readings of the kanji's popular words (the word must
    /// start with the kanji: a kun reading form is kanji + okurigana).
    /// A word with several corpus readings contributes all of them —
    /// e.g. 下る is read both くだる and さがる in JmdictFurigana.
    pub fn popular_full_readings(&self, kanji: char, popular_words: &[String]) -> HashSet<String> {
        popular_words
            .iter()
            .flat_map(|word| {
                self.words(kanji)
                    .iter()
                    .filter(|candidate| {
                        candidate.text == *word && candidate.text.starts_with(kanji)
                    })
                    .map(|entry| entry.reading.clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Number of corpus words where the kanji position reads exactly the
    /// given hiragana reading.
    pub fn exact_reading_count(&self, kanji: char, reading: &str) -> usize {
        self.words(kanji)
            .iter()
            .filter(|word| {
                let Some(position) = word.text.chars().position(|c| c == kanji) else {
                    return false;
                };
                word.span_readings.iter().any(|&(start, end, ref text)| {
                    start <= position && position <= end && text == reading
                })
            })
            .count()
    }

    fn words(&self, kanji: char) -> &[CorpusWord] {
        self.words_by_kanji
            .get(&kanji)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

pub fn is_kanji(ch: char) -> bool {
    let code = ch as u32;
    (0x4E00..=0x9FFF).contains(&code)
        || (0x3400..=0x4DBF).contains(&code)
        || (0xF900..=0xFAFF).contains(&code)
}

pub fn katakana_to_hiragana(text: &str) -> String {
    text.chars()
        .map(|c| {
            let code = c as u32;
            if (0x30A1..=0x30F6).contains(&code) {
                char::from_u32(code - 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kun_body_candidates_resolves_exact_form_words() {
        // Arrange
        let corpus = FuriganaDictionary::from_text("見る|みる|0:み\n見える|みえる|0:み").unwrap();
        let index = KanjiCorpusIndex::build(&corpus);

        // Act
        let bodies = index.kun_body_candidates('見', "みる");

        // Assert
        assert_eq!(bodies, vec!["み"]);
    }

    #[test]
    fn popular_full_readings_collect_every_corpus_reading_of_a_word() {
        // Arrange: 下る carries two readings in JmdictFurigana.
        let corpus =
            FuriganaDictionary::from_text("下る|くだる|0:くだ\n下る|さがる|0:さが").unwrap();
        let index = KanjiCorpusIndex::build(&corpus);

        // Act
        let readings = index.popular_full_readings('下', &["下る".to_string()]);

        // Assert
        let expected: HashSet<String> =
            ["くだる", "さがる"].iter().map(|s| s.to_string()).collect();
        assert_eq!(readings, expected);
    }

    #[test]
    fn exact_reading_count_matches_span_at_kanji_position() {
        // Arrange
        let corpus =
            FuriganaDictionary::from_text("世話|せわ|0:せ\n世間|せけん|0:せ\n世紀|せいき|0:せい")
                .unwrap();
        let index = KanjiCorpusIndex::build(&corpus);

        // Act
        let (se, sei) = (
            index.exact_reading_count('世', "せ"),
            index.exact_reading_count('世', "せい"),
        );

        // Assert
        assert_eq!(se, 2);
        assert_eq!(sei, 1);
    }

    #[test]
    fn katakana_to_hiragana_maps_standard_block() {
        assert_eq!(katakana_to_hiragana("ジュウ"), "じゅう");
        assert_eq!(katakana_to_hiragana("みる"), "みる");
    }

    #[test]
    fn is_kanji_recognizes_cjk_ranges() {
        assert!(is_kanji('自'));
        assert!(!is_kanji('あ'));
        assert!(!is_kanji('A'));
    }
}
