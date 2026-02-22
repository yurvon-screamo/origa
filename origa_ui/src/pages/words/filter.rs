use super::vocabulary_card_item::CardStatus;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Filter {
    #[default]
    All,
    New,
    Hard,
    InProgress,
    Learned,
}

impl Filter {
    pub fn label(&self) -> &'static str {
        match self {
            Filter::All => "Все",
            Filter::New => "Новые",
            Filter::Hard => "Сложные",
            Filter::InProgress => "В процессе",
            Filter::Learned => "Изученные",
        }
    }

    pub fn matches(&self, status: CardStatus) -> bool {
        match self {
            Filter::All => true,
            Filter::New => status == CardStatus::New,
            Filter::Hard => status == CardStatus::Hard,
            Filter::InProgress => status == CardStatus::InProgress,
            Filter::Learned => status == CardStatus::Learned,
        }
    }
}
