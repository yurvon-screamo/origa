use origa::domain::{Card as DomainCard, NativeLanguage};
use tracing::warn;

use super::card_type::CardType;

/// Card question text with a `warn!` on lookup failure (card_type +
/// content_key context). The single logging site for every lesson card
/// view — previously each view repeated the same `match` + log block.
pub(crate) fn card_question_text(
    card: &DomainCard,
    lang: &NativeLanguage,
    card_type: CardType,
) -> String {
    match card.question(lang) {
        Ok(q) => q.text().to_string(),
        Err(e) => {
            warn!(
                card_type = ?card_type,
                content_key = %card.content_key(),
                error = %e,
                "Failed to get card question"
            );
            String::new()
        },
    }
}

/// Question text as displayed on the question side: na-adjective words
/// get the な suffix appended (the question text is Japanese there).
/// NOT applicable to the reversed path of the normal lesson card — there
/// the question is a translation and must not receive the suffix.
pub(crate) fn display_question_text(
    card: &DomainCard,
    lang: &NativeLanguage,
    card_type: CardType,
) -> String {
    let text = card_question_text(card, lang, card_type);
    if super::na_adjective_helper::is_na_adjective_card(card) {
        super::na_adjective_helper::append_na_suffix(&text)
    } else {
        text
    }
}
