use crate::dictionary::furigana_dict::{self, FuriganaEntry, ReadingSpan, get_furigana_dict};
use crate::domain::OrigaError;
use crate::domain::hiragana_to_katakana;
use crate::domain::katakana_to_hiragana;
use crate::domain::tokenizer::{TokenInfo, tokenize_text};

#[derive(Debug, Clone)]
pub struct AnnotatedSpan {
    pub text: String,
    pub reading: Option<String>,
    pub reading_spans: Vec<ReadingSpan>,
}

enum InternalToken {
    Single {
        surface: String,
        lookup_text: String,
        reading_hint: Option<String>,
    },
    Merged {
        text: String,
    },
}

pub fn annotate_text(text: &str) -> Result<Vec<AnnotatedSpan>, OrigaError> {
    let dict = get_furigana_dict().ok_or(OrigaError::FuriganaError {
        reason: "Furigana dictionary not loaded".to_string(),
    })?;

    let tokens = tokenize_text(text)?;
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let internal_tokens = build_internal_tokens(&tokens, dict);

    Ok(internal_tokens
        .into_iter()
        .map(|token| resolve_annotation(token, dict))
        .collect())
}

fn build_internal_tokens(
    tokens: &[TokenInfo],
    dict: &furigana_dict::FuriganaDictionary,
) -> Vec<InternalToken> {
    if tokens.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut buffer_start: usize = 0;
    let mut buffer_end: usize = 1;
    let mut possibilities = dict.lookup_prefixed(tokens[0].orthographic_surface_form());

    while buffer_start < tokens.len() {
        let next_exists = buffer_end < tokens.len();

        let current_substring = concat_surfaces(&tokens[buffer_start..buffer_end]);
        let possibilities_remain = possibilities
            .iter()
            .any(|p| p.text.starts_with(current_substring.as_str()));

        if next_exists && possibilities_remain {
            buffer_end += 1;
            continue;
        }

        let longest_end = find_longest_match(tokens, buffer_start, buffer_end, &possibilities);

        if longest_end <= buffer_start + 1 {
            let token = &tokens[buffer_start];
            result.push(InternalToken::Single {
                surface: token.orthographic_surface_form().to_string(),
                lookup_text: token.orthographic_base_form().to_string(),
                reading_hint: Some(token.phonological_surface_form().to_string()),
            });
            buffer_start += 1;
        } else {
            let text = concat_surfaces(&tokens[buffer_start..longest_end]);
            result.push(InternalToken::Merged { text });
            buffer_start = longest_end;
        }

        buffer_end = buffer_start + 1;
        if let Some(t) = tokens.get(buffer_start) {
            possibilities = dict.lookup_prefixed(t.orthographic_surface_form());
        }
    }

    result
}

fn find_longest_match(
    tokens: &[TokenInfo],
    buffer_start: usize,
    buffer_end: usize,
    possibilities: &[&FuriganaEntry],
) -> usize {
    let mut longest_end = buffer_end;
    while longest_end > buffer_start {
        let substring = concat_surfaces(&tokens[buffer_start..longest_end]);
        if possibilities.iter().any(|p| p.text == substring) {
            break;
        }
        longest_end -= 1;
    }
    longest_end
}

fn concat_surfaces(tokens: &[TokenInfo]) -> String {
    tokens
        .iter()
        .map(|t| t.orthographic_surface_form())
        .collect()
}

