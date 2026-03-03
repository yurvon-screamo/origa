use super::super::shared::CardStatus;
use crate::ui_components::{
    Button, ButtonVariant, Card, CardHistoryModal, FavoriteButton, HistoryButton, KanjiViewMode,
    KanjiWritingSection, MarkdownText, Tag, Text, TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};
use origa::domain::{Card as DomainCard, StudyCard};
use ulid::Ulid;

#[component]
pub fn KanjiCardItem(study_card: StudyCard, on_toggle_favorite: Callback<Ulid>) -> impl IntoView {
    let card = study_card.card();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();
    let memory_clone = memory.clone();

    let is_history_open = RwSignal::new(false);

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
    let is_expanded = RwSignal::new(false);
    let has_examples = !example_words.is_empty();

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex justify-between items-start">
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-3 mb-2">
                        <span class="text-3xl font-serif">{kanji_char.clone()}</span>
                        <div class="min-w-0 flex-1">
                            <MarkdownText content=Signal::derive(move || description.clone())/>
                        </div>
                        <Tag variant=Signal::derive(move || status.tag_variant())>
                            {status.label()}
                        </Tag>
                        <FavoriteButton
                            is_favorite=Signal::derive(move || is_favorite)
                            on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        />
                        <HistoryButton on_click=Callback::new(move |_| is_history_open.set(true)) />
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
                        if has_examples && is_expanded.get() {
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
                    <Show when=move || has_examples>
                        <div class="mt-2 flex items-center gap-3">
                            <Button
                                variant=ButtonVariant::Ghost
                                on_click=Callback::new(move |_: MouseEvent| {
                                    is_expanded.update(|v| *v = !*v);
                                })
                            >
                                {move || if is_expanded.get() { "Свернуть" } else { "Развернуть" }}
                            </Button>
                        </div>
                    </Show>
                </div>
            </div>
            {move || {
                if is_expanded.get() {
                    view! {
                        <KanjiWritingSection kanji=kanji_for_animation.get_value() mode=KanjiViewMode::Frames />
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            <CardHistoryModal
                is_open=Signal::derive(move || is_history_open.get())
                memory=memory_clone.clone()
                on_close=Callback::new(move |_| is_history_open.set(false))
            />
        </Card>
    }
}
