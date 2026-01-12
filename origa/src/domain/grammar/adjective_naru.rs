// use serde::{Deserialize, Serialize};

// use crate::domain::{
//     OrigaError,
//     grammar::GrammarRule,
//     tokenizer::PartOfSpeech,
//     value_objects::{JapaneseLevel, NativeLanguage},
// };

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct AdjectiveNaruRule;

// impl GrammarRule for AdjectiveNaruRule {
//     fn apply_to(&self) -> Vec<PartOfSpeech> {
//         vec![PartOfSpeech::IAdjective, PartOfSpeech::NaAdjective]
//     }

//     fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
//         match part_of_speech {
//             PartOfSpeech::IAdjective => {
//                 let word = word.trim_end_matches("い");
//                 Ok(format!("{}くなる", word))
//             }
//             PartOfSpeech::NaAdjective => {
//                 let word = word.trim_end_matches("な");
//                 Ok(format!("{}になる", word))
//             }
//             _ => Err(OrigaError::GrammarFormatError {
//                 reason: "Not supported part of speech".to_string(),
//             }),
//         }
//     }

//     fn title(&self, lang: &NativeLanguage) -> String {
//         match lang {
//             NativeLanguage::Russian => "Изменение состояния ～く/～になる",
//             NativeLanguage::English => "Change of state ～く/～になる",
//         }
//         .to_string()
//     }

//     fn short_description(&self, lang: &NativeLanguage) -> String {
//         match lang {
//             NativeLanguage::Russian => "Становиться **прилагательным**",
//             NativeLanguage::English => "",
//         }
//         .to_string()
//     }

//     fn md_description(&self, lang: &NativeLanguage) -> String {
//         match lang {
//             NativeLanguage::Russian => {
//                 r#"# Изменение состояния ～く/～になる

// Становиться + прилагательное.

// ## な прилагательные

// Для образования конструкции с な прилагательным, к основе прилагательного добавляется "になる".

// ## い прилагательные

// Для образования конструкции с い прилагательным, к основе прилагательного добавляется "くなる".

// ## Примеры
// - このバラの花はもっと美しくなりました (Цветок стал красивее)
// - 静かになりました (Стало тихо)
// "#
//             }
//             NativeLanguage::English => {
//                 r#"# Change of state ～く/～になる

// Become + adjective.

// ## な adjectives

// To form the construction with a な adjective, add "になる" to the base adjective.

// ## い adjectives

// To form the construction with an い adjective, add "くなる" to the base adjective.

// ## Examples
// - このバラの花はもっと美しくなりました (The rose became more beautiful)
// - 静かになりました (It became quiet)"#
//             }
//         }
//         .to_string()
//     }

//     fn level(&self) -> JapaneseLevel {
//         JapaneseLevel::N5
//     }
// }
