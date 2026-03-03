use crate::ui_components::TagVariant;
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

    pub fn label(&self) -> &'static str {
        match self {
            CardStatus::New => "Новое",
            CardStatus::Hard => "Сложное",
            CardStatus::InProgress => "В процессе",
            CardStatus::Learned => "Изучено",
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
