use crate::domain::{
    OrigaError,
    japanese::{JapaneseChar, JapaneseText},
    tokenizer::tokenize_text,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuriganaSegment {
    text: String,
    reading: Option<String>,
}

impl FuriganaSegment {
    pub fn new(text: String, reading: Option<String>) -> Self {
        Self { text, reading }
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
}

pub fn furiganize_segments(text: &str) -> Result<Vec<FuriganaSegment>, OrigaError> {
    let mut segments = Vec::new();
    let mut current_segment = String::new();
    let mut is_current_japanese = false;

    for ch in text.chars() {
        let is_japanese = ch.is_japanese();

        if current_segment.is_empty() {
            is_current_japanese = is_japanese;
            current_segment.push(ch);
        } else if is_japanese == is_current_japanese {
            current_segment.push(ch);
        } else {
            if is_current_japanese {
                segments.extend(furiganize_clear_japanese_segments(&current_segment)?);
            } else {
                segments.push(FuriganaSegment::new(current_segment.clone(), None));
            }
            current_segment.clear();
            current_segment.push(ch);
            is_current_japanese = is_japanese;
        }
    }

    if !current_segment.is_empty() {
        if is_current_japanese {
            segments.extend(furiganize_clear_japanese_segments(&current_segment)?);
        } else {
            segments.push(FuriganaSegment::new(current_segment, None));
        }
    }

    Ok(segments)
}

fn furiganize_clear_japanese_segments(text: &str) -> Result<Vec<FuriganaSegment>, OrigaError> {
    let tokens = tokenize_text(text)?;
    let mut segments = Vec::new();

    for token in tokens {
        let segment = if token.orthographic_surface_form().contains_kanji() {
            FuriganaSegment::new(
                token.orthographic_surface_form().to_string(),
                Some(token.phonological_surface_form().to_string()),
            )
        } else {
            FuriganaSegment::new(token.orthographic_surface_form().to_string(), None)
        };
        segments.push(segment);
    }

    Ok(segments)
}

pub fn furiganize_text_html(segments: &[FuriganaSegment]) -> String {
    segments
        .iter()
        .map(|seg| match &seg.reading {
            Some(reading) => format!(
                "<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>",
                seg.text, reading
            ),
            None => seg.text.clone(),
        })
        .collect()
}

pub fn furiganize_text(text: &str) -> Result<String, OrigaError> {
    let segments = furiganize_segments(text)?;
    Ok(furiganize_text_html(&segments))
}

#[cfg(test)]
mod tests {
    use std::{env, fs, io::Read, path::PathBuf};

    use flate2::read::DeflateDecoder;

    use super::*;
    use crate::domain::{DictionaryData, init_dictionary, is_dictionary_loaded};

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

        let dict_dir = if let Ok(out_dir) = env::var("OUT_DIR") {
            let out_dict = PathBuf::from(out_dir).join("lindera-unidic");
            if out_dict.exists() {
                out_dict
            } else {
                let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
                PathBuf::from(manifest_dir)
                    .parent()
                    .unwrap()
                    .join("origa_ui")
                    .join("public")
                    .join("dictionaries")
                    .join("unidic")
            }
        } else {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            PathBuf::from(manifest_dir)
                .parent()
                .unwrap()
                .join("origa_ui")
                .join("public")
                .join("dictionaries")
                .join("unidic")
        };

        let read_file = |name: &str| fs::read(dict_dir.join(name)).unwrap();

        let data = DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_da: decompress(read_file("dict.da")),
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
        let segment = FuriganaSegment::new("食べ".to_string(), Some("タベ".to_string()));
        assert_eq!(segment.text(), "食べ");
        assert_eq!(segment.reading(), Some("タベ"));
        assert!(segment.has_reading());
    }

    #[test]
    fn should_create_segment_without_reading() {
        let segment = FuriganaSegment::new("たべ".to_string(), None);
        assert_eq!(segment.text(), "たべ");
        assert_eq!(segment.reading(), None);
        assert!(!segment.has_reading());
    }

    #[test]
    fn should_furiganize_kanji_word_with_reading() {
        ensure_dictionary();
        let segments = furiganize_segments("食べ物").unwrap();
        assert!(!segments.is_empty());
        assert!(segments.iter().any(|s| s.has_reading()));
    }

    #[test]
    fn should_furiganize_hiragana_without_reading() {
        ensure_dictionary();
        let segments = furiganize_segments("たべもの").unwrap();
        assert!(!segments.is_empty());
        assert!(segments.iter().all(|s| !s.has_reading()));
    }

    #[test]
    fn should_furiganize_mixed_text() {
        ensure_dictionary();
        let segments = furiganize_segments("食べます").unwrap();
        assert!(!segments.is_empty());
    }

    #[test]
    fn should_furiganize_non_japanese_text() {
        let segments = furiganize_segments("hello").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "hello");
        assert!(!segments[0].has_reading());
    }

    #[test]
    fn should_furiganize_mixed_japanese_and_ascii() {
        ensure_dictionary();
        let segments = furiganize_segments("hello食べ物world").unwrap();
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
        )];
        let html = furiganize_text_html(&segments);
        assert_eq!(html, "<ruby>食<rp>(</rp><rt>ショク</rt><rp>)</rp></ruby>");
    }

    #[test]
    fn should_generate_html_for_segment_without_reading() {
        let segments = vec![FuriganaSegment::new("たべ".to_string(), None)];
        let html = furiganize_text_html(&segments);
        assert_eq!(html, "たべ");
    }

    #[test]
    fn should_generate_html_for_mixed_segments() {
        let segments = vec![
            FuriganaSegment::new("食".to_string(), Some("ショク".to_string())),
            FuriganaSegment::new("べ".to_string(), None),
        ];
        let html = furiganize_text_html(&segments);
        assert_eq!(html, "<ruby>食<rp>(</rp><rt>ショク</rt><rp>)</rp></ruby>べ");
    }

    #[test]
    fn should_furiganize_text_backwards_compatible() {
        ensure_dictionary();
        let result = furiganize_text("食べ物").unwrap();
        assert!(result.contains("<ruby>"));
        assert!(result.contains("<rt>"));
    }
}
