use super::kanji_card_details::{KanjiCardDetails, RadicalDisplay};
use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, Heading, HeadingLevel, MarkdownText, MarkdownVariant,
    Text, TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};
use std::collections::HashSet;

#[component]
pub fn LessonCardAnswer(
    question_text: String,
    answer_text: String,
    is_expanded: RwSignal<bool>,
    needs_collapse: RwSignal<bool>,
    content_ref: NodeRef<leptos::html::Div>,
    on_toggle: Callback<()>,
    is_kanji: bool,
    is_reversed: bool,
    on_readings: Option<Vec<String>>,
    kun_readings: Option<Vec<String>>,
    radicals: Option<Vec<RadicalDisplay>>,
    example_words: Option<Vec<(String, String)>>,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let question = StoredValue::new(question_text);
    let answer = StoredValue::new(answer_text);
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);
    let radicals_stored = StoredValue::new(radicals);
    let examples_stored = StoredValue::new(example_words);

    view! {
        <div class="text-center">
            <Show
                when=move || is_kanji
                fallback=move || {
                    view! {
                        <Heading level=HeadingLevel::H3 class="mb-2">
                            <Show
                                when=move || is_reversed
                                fallback=move || {
                                    view! {
                                        <FuriganaText
                                            text=question.get_value()
                                            known_kanji=known_kanji.get()
                                            class=Signal::derive(|| "text-3xl leading-snug".to_string())
                                        />
                                    }
                                }
                            >
                                <MarkdownText
                                    content=Signal::derive(move || question.get_value())
                                    variant=Signal::derive(|| MarkdownVariant::Large)
                                    known_kanji=known_kanji.get()
                                />
                            </Show>
                        </Heading>
                    }
                }
            >
                <Heading level=HeadingLevel::H1 class="text-6xl mb-2 text-primary">
                    {question.get_value()}
                </Heading>
            </Show>

            <div
                node_ref=content_ref
                class=move || if is_expanded.get() { "border-t border-[var(--border-light)] pt-4 mt-4" } else { "border-t border-[var(--border-light)] pt-4 mt-4 line-clamp-3" }
            >
                <div class="max-w-max mx-auto space-y-4">
                    <Show
                        when=move || is_kanji
                        fallback=move || {
                            view! {
                                <div class="flex gap-4 items-baseline text-left">
                                    <div class="w-16 shrink-0">
                                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                            "Ответ:"
                                        </Text>
                                    </div>
                                    <Show
                                        when=move || is_reversed
                                        fallback=move || {
                                            view! {
                                                <MarkdownText
                                                    content=Signal::derive(move || answer.get_value())
                                                    variant=Signal::derive(|| MarkdownVariant::Large)
                                                    known_kanji=known_kanji.get()
                                                />
                                            }
                                        }
                                    >
                                        <FuriganaText text=answer.get_value() known_kanji=known_kanji.get()/>
                                    </Show>
                                </div>
                            }
                        }
                    >
                        <KanjiCardDetails
                            kanji=question.get_value()
                            radicals=radicals_stored.get_value()
                            example_words=examples_stored.get_value()
                            show_details=is_expanded
                            on_readings=on_readings_stored.get_value()
                            kun_readings=kun_readings_stored.get_value()
                            known_kanji=known_kanji
                        />
                    </Show>
                </div>
            </div>

            <Show when=move || needs_collapse.get()>
                <div class="mt-2">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |_: MouseEvent| on_toggle.run(()))
                    >
                        {move || if is_expanded.get() { "Свернуть" } else { "Развернуть" }}
                    </Button>
                </div>
            </Show>
        </div>
    }
}
