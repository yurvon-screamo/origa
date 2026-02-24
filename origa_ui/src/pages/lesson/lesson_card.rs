use crate::ui_components::{
    Button, ButtonVariant, Card, FuriganaText, Heading, HeadingLevel, Tag, TagVariant, Text,
    TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::Card as DomainCard;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum CardType {
    #[default]
    Vocabulary,
    Kanji,
    Grammar,
}

impl CardType {
    pub fn label(&self) -> &'static str {
        match self {
            CardType::Vocabulary => "Слово",
            CardType::Kanji => "Кандзи",
            CardType::Grammar => "Грамматика",
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

#[component]
pub fn LessonCard(
    card: DomainCard,
    show_answer: bool,
    on_show_answer: Callback<()>,
) -> impl IntoView {
    let card_type = CardType::from(&card);
    let question = StoredValue::new(card.question().text().to_string());
    let answer = StoredValue::new(card.answer().text().to_string());

    let radicals: Option<String> = match &card {
        DomainCard::Kanji(kanji) => kanji.radicals_info().ok().map(|r| {
            r.iter()
                .map(|info| info.radical().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }),
        _ => None,
    };
    let radicals = StoredValue::new(radicals);

    view! {
        <Card class=Signal::derive(|| "p-6 min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <div class="flex items-center gap-2 mb-4">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label()}
                </Tag>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <Show when=move || !show_answer>
                    <div class="text-center">
                        <Heading level=HeadingLevel::H2 class="mb-4">
                            <Show when=move || card_type == CardType::Kanji>
                                {question.get_value()}
                            </Show>
                            <Show when=move || card_type != CardType::Kanji>
                                <FuriganaText text=question.get_value()/>
                            </Show>
                        </Heading>

                        <Button
                            variant=Signal::derive(|| ButtonVariant::Filled)
                            on_click=Callback::new(move |_| on_show_answer.run(()))
                        >
                            "Показать ответ [Пробел]"
                        </Button>
                    </div>
                </Show>

                <Show when=move || show_answer>
                    <div class="text-center">
                        <Heading level=HeadingLevel::H3 class="mb-2">
                            <Show when=move || card_type == CardType::Kanji>
                                {question.get_value()}
                            </Show>
                            <Show when=move || card_type != CardType::Kanji>
                                <FuriganaText text=question.get_value()/>
                            </Show>
                        </Heading>

                        <Show when=move || radicals.get_value().is_some()>
                            <div class="mb-4">
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    {format!("Радикалы: {}", radicals.get_value().unwrap_or_default())}
                                </Text>
                            </div>
                        </Show>

                        <div class="border-t border-[var(--border-light)] pt-4 mt-4">
                            <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-2">
                                "Ответ:"
                            </Text>
                            <Text size=TextSize::Large variant=TypographyVariant::Primary>
                                <Show when=move || card_type == CardType::Kanji>
                                    {answer.get_value()}
                                </Show>
                                <Show when=move || card_type != CardType::Kanji>
                                    <FuriganaText text=answer.get_value()/>
                                </Show>
                            </Text>
                        </div>
                    </div>
                </Show>
            </div>
        </Card>
    }
}
