mod loader;
mod part_of_speech;

use std::sync::OnceLock;

pub use loader::load_dictionary;
pub use part_of_speech::PartOfSpeech;

use crate::domain::OrigaError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenInfo {
    orthographic_base_form: String,
    phonological_base_form: String,
    orthographic_surface_form: String,
    phonological_surface_form: String,
    part_of_speech: PartOfSpeech,
}

impl TokenInfo {
    pub fn orthographic_base_form(&self) -> &str {
        &self.orthographic_base_form
    }

    pub fn phonological_base_form(&self) -> &str {
        &self.phonological_base_form
    }

    pub fn orthographic_surface_form(&self) -> &str {
        &self.orthographic_surface_form
    }

    pub fn phonological_surface_form(&self) -> &str {
        &self.phonological_surface_form
    }

    pub fn part_of_speech(&self) -> &PartOfSpeech {
        &self.part_of_speech
    }
}

struct DictionaryData {
    char_def: Vec<u8>,
    matrix: Vec<u8>,
    dict_da: Vec<u8>,
    dict_vals: Vec<u8>,
    unk: Vec<u8>,
    words_idx: Vec<u8>,
    words: Vec<u8>,
    metadata: Vec<u8>,
}

static DICTIONARY_DATA: OnceLock<DictionaryData> = OnceLock::new();
static TOKENIZER: OnceLock<lindera::tokenizer::Tokenizer> = OnceLock::new();

pub fn is_dictionary_loaded() -> bool {
    TOKENIZER.get().is_some()
}

pub fn tokenize_text(text: &str) -> Result<Vec<TokenInfo>, OrigaError> {
    let tokenizer = TOKENIZER.get().ok_or(OrigaError::TokenizerError {
        reason: "Dictionary not loaded. Call load_dictionary() first.".to_string(),
    })?;

    let filtered_text = crate::domain::filter_japanese_text(text);
    let mut tokens =
        tokenizer
            .tokenize(&filtered_text)
            .map_err(|e| OrigaError::TokenizerError {
                reason: e.to_string(),
            })?;

    let token_infos = tokens
        .iter_mut()
        .map(|token| TokenInfo {
            orthographic_base_form: token.get("lexeme").unwrap_or_default().to_string(),
            phonological_base_form: token
                .get("phonological_base_form")
                .unwrap_or_default()
                .to_string(),
            orthographic_surface_form: token
                .get("orthographic_surface_form")
                .unwrap_or_default()
                .to_string(),
            phonological_surface_form: token
                .get("phonological_surface_form")
                .unwrap_or_default()
                .to_string(),
            part_of_speech: token
                .get("part_of_speech")
                .unwrap_or_default()
                .parse()
                .unwrap_or(PartOfSpeech::Unspecified),
        })
        .collect();

    Ok(token_infos)
}

#[cfg(test)]
mod tests {
    use crate::domain::tokenizer::loader::load_dictionary;

    use super::*;

    fn ensure_dictionary() {
        if !is_dictionary_loaded() {
            let _ = load_dictionary();
        }
    }

    #[test]
    fn should_return_base_form_for_verb() {
        ensure_dictionary();
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_base_form_for_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "食べ物");
        assert_eq!(tokens[0].phonological_base_form, "タベモノ");
    }

    #[test]
    fn should_return_base_form_for_adjective() {
        ensure_dictionary();
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "美味しい");
        assert_eq!(tokens[0].phonological_base_form, "オイシー");
    }

    #[test]
    fn should_return_base_form_for_hiragana() {
        ensure_dictionary();
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_surface_form_for_verb() {
        ensure_dictionary();
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_return_surface_form_for_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ物");
        assert_eq!(tokens[0].phonological_surface_form, "タベモノ");
    }

    #[test]
    fn should_return_surface_form_for_adjective() {
        ensure_dictionary();
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "美味しい");
        assert_eq!(tokens[0].phonological_surface_form, "オイシー");
    }

    #[test]
    fn should_return_surface_form_for_hiragana() {
        ensure_dictionary();
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "たべ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }
}
