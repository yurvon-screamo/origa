use crate::ui_components::{
    Card, Heading, HeadingLevel, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, StudyCard};

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

#[component]
pub fn VocabularyCardItem(study_card: StudyCard) -> impl IntoView {
    let card = study_card.card();
    let memory = study_card.memory();

    let (word, meaning) = match card {
        DomainCard::Vocabulary(vocab) => (
            vocab.word().text().to_string(),
            vocab.meaning().text().to_string(),
        ),
        _ => ("?".to_string(), "?".to_string()),
    };

    let status = CardStatus::from_study_card(&study_card);

    let difficulty = memory
        .difficulty()
        .map(|d| format!("{:.1}", d.value()))
        .unwrap_or("-".to_string());
    let stability = memory
        .stability()
        .map(|s| format!("{:.1}", s.value()))
        .unwrap_or("-".to_string());
    let next_review = memory
        .next_review_date()
        .map(|d| d.format("%d.%m.%Y").to_string())
        .unwrap_or("-".to_string());

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex justify-between items-start">
                <div class="flex-1">
                    <div class="flex items-center gap-2 mb-2">
                        <Heading level=HeadingLevel::H4>
                            {word}
                        </Heading>
                        <Tag variant=Signal::derive(move || status.tag_variant())>
                            {status.label()}
                        </Tag>
                    </div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {meaning}
                    </Text>
                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        class=Signal::derive(|| "mt-2".to_string())
                    >
                        {format!("Повтор: {} | Слож: {} | Стаб: {}", next_review, difficulty, stability)}
                    </Text>
                </div>
            </div>
        </Card>
    }
}
