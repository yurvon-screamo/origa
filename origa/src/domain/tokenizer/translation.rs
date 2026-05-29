use serde::Serialize;

use super::{PartOfSpeech, TokenInfo};
use crate::dictionary::grammar::GRAMMAR_RULES;
use crate::dictionary::vocabulary::get_translation;
use crate::domain::NativeLanguage;

#[derive(Debug, Clone, Serialize)]
pub struct TokenTranslation {
    pub surface_form: String,
    pub base_form: String,
    pub reading: String,
    pub pos: PartOfSpeech,
    pub translation: Option<String>,
    pub grammar_label: Option<String>,
}

pub fn lookup_tokens_translations(
    tokens: &[TokenInfo],
    native_language: &NativeLanguage,
) -> Vec<TokenTranslation> {
    tokens
        .iter()
        .map(|token| {
            let base_form = token.orthographic_base_form().to_string();
            let translation = get_translation(&base_form, native_language);
            let grammar_label = resolve_grammar_label(
                token.orthographic_surface_form(),
                token.part_of_speech(),
                native_language,
            );

            TokenTranslation {
                surface_form: token.orthographic_surface_form().to_string(),
                base_form,
                reading: token.phonological_surface_form().to_string(),
                pos: token.part_of_speech().clone(),
                translation,
                grammar_label,
            }
        })
        .collect()
}

fn resolve_grammar_label(
    surface: &str,
    pos: &PartOfSpeech,
    native_language: &NativeLanguage,
) -> Option<String> {
    if pos.is_vocabulary_word() {
        return None;
    }

    let rules = GRAMMAR_RULES.get()?;
    for rule in rules.iter() {
        for group in rule.keywords().iter() {
            if group.iter().any(|kw| surface == kw) {
                return Some(rule.content(native_language).title().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(base: &str, surface: &str, reading: &str, pos: PartOfSpeech) -> TokenInfo {
        TokenInfo {
            orthographic_base_form: base.to_string(),
            phonological_base_form: reading.to_string(),
            orthographic_surface_form: surface.to_string(),
            phonological_surface_form: reading.to_string(),
            part_of_speech: pos,
        }
    }

    #[test]
    fn should_map_all_fields_from_token_info() {
        let tokens = vec![TokenInfo {
            orthographic_base_form: "食べる".to_string(),
            phonological_base_form: "タベル".to_string(),
            orthographic_surface_form: "食べ".to_string(),
            phonological_surface_form: "タベ".to_string(),
            part_of_speech: PartOfSpeech::Verb,
        }];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::English);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].base_form, "食べる");
        assert_eq!(result[0].surface_form, "食べ");
        assert_eq!(result[0].reading, "タベ");
        assert_eq!(result[0].pos, PartOfSpeech::Verb);
    }

    #[test]
    fn should_return_none_translation_for_unknown_word() {
        let tokens = vec![make_token("未知語", "未知語", "ミチゴ", PartOfSpeech::Noun)];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::Russian);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].base_form, "未知語");
        assert_eq!(result[0].reading, "ミチゴ");
        assert!(result[0].translation.is_none());
    }

    #[test]
    fn should_include_punctuation_with_none_translation() {
        let tokens = vec![make_token("。", "。", "。", PartOfSpeech::Symbol)];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::Russian);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].surface_form, "。");
        assert!(result[0].translation.is_none());
    }

    #[test]
    fn should_handle_multiple_tokens() {
        let tokens = vec![
            make_token("猫", "猫", "ネコ", PartOfSpeech::Noun),
            make_token("は", "は", "ハ", PartOfSpeech::Particle),
            make_token("。", "。", "。", PartOfSpeech::Symbol),
        ];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::English);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].base_form, "猫");
        assert_eq!(result[1].base_form, "は");
        assert_eq!(result[2].surface_form, "。");
    }

    #[test]
    fn should_return_empty_for_empty_input() {
        let tokens: Vec<TokenInfo> = vec![];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::Russian);

        assert!(result.is_empty());
    }
}
