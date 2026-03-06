use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, Heading, HeadingLevel, MarkdownText, MarkdownVariant,
    Text, TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};
use origa::domain::User;

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
) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let question = StoredValue::new(question_text);
    let answer = StoredValue::new(answer_text);
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);

    view! {
        <div class="text-center">
            <Show when=move || !is_kanji>
                <Heading level=HeadingLevel::H3 class="mb-2">
                    <Show
                        when=move || is_reversed
                        fallback=move || {
                            view! { <FuriganaText text=question.get_value() known_kanji=known_kanji.get()/> }
                        }
                    >
                        <MarkdownText
                            content=Signal::derive(move || question.get_value())
                            variant=Signal::derive(|| MarkdownVariant::Large)
                            known_kanji=known_kanji.get()
                        />
                    </Show>
                </Heading>
            </Show>

            <div
                node_ref=content_ref
                class=move || if is_expanded.get() { "border-t border-[var(--border-light)] pt-4 mt-4" } else { "border-t border-[var(--border-light)] pt-4 mt-4 line-clamp-3" }
            >
                <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-2">
                    "Ответ:"
                </Text>
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

                <Show when=move || is_kanji>
                    <div class="mt-4 space-y-3">
                        <Show when=move || on_readings_stored.get_value().is_some()>
                            <div class="flex gap-2 items-center justify-center flex-wrap">
                                <div class="flex gap-1.5 items-center">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        "音読み"
                                    </Text>
                                </div>
                                {on_readings_stored
                                    .get_value()
                                    .unwrap_or_default()
                                    .iter()
                                    .map(|reading| {
                                        view! {
                                            <span class="inline-block px-2 py-1 bg-[var(--bg-secondary)] rounded text-sm">
                                                {reading.clone()}
                                            </span>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        </Show>

                        <Show when=move || kun_readings_stored.get_value().is_some()>
                            <div class="flex gap-2 items-center justify-center flex-wrap">
                                <div class="flex gap-1.5 items-center">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        "訓読み"
                                    </Text>
                                </div>
                                {kun_readings_stored
                                    .get_value()
                                    .unwrap_or_default()
                                    .iter()
                                    .map(|reading| {
                                        view! {
                                            <span class="inline-block px-2 py-1 bg-[var(--bg-secondary)] rounded text-sm">
                                                {reading.clone()}
                                            </span>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        </Show>
                    </div>
                </Show>
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
