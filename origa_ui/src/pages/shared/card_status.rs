use crate::i18n::Locale;
use crate::ui_components::TagVariant;
use leptos_i18n::I18nContext;
use origa::domain::StudyCard;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum CardStatus {
    #[default]
    New,
    Hard,
    InProgress,
    Learned,
}

impl CardStatus {
    pub fn from_study_card(card: &StudyCard) -> Self {
        let memory = card.memory();
        if memory.is_new() {
            CardStatus::New
        } else if memory.is_high_difficulty() {
            CardStatus::Hard
        } else if memory.is_known_card() {
            CardStatus::Learned
        } else {
            CardStatus::InProgress
        }
    }

    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            CardStatus::New => i18n.get_keys().shared().status_new().inner().to_string(),
            CardStatus::Hard => i18n.get_keys().shared().status_hard().inner().to_string(),
            CardStatus::InProgress => i18n
                .get_keys()
                .shared()
                .status_in_progress()
                .inner()
                .to_string(),
            CardStatus::Learned => i18n
                .get_keys()
                .shared()
                .status_learned()
                .inner()
                .to_string(),
        }
    }

    pub fn tag_variant(&self) -> TagVariant {
        match self {
            CardStatus::New => TagVariant::Default,
            CardStatus::Hard => TagVariant::Terracotta,
            CardStatus::InProgress => TagVariant::Filled,
            CardStatus::Learned => TagVariant::Olive,
        }
    }
}
