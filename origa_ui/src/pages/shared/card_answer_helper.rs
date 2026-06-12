use origa::domain::{Card, CardAnswer, NativeLanguage};

/// Extract plain-text representation of a card answer for display
/// (search, metrics, card lists).
///
/// Returns `translations.join(", ")` for Vocabulary,
/// text for Text, or an empty string on error.
pub fn format_answer_text(card: &Card, lang: &NativeLanguage) -> String {
    match card.answer(lang) {
        Ok(CardAnswer::Vocabulary { translations, .. }) => translations.join(", "),
        Ok(CardAnswer::Text(s)) => s,
        Err(_) => String::new(),
    }
}

/// Extract translations + description for WordTranslations component.
///
/// Returns `(translations, description)` for Vocabulary,
/// `(vec![text], None)` for Text, or `(vec![], None)` on error.
pub fn format_answer_parts(card: &Card, lang: &NativeLanguage) -> (Vec<String>, Option<String>) {
    match card.answer(lang) {
        Ok(CardAnswer::Vocabulary {
            translations,
            description,
        }) => (translations, description),
        Ok(CardAnswer::Text(s)) => (vec![s], None),
        Err(_) => (vec![], None),
    }
}
