use crate::ui_components::TagVariant;
use origa::domain::Card as DomainCard;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum CardType {
    #[default]
    Vocabulary,
    Kanji,
    Grammar,
    Radical,
}

impl CardType {
    pub fn label(&self) -> &'static str {
        match self {
            CardType::Vocabulary => "Слово",
            CardType::Kanji => "Кандзи",
            CardType::Grammar => "Грамматика",
            CardType::Radical => "Радикал",
        }
    }

    pub fn tag_variant(&self) -> TagVariant {
        match self {
            CardType::Vocabulary => TagVariant::Default,
            CardType::Kanji => TagVariant::Olive,
            CardType::Grammar => TagVariant::Terracotta,
            CardType::Radical => TagVariant::Default,
        }
    }
}

impl From<&DomainCard> for CardType {
    fn from(card: &DomainCard) -> Self {
        match card {
            DomainCard::Vocabulary(_) => CardType::Vocabulary,
            DomainCard::Kanji(_) => CardType::Kanji,
            DomainCard::Grammar(_) => CardType::Grammar,
            DomainCard::Radical(_) => CardType::Radical,
        }
    }
}
