use crate::ui_components::{
    Card, KanjiViewMode, KanjiWritingSection, MarkdownText, Tag, TagVariant, Text, TextSize,
    TypographyVariant,
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
pub fn KanjiCardItem(study_card: StudyCard) -> impl IntoView {
    let card = study_card.card();
    let memory = study_card.memory();

    let (kanji_char, description, radicals, example_words) = match card {
        DomainCard::Kanji(kanji_card) => {
            let radicals_str = kanji_card
                .radicals_info()
                .ok()
                .map(|r| r.iter().map(|rad| rad.radical()).collect::<String>())
                .unwrap_or_default();
            let examples_str = kanji_card
                .example_words()
                .iter()
                .map(|w| format!("{} ({})", w.word(), w.meaning()))
                .collect::<Vec<_>>()
                .join(", ");
            (
                kanji_card.kanji().text().to_string(),
                kanji_card.description().text().to_string(),
                radicals_str,
                examples_str,
            )
        }
        _ => (
            "?".to_string(),
            "?".to_string(),
            "".to_string(),
            "".to_string(),
        ),
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

    let kanji_for_animation = StoredValue::new(kanji_char.clone());

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex justify-between items-start">
                <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                        <span class="text-3xl font-serif">{kanji_char.clone()}</span>
                        <div class=Signal::derive(|| "flex-1".to_string())>
                            <MarkdownText content=Signal::derive(move || description.clone())/>
                        </div>
                        <Tag variant=Signal::derive(move || status.tag_variant())>
                            {status.label()}
                        </Tag>
                    </div>
                    {move || {
                        if !radicals.is_empty() {
                            view! {
                                <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-1".to_string())>
                                    {format!("Радикалы: {}", radicals)}
                                </Text>
                            }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                    {move || {
                        if !example_words.is_empty() {
                            let examples = example_words.clone();
                            view! {
                                <div class=Signal::derive(|| "mb-1".to_string())>
                                    <MarkdownText content=Signal::derive(move || format!("**Примеры:** {}", examples))/>
                                </div>
                            }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        class=Signal::derive(|| "mt-2".to_string())
                    >
                        {format!("Повтор: {} | Слож: {} | Стаб: {}", next_review, difficulty, stability)}
                    </Text>
                </div>
            </div>
            <KanjiWritingSection kanji=kanji_for_animation.get_value() mode=KanjiViewMode::Frames />
        </Card>
    }
}
