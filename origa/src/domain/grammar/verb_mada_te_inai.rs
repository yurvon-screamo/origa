// use serde::{Deserialize, Serialize};

// use crate::domain::{
//     OrigaError,
//     grammar::{GrammarRule, verb_forms::to_te_form},
//     tokenizer::PartOfSpeech,
//     value_objects::{JapaneseLevel, NativeLanguage},
// };

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct VerbMadaTeInaiRule;

// impl GrammarRule for VerbMadaTeInaiRule {
//     fn apply_to(&self) -> Vec<PartOfSpeech> {
//         vec![PartOfSpeech::Verb]
//     }

//     fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
//         match part_of_speech {
//             PartOfSpeech::Verb => Ok(format!("まだ{}いません", to_te_form(word))),
//             _ => Err(OrigaError::GrammarFormatError {
//                 reason: "Not supported part of speech".to_string(),
//             }),
//         }
//     }

//     fn title(&self, lang: &NativeLanguage) -> String {
//         match lang {
//             NativeLanguage::Russian => "Форма まだ～ていません",
//             NativeLanguage::English => "Form まだ～ていません",
//         }
//         .to_string()
//     }

//     fn short_description(&self, lang: &NativeLanguage) -> String {
//         match lang {
//             NativeLanguage::English => "",
//             NativeLanguage::Russian => "Пока не...",
//         }
//         .to_string()
//     }

//     fn md_description(&self, lang: &NativeLanguage) -> String {
//         match lang {
//             NativeLanguage::Russian => {
//                 r#"# Форма まだ～ていません

// Форма "пока не" для действий.

// ## Примеры
// - まだ、決まっていません (Пока не решил)
// - まだ食べていません (Пока не ел)"#
//             }
//             NativeLanguage::English => {
//                 r#"# Form まだ～ていません

// Form for "not yet" with actions.

// ## Examples
// - まだ、決まっていません (Not decided yet)
// - まだ食べていません (Haven't eaten yet)"#
//             }
//         }
//         .to_string()
//     }

//     fn level(&self) -> JapaneseLevel {
//         JapaneseLevel::N5
//     }
// }
