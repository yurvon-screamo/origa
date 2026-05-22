use origa::domain::{Card as DomainCard, PartOfSpeech};
use tracing::warn;

const STATEMENT_SEPARATOR: &str = " \n ";

/// Checks if the given card is a Vocabulary na-adjective.
/// Returns false for non-vocabulary cards or on error (with warning log).
pub fn is_na_adjective_card(card: &DomainCard) -> bool {
    let DomainCard::Vocabulary(vocab) = card else {
        return false;
    };
    match vocab.part_of_speech() {
        Ok(PartOfSpeech::NaAdjective) => true,
        Ok(_) => false,
        Err(e) => {
            warn!(
                word = %vocab.word().text(),
                error = %e,
                "Failed to determine part of speech for na-adjective check"
            );
            false
        },
    }
}

/// Appends な to the text for na-adjectives.
/// For statement-style text (containing STATEMENT_SEPARATOR), only the first part is modified.
/// Does NOT double な if text already ends with it.
pub fn append_na_suffix(text: &str) -> String {
    if let Some((word_part, rest)) = text.split_once(STATEMENT_SEPARATOR) {
        return format!(
            "{}な{}{}",
            ensure_no_trailing_na(word_part),
            STATEMENT_SEPARATOR,
            rest
        );
    }
    format!("{}な", ensure_no_trailing_na(text))
}

fn ensure_no_trailing_na(text: &str) -> &str {
    text.strip_suffix("な").unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_na_suffix_adds_na() {
        assert_eq!(append_na_suffix("静か"), "静かな");
    }

    #[test]
    fn append_na_suffix_does_not_double_na() {
        assert_eq!(append_na_suffix("静かな"), "静かな");
    }

    #[test]
    fn append_na_suffix_handles_statement_format() {
        assert_eq!(append_na_suffix("静か \n quiet"), "静かな \n quiet");
    }

    #[test]
    fn append_na_suffix_statement_does_not_double_na() {
        assert_eq!(append_na_suffix("静かな \n quiet"), "静かな \n quiet");
    }

    #[test]
    fn append_na_suffix_empty_string() {
        assert_eq!(append_na_suffix(""), "な");
    }

    #[test]
    fn ensure_no_trailing_na_strips_na() {
        assert_eq!(ensure_no_trailing_na("静かな"), "静か");
    }

    #[test]
    fn ensure_no_trailing_na_keeps_text_without_na() {
        assert_eq!(ensure_no_trailing_na("静か"), "静か");
    }
}
