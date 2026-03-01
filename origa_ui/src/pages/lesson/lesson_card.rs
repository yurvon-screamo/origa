use crate::ui_components::{
    get_reading_from_text, is_speech_supported, speak_text, AudioButtons, Button, ButtonVariant,
    Card, DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection,
    MarkdownText, MarkdownVariant, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};
use origa::domain::Card as DomainCard;

use super::lesson_state::LessonContext;

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

    let example_words: Option<Vec<(String, String)>> = match &card {
        DomainCard::Kanji(kanji) => {
            let examples: Vec<_> = kanji
                .example_words()
                .iter()
                .map(|e| (e.word().to_string(), e.meaning().to_string()))
                .collect();
            if examples.is_empty() {
                None
            } else {
                Some(examples)
            }
        }
        _ => None,
    };
    let example_words = StoredValue::new(example_words);

    let kanji_for_animation = StoredValue::new(match &card {
        DomainCard::Kanji(_) => Some(card.question().text().to_string()),
        _ => None,
    });

    let lesson_ctx = use_context::<LessonContext>();
    let question_text = question.get_value();

    let is_expanded = RwSignal::new(false);
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let needs_collapse = RwSignal::new(false);

    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_answer && card_type != CardType::Kanji && is_speech_supported() && !is_muted {
            let reading = get_reading_from_text(&question_text);
            let _ = speak_text(&reading, 1.0);
        }
    });

    Effect::new(move |_| {
        if show_answer {
            if let Some(el) = content_ref.get() {
                let is_overflow = el.scroll_height() > el.client_height();
                needs_collapse.set(is_overflow);
            }
        }
    });

    view! {
        <Card class=Signal::derive(|| "p-6 min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <div class="flex items-center justify-between mb-4">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label()}
                </Tag>
                <Show when=move || card_type != CardType::Kanji>
                    <AudioButtons
                        text=question.get_value()
                        class=Signal::derive(|| "".to_string())
                    />
                </Show>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <Show when=move || !show_answer>
                    <div class="text-center">
                        <Show when=move || card_type != CardType::Kanji>
                            <div class="mb-4">
                                <Heading level=HeadingLevel::H2>
                                    <FuriganaText text=question.get_value()/>
                                </Heading>
                            </div>
                        </Show>

                        <Show when=move || kanji_for_animation.get_value().is_some()>
                            {move || {
                                kanji_for_animation.get_value().map(|kanji| {
                                    let kanji_text = kanji.clone();
                                    view! {
                                        <div class="mb-6">
                                            <DisplayText>
                                                {kanji_text}
                                            </DisplayText>
                                        </div>
                                        <KanjiWritingSection kanji=kanji mode=KanjiViewMode::Animation />
                                    }
                                })
                            }}
                        </Show>

                        <Button
                            variant=Signal::derive(|| ButtonVariant::Filled)
                            on_click=Callback::new(move |_| on_show_answer.run(()))
                        >
                            "Показать ответ" <span class="hidden sm:inline">"[Пробел]"</span>
                        </Button>
                    </div>
                </Show>

                <Show when=move || show_answer>
                    <div class="text-center">
                        <Show when=move || card_type != CardType::Kanji>
                            <Heading level=HeadingLevel::H3 class="mb-2">
                                <FuriganaText text=question.get_value()/>
                            </Heading>
                        </Show>

                        <div
                            node_ref=content_ref
                            class=move || if is_expanded.get() { "border-t border-[var(--border-light)] pt-4 mt-4" } else { "border-t border-[var(--border-light)] pt-4 mt-4 line-clamp-3" }
                        >
                            <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-2">
                                "Ответ:"
                            </Text>
                            <MarkdownText content=Signal::derive(move || answer.get_value())/>
                        </div>

                        <Show when=move || is_expanded.get()>
                            <Show when=move || kanji_for_animation.get_value().is_some()>
                                {move || {
                                    kanji_for_animation.get_value().map(|kanji| view! {
                                        <KanjiWritingSection kanji=kanji mode=KanjiViewMode::Frames />
                                    })
                                }}
                            </Show>

                            <Show when=move || radicals.get_value().is_some()>
                                <div class="my-6">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        {format!("Радикалы: {}", radicals.get_value().unwrap_or_default())}
                                    </Text>
                                </div>
                            </Show>

                            <Show when=move || example_words.get_value().is_some()>
                                <div class="my-6">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-3 block text-left">
                                        "Примеры слов:"
                                    </Text>
                                    <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 text-left">
                                        {move || {
                                            example_words.get_value().map(|examples| {
                                                examples
                                                    .into_iter()
                                                    .map(|(word, meaning)| {
                                                        let meaning = StoredValue::new(meaning);
                                                        view! {
                                                            <div class="p-2 bg-[var(--bg-secondary)] rounded">
                                                                <Text size=TextSize::Default class="font-bold">
                                                                    <FuriganaText text=word />
                                                                </Text>
                                                                <MarkdownText
                                                                    content=Signal::derive(move || meaning.get_value())
                                                                    variant=MarkdownVariant::Compact
                                                                    class="text-[var(--fg-muted)]"
                                                                />
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                            })
                                        }}
                                    </div>
                                </div>
                            </Show>
                        </Show>

                        <Show when=move || needs_collapse.get()>
                            <div class="mt-2">
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
                </Show>
            </div>
        </Card>
    }
}
