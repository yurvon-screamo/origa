use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, Heading, HeadingLevel, MarkdownText, MarkdownVariant,
    Text, TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};

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
) -> impl IntoView {
    let question = StoredValue::new(question_text);
    let answer = StoredValue::new(answer_text);

    view! {
        <div class="text-center">
            <Show when=move || !is_kanji>
                <Heading level=HeadingLevel::H3 class="mb-2">
                    <Show
                        when=move || is_reversed
                        fallback=move || {
                            view! { <FuriganaText text=question.get_value()/> }
                        }
                    >
                        <MarkdownText
                            content=Signal::derive(move || question.get_value())
                            variant=Signal::derive(|| MarkdownVariant::Large)
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
                            />
                        }
                    }
                >
                    <FuriganaText text=answer.get_value()/>
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
