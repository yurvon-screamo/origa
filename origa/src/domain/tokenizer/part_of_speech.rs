use serde::{Deserialize, Serialize};

use crate::domain::OrigaError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Verb,
    Noun,
    IAdjective,
    NaAdjective,
    Adverb,
    PreNounAdjectival,
    Conjunction,
    Interjection,
    Prefix,
    Suffix,
    Particle,
    AuxiliaryVerb,
    Pronoun,
    Numeral,
    Determiner,
    Unspecified,
    Other,
    Symbol,
    Whitespace,
    AuxiliarySymbol,
}

impl PartOfSpeech {
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
    type Err = OrigaError;

    fn from_str(japanese: &str) -> Result<Self, Self::Err> {
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
}
