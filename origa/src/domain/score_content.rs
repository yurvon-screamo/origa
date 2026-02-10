use std::collections::{HashMap, HashSet};

use ulid::Ulid;

use crate::domain::{Card, JapaneseChar, OrigaError, StudyCard, tokenize_text};

pub struct ScoreContentResult {
    unknown_words: Vec<String>,
    unknown_kanji: Vec<String>,

    known_words: Vec<String>,
    known_kanji: Vec<String>,
}

pub fn score_content(
    content: &str,
    cards: &HashMap<Ulid, StudyCard>,
) -> Result<ScoreContentResult, OrigaError> {
    let mut result = ScoreContentResult {
        known_words: vec![],
        unknown_words: vec![],

        known_kanji: vec![],
        unknown_kanji: vec![],
    };

    content
        .chars()
        .filter(|c| c.is_kanji())
        .map(|x| x.to_string())
        .collect::<HashSet<_>>()
        .iter()
        .for_each(|kanji| {
            let is_known = cards.values().any(|x| match x.card() {
                Card::Kanji(card) => card.kanji().text() == *kanji && x.memory().is_known_card(),
                _ => false,
            });

            match is_known {
                true => result.known_kanji.push(kanji.to_string()),
                false => result.unknown_kanji.push(kanji.to_string()),
            }
        });

    tokenize_text(content)?
        .iter()
        .filter(|token| token.part_of_speech().is_vocabulary_word())
        .map(|token| token.orthographic_base_form().to_string())
        .collect::<HashSet<_>>()
        .iter()
        .for_each(|word| {
            let is_known = cards.values().any(|x| match x.card() {
                Card::Vocabulary(card) => card.word().text() == *word && x.memory().is_known_card(),
                _ => false,
            });

            match is_known {
                true => result.known_words.push(word.to_string()),
                false => result.unknown_words.push(word.to_string()),
            }
        });

    Ok(result)
}
