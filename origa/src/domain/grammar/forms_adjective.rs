use crate::domain::{OrigaError, PartOfSpeech};

pub fn adjective_remove_postfix(
    word: &str,
    part_of_speech: &PartOfSpeech,
) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => Ok(format!("{}くなる", word.trim_end_matches("い"))),
        PartOfSpeech::NaAdjective => Ok(format!("{}になる", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Not supported part of speech".to_string(),
        }),
    }
}
