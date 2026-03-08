use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection,
    MarkdownText, MarkdownVariant, ReadingGroup, Text, TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};
use origa::domain::{GrammarInfo, User};

use super::grammar_info_badge::GrammarInfoBadge;

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
    grammar_info: Option<GrammarInfo>,
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
    let grammar_info_stored = StoredValue::new(grammar_info);

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
                <div class="max-w-max mx-auto space-y-4">
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

                    <Show when=move || is_kanji>
                        <div class="space-y-4">
                            <div class="space-y-3">
                                <ReadingGroup label="音読み[онъёми]" readings=on_readings_stored />
                                <ReadingGroup label="訓読み[кунъёми]" readings=kun_readings_stored />
                            </div>
                            <div class="flex gap-4 items-start text-left">
                                <div class="w-16 shrink-0">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        "Написание"
                                    </Text>
                                </div>
                                <KanjiWritingSection
                                    kanji=question.get_value()
                                    mode=KanjiViewMode::Frames
                                />
                            </div>
                        </div>
                    </Show>
                </div>
            </div>

            <Show when=move || grammar_info_stored.get_value().is_some()>
                {move || {
                    grammar_info_stored.get_value().map(|info| {
                        view! {
                            <GrammarInfoBadge
                                title=info.title().to_string()
                                description=info.description().to_string()
                            />
                        }
                    })
                }}
            </Show>

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
