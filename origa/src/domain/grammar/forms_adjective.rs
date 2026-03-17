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
    use rstest::rstest;

    #[rstest]
    #[case("高い", "高くない")]
    #[case("新しい", "新しくない")]
    fn kunai_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_kunai_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高かった")]
    #[case("新しい", "新しかった")]
    fn katta_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_katta_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高くなかった")]
    fn kunakatta_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_kunakatta_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高くて")]
    fn kute_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_kute_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高く")]
    fn ku_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_ku_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高ければ")]
    fn kereba_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_kereba_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("静かな", "静かな")]
    fn na_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_na_form(input, &PartOfSpeech::NaAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("静かな", "静かで")]
    fn de_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_de_form(input, &PartOfSpeech::NaAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("静かな", "静かなら")]
    fn nara_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_nara_form(input, &PartOfSpeech::NaAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高すぎる")]
    fn sugiru_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_sugiru_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case("高い", "高なさそう")]
    fn nasasou_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_nasasou_form(input, &PartOfSpeech::IAdjective).unwrap(),
            expected
        );
    }
}
