use crate::domain::{
    OrigaError,
    japanese::{JapaneseChar, JapaneseText},
    tokenizer::tokenize_text,
};

pub fn furiganize_text(text: &str) -> Result<String, OrigaError> {
    let mut result = String::new();
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
                result.push_str(&furiganize_clear_japanese(&current_segment)?);
            } else {
                result.push_str(&current_segment);
            }
            current_segment.clear();
            current_segment.push(ch);
            is_current_japanese = is_japanese;
        }
    }

    if !current_segment.is_empty() {
        if is_current_japanese {
            result.push_str(&furiganize_clear_japanese(&current_segment)?);
        } else {
            result.push_str(&current_segment);
        }
    }

    Ok(result)
}

fn furiganize_clear_japanese(text: &str) -> Result<String, OrigaError> {
    let tokens = tokenize_text(text)?;
    let mut result = String::new();

    for token in tokens {
        let furigana = if token.orthographic_surface_form().contains_kanji() {
            format_html(
                token.orthographic_surface_form(),
                token.phonological_surface_form(),
            )
        } else {
            token.orthographic_surface_form().to_string()
        };

        result.push_str(&furigana);
    }

    Ok(result)
}

fn format_html(base: &str, text: &str) -> String {
    format!("<ruby>{base}<rp>(</rp><rt>{text}</rt><rp>)</rp></ruby>")
}
