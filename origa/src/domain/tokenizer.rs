use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Verb,        // Глагол
    Noun,        // Существительное
    IAdjective,  // И-прилагательное
    NaAdjective, // На-прилагательное

    Adverb,            // Детерминатив
    PreNounAdjectival, // Предикатив

    Conjunction,     // Союз
    Interjection,    // Междометие
    Prefix,          // Префикс
    Suffix,          // Суффикс
    Particle,        // Частица
    AuxiliaryVerb,   // Вспомогательный глагол
    Pronoun,         // Местоимение
    Numeral,         // Числительное
    Determiner,      // Определитель
    Unspecified,     // Неизвестно
    Other,           // Другое
    Symbol,          // Символ
    Whitespace,      // Пробел
    AuxiliarySymbol, // Вспомогательный символ
}

impl PartOfSpeech {
    /// Является ли часть речи "словарным словом" (лексической единицей) для изучения в контексте JLPT.
    ///
    /// Включает:
    /// - Все самостоятельные части речи (существительные, глаголы, прилагательные)
    /// - Наречия и предикативные прилагательные (ключевые для построения предложений)
    /// - Местоимения и числительные (базовая лексика всех уровней)
    /// - Предикативные определители (например: この, その, あの)
    ///
    /// Исключает:
    /// - Служебные части речи (частицы, вспомогательные глаголы)
    /// - Грамматические символы и пробелы
    /// - Редкие/архаичные категории
    pub fn is_vocabulary_word(&self) -> bool {
        matches!(
            self,
            PartOfSpeech::Noun
                | PartOfSpeech::Verb
                | PartOfSpeech::IAdjective
                | PartOfSpeech::NaAdjective
                | PartOfSpeech::Adverb
                | PartOfSpeech::PreNounAdjectival
                | PartOfSpeech::Pronoun
                | PartOfSpeech::Numeral
                | PartOfSpeech::Determiner
                | PartOfSpeech::Conjunction
        )
    }
}

impl std::str::FromStr for PartOfSpeech {
    fn from_str(japanese: &str) -> Result<Self, OrigaError> {
        Ok(match japanese {
            "動詞" => PartOfSpeech::Verb,
            "名詞" => PartOfSpeech::Noun,
            "形容詞" => PartOfSpeech::IAdjective,
            "形状詞" => PartOfSpeech::NaAdjective,
            "副詞" => PartOfSpeech::Adverb,
            "連体詞" => PartOfSpeech::PreNounAdjectival,
            "接続詞" => PartOfSpeech::Conjunction,
            "感動詞" => PartOfSpeech::Interjection,
            "接頭辞" => PartOfSpeech::Prefix,
            "接尾辞" => PartOfSpeech::Suffix,
            "助詞" => PartOfSpeech::Particle,
            "助動詞" => PartOfSpeech::AuxiliaryVerb,
            "代名詞" => PartOfSpeech::Pronoun,
            "数詞" => PartOfSpeech::Numeral,
            "限定詞" => PartOfSpeech::Determiner,
            "未特定" => PartOfSpeech::Unspecified,
            "その他" => PartOfSpeech::Other,
            "記号" => PartOfSpeech::Symbol,
            "空白" => PartOfSpeech::Whitespace,
            "補助記号" => PartOfSpeech::AuxiliarySymbol,
            _ => {
                return Err(OrigaError::TokenizerError {
                    reason: format!("Unknown part of speech: '{japanese}'"),
                });
            }
        })
    }

    type Err = OrigaError;
}

static TOKENIZER: LazyLock<lindera::tokenizer::Tokenizer> = LazyLock::new(|| {
    let dictionary = lindera::dictionary::load_dictionary("embedded://unidic")
        .map_err(|e| OrigaError::TokenizerError {
            reason: e.to_string(),
        })
        .unwrap();

    let segmenter =
        lindera::segmenter::Segmenter::new(lindera::mode::Mode::Normal, dictionary, None);

    lindera::tokenizer::Tokenizer::new(segmenter)
});

pub fn tokenize_text(text: &str) -> Result<Vec<TokenInfo>, OrigaError> {
    let mut tokens = TOKENIZER
        .tokenize(text)
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
    use super::*;

    #[test]
    fn should_return_base_form_for_verb() {
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_base_form_for_noun() {
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "食べ物");
        assert_eq!(tokens[0].phonological_base_form, "タベモノ");
    }

    #[test]
    fn should_return_base_form_for_adjective() {
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "美味しい");
        assert_eq!(tokens[0].phonological_base_form, "オイシー");
    }

    #[test]
    fn should_return_base_form_for_hiragana() {
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_surface_form_for_verb() {
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_return_surface_form_for_noun() {
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ物");
        assert_eq!(tokens[0].phonological_surface_form, "タベモノ");
    }

    #[test]
    fn should_return_surface_form_for_adjective() {
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "美味しい");
        assert_eq!(tokens[0].phonological_surface_form, "オイシー");
    }

    #[test]
    fn should_return_surface_form_for_hiragana() {
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "たべ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }
}
