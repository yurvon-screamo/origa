use crate::i18n::*;
use origa::domain::{Card as DomainCard, NativeLanguage};
use ulid::Ulid;

#[derive(Clone)]
pub(super) struct ScoringCard {
    pub card_id: Ulid,
    pub question: String,
    pub answer: String,
}

pub(super) fn extract_card_data(
    card: &DomainCard,
    lang: &NativeLanguage,
    i18n: I18nContext<Locale>,
) -> (String, String) {
    let locale = i18n.get_locale();
    let no_translation = || td_string!(locale, common.no_translation).to_string();
    match card {
        DomainCard::Vocabulary(v) => (
            v.word().text().to_string(),
            v.answer(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(no_translation),
        ),
        DomainCard::Kanji(k) => (
            k.kanji().text().to_string(),
            k.description()
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(no_translation),
        ),
        DomainCard::Grammar(g) => (
            g.title(lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(),
            g.description(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(no_translation),
        ),
    }
}

pub(super) fn build_scoring_cards(
    study_cards: &std::collections::HashMap<Ulid, origa::domain::StudyCard>,
    lang: &NativeLanguage,
    i18n: I18nContext<Locale>,
) -> Vec<ScoringCard> {
    study_cards
        .values()
        .filter(|sc| sc.memory().is_new())
        .map(|sc| {
            let (question, answer) = extract_card_data(sc.card(), lang, i18n);
            ScoringCard {
                card_id: *sc.card_id(),
                question,
                answer,
            }
        })
        .collect()
}
