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

pub fn to_kunai_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}くない", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_katta_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}かった", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_kunakatta_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}くなかった", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_kute_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}くて", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_ku_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}く", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_kereba_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}ければ", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_sou_form_iadj(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}そう", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_sugiru_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}すぎる", stem))
        }
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

pub fn to_na_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::NaAdjective => Ok(format!("{}な", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only NaAdjective supported".to_string(),
        }),
    }
}

pub fn to_de_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::NaAdjective => Ok(format!("{}で", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only NaAdjective supported".to_string(),
        }),
    }
}

pub fn to_nara_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::NaAdjective => Ok(format!("{}なら", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only NaAdjective supported".to_string(),
        }),
    }
}

pub fn to_sou_form_naadj(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::NaAdjective => Ok(format!("{}そう", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only NaAdjective supported".to_string(),
        }),
    }
}

pub fn to_nasasou_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}なさそう", stem))
        }
        PartOfSpeech::NaAdjective => Ok(format!("{}じゃなさそう", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only adjectives supported".to_string(),
        }),
    }
}

pub fn to_garu_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => {
            let stem = word.trim_end_matches("い");
            Ok(format!("{}がる", stem))
        }
        PartOfSpeech::NaAdjective => Ok(format!("{}がる", word.trim_end_matches("な"))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only adjectives supported".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_kunai_form() {
        assert_eq!(
            to_kunai_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高くない"
        );
        assert_eq!(
            to_kunai_form("新しい", &PartOfSpeech::IAdjective).unwrap(),
            "新しくない"
        );
    }

    #[test]
    fn test_to_katta_form() {
        assert_eq!(
            to_katta_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高かった"
        );
        assert_eq!(
            to_katta_form("新しい", &PartOfSpeech::IAdjective).unwrap(),
            "新しかった"
        );
    }

    #[test]
    fn test_to_kunakatta_form() {
        assert_eq!(
            to_kunakatta_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高くなかった"
        );
    }

    #[test]
    fn test_to_kute_form() {
        assert_eq!(
            to_kute_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高くて"
        );
    }

    #[test]
    fn test_to_ku_form() {
        assert_eq!(
            to_ku_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高く"
        );
    }

    #[test]
    fn test_to_kereba_form() {
        assert_eq!(
            to_kereba_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高ければ"
        );
    }

    #[test]
    fn test_to_na_form() {
        assert_eq!(
            to_na_form("静かな", &PartOfSpeech::NaAdjective).unwrap(),
            "静かな"
        );
    }

    #[test]
    fn test_to_de_form() {
        assert_eq!(
            to_de_form("静かな", &PartOfSpeech::NaAdjective).unwrap(),
            "静かで"
        );
    }

    #[test]
    fn test_to_nara_form() {
        assert_eq!(
            to_nara_form("静かな", &PartOfSpeech::NaAdjective).unwrap(),
            "静かなら"
        );
    }

    #[test]
    fn test_to_sugiru_form() {
        assert_eq!(
            to_sugiru_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高すぎる"
        );
    }

    #[test]
    fn test_to_nasasou_form() {
        assert_eq!(
            to_nasasou_form("高い", &PartOfSpeech::IAdjective).unwrap(),
            "高なさそう"
        );
    }
}
