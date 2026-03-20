use crate::domain::{OrigaError, PartOfSpeech};

fn i_adj_stem(word: &str) -> &str {
    word.trim_end_matches("い")
}

fn na_adj_stem(word: &str) -> &str {
    word.trim_end_matches("な")
}

fn i_adj_form(
    word: &str,
    suffix: &str,
    part_of_speech: &PartOfSpeech,
) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => Ok(format!("{}{}", i_adj_stem(word), suffix)),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only IAdjective supported".to_string(),
        }),
    }
}

fn na_adj_form(
    word: &str,
    suffix: &str,
    part_of_speech: &PartOfSpeech,
) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::NaAdjective => Ok(format!("{}{}", na_adj_stem(word), suffix)),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only NaAdjective supported".to_string(),
        }),
    }
}

pub fn adjective_remove_postfix(
    word: &str,
    part_of_speech: &PartOfSpeech,
) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => Ok(format!("{}くなる", i_adj_stem(word))),
        PartOfSpeech::NaAdjective => Ok(format!("{}になる", na_adj_stem(word))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Not supported part of speech".to_string(),
        }),
    }
}

pub fn to_kunai_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "くない", part_of_speech)
}

pub fn to_katta_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "かった", part_of_speech)
}

pub fn to_kunakatta_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "くなかった", part_of_speech)
}

pub fn to_kute_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "くて", part_of_speech)
}

pub fn to_ku_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "く", part_of_speech)
}

pub fn to_kereba_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "ければ", part_of_speech)
}

pub fn to_sou_form_iadj(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "そう", part_of_speech)
}

pub fn to_sugiru_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    i_adj_form(word, "すぎる", part_of_speech)
}

pub fn to_na_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    na_adj_form(word, "な", part_of_speech)
}

pub fn to_de_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    na_adj_form(word, "で", part_of_speech)
}

pub fn to_nara_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    na_adj_form(word, "なら", part_of_speech)
}

pub fn to_sou_form_naadj(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    na_adj_form(word, "そう", part_of_speech)
}

pub fn to_nasasou_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => Ok(format!("{}なさそう", i_adj_stem(word))),
        PartOfSpeech::NaAdjective => Ok(format!("{}じゃなさそう", na_adj_stem(word))),
        _ => Err(OrigaError::GrammarFormatError {
            reason: "Only adjectives supported".to_string(),
        }),
    }
}

pub fn to_garu_form(word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
    match part_of_speech {
        PartOfSpeech::IAdjective => Ok(format!("{}がる", i_adj_stem(word))),
        PartOfSpeech::NaAdjective => Ok(format!("{}がる", na_adj_stem(word))),
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