fn resolve_annotation(
    token: InternalToken,
    dict: &furigana_dict::FuriganaDictionary,
) -> AnnotatedSpan {
    match token {
        InternalToken::Single {
            surface,
            lookup_text,
            reading_hint,
        } => {
            let mut entries = dict.lookup_word(&lookup_text);
            sort_by_reading_hint(&mut entries, reading_hint.as_deref());

            if let Some(best) = entries.first() {
                AnnotatedSpan {
                    text: surface,
                    reading: Some(hiragana_to_katakana(&best.reading)),
                    reading_spans: best
                        .reading_spans
                        .iter()
                        .map(|s| ReadingSpan {
                            start_index: s.start_index,
                            end_index: s.end_index,
                            text: hiragana_to_katakana(&s.text),
                        })
                        .collect(),
                }
            } else {
                AnnotatedSpan {
                    text: surface,
                    reading: reading_hint,
                    reading_spans: vec![],
                }
            }
        },
        InternalToken::Merged { text } => {
            let entries = dict.lookup_word(&text);
            if let Some(best) = entries.first() {
                AnnotatedSpan {
                    text,
                    reading: Some(hiragana_to_katakana(&best.reading)),
                    reading_spans: best
                        .reading_spans
                        .iter()
                        .map(|s| ReadingSpan {
                            start_index: s.start_index,
                            end_index: s.end_index,
                            text: hiragana_to_katakana(&s.text),
                        })
                        .collect(),
                }
            } else {
                AnnotatedSpan {
                    text,
                    reading: None,
                    reading_spans: vec![],
                }
            }
        },
    }
}

fn sort_by_reading_hint(entries: &mut Vec<&FuriganaEntry>, hint: Option<&str>) {
    let Some(hint) = hint else { return };
    let hint_hiragana = katakana_to_hiragana(hint);
    entries.sort_by(|a, b| {
        let a_matches = a.reading == hint_hiragana;
        let b_matches = b.reading == hint_hiragana;
        b_matches.cmp(&a_matches)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_by_reading_hint_prioritizes_matching_reading() {
        let content = "大人|おとな|0-1:おとな\n大人|だいじん|0-1:だいじん";
        let dict = furigana_dict::FuriganaDictionary::from_text(content).unwrap();

        let mut entries = dict.lookup_word("大人");
        assert_eq!(entries.len(), 2);

        sort_by_reading_hint(&mut entries, Some("オトナ"));
        assert_eq!(entries[0].reading, "おとな");

        sort_by_reading_hint(&mut entries, Some("ダイジン"));
        assert_eq!(entries[0].reading, "だいじん");
    }

    #[test]
    fn sort_by_reading_hint_no_hint_does_nothing() {
        let content = "指|ゆび|0:ゆび";
        let dict = furigana_dict::FuriganaDictionary::from_text(content).unwrap();

        let mut entries = dict.lookup_word("指");
        let original_reading = entries[0].reading.clone();
        sort_by_reading_hint(&mut entries, None);
        assert_eq!(entries[0].reading, original_reading);
    }

    fn setup_dictionaries() {
        if !crate::domain::is_dictionary_loaded() {
            let data = create_lindera_dictionary_data();
            crate::domain::init_dictionary(data).unwrap();
        }

        if !furigana_dict::is_furigana_dict_loaded() {
            let content = "\
食べる|たべる|0:た
食べ物|たべもの|0:たべ;2:もの
大人|おとな|0-1:おとな
指|ゆび|0:ゆび
";
            furigana_dict::init_furigana_dict(content).unwrap();
        }
    }

    fn create_lindera_dictionary_data() -> crate::domain::DictionaryData {
        use flate2::read::DeflateDecoder;
        use std::fs;
        use std::io::Read;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let dict_dir = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionaries");

        let decompress = |data: Vec<u8>| -> Vec<u8> {
            let mut decoder = DeflateDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();
            decompressed
        };

        let read_file = |name: &str| fs::read(dict_dir.join(name)).unwrap();

        crate::domain::DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_da: decompress(read_file("dict.da")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        }
    }

    #[test]
    fn annotate_text_returns_empty_for_empty_input() {
        setup_dictionaries();
        let result = annotate_text("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn annotate_text_produces_spans_for_known_word() {
        setup_dictionaries();
        let result = annotate_text("食べ物").unwrap();
        assert!(!result.is_empty());

        let food = result.iter().find(|s| s.text.contains('食'));
        assert!(food.is_some());
        let span = food.unwrap();
        assert!(span.reading.is_some());
        assert!(!span.reading_spans.is_empty());
    }

    #[test]
    fn annotate_text_fallback_when_no_dict_entry() {
        setup_dictionaries();
        let result = annotate_text("食べます").unwrap();
        assert!(!result.is_empty());

        let tabe = result.iter().find(|s| s.text == "食べ");
        assert!(tabe.is_some());
        assert!(tabe.unwrap().reading.is_some());
    }
}
