use crate::i18n::*;
use crate::pages::lesson::card_type::CardType;
use origa::domain::{Card as DomainCard, NativeLanguage};
use ulid::Ulid;

#[derive(Clone)]
pub(super) struct ScoringCard {
    pub card_id: Ulid,
    pub question: String,
    pub answer: String,
    pub card_type: CardType,
}

pub(super) fn extract_card_data(
    card: &DomainCard,
    lang: &NativeLanguage,
    i18n: I18nContext<Locale>,
) -> (String, String, CardType) {
    let locale = i18n.get_locale();
    let no_translation = || td_string!(locale, common.no_translation).to_string();
    match card {
        DomainCard::Vocabulary(v) => (v.word().text().to_string(), v.answer(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(no_translation), CardType::Vocabulary),
        DomainCard::Kanji(k) => (k.kanji().text().to_string(), k.description(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(no_translation), CardType::Kanji),
        DomainCard::Grammar(g) => (g.title(lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(), g.description(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(no_translation), CardType::Grammar),
        DomainCard::Phrase(p) => (p.question().unwrap_or_default(), p.answer(lang).unwrap_or_else(no_translation), CardType::Phrase),
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
            let (question, answer, card_type) = extract_card_data(sc.card(), lang, i18n);
            ScoringCard {
                card_id: *sc.card_id(),
                question,
                answer,
                card_type,
            }
        })
        .collect()
}
