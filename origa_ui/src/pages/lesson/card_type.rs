use crate::i18n::*;
use crate::ui_components::TagVariant;
use origa::domain::Card as DomainCard;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum CardType {
    #[default]
    Vocabulary,
    Kanji,
    Grammar,
}

impl CardType {
    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            CardType::Vocabulary => i18n.get_keys().lesson().word().inner().to_string(),
            CardType::Kanji => i18n.get_keys().lesson().kanji().inner().to_string(),
            CardType::Grammar => i18n.get_keys().lesson().grammar().inner().to_string(),
        }
    }

    pub fn tag_variant(&self) -> TagVariant {
        match self {
            CardType::Vocabulary => TagVariant::Default,
            CardType::Kanji => TagVariant::Olive,
            CardType::Grammar => TagVariant::Terracotta,
        }
    }
}

impl From<&DomainCard> for CardType {
    fn from(card: &DomainCard) -> Self {
        match card {
            DomainCard::Vocabulary(_) => CardType::Vocabulary,
            DomainCard::Kanji(_) => CardType::Kanji,
            DomainCard::Grammar(_) => CardType::Grammar,
        }
    }
}
