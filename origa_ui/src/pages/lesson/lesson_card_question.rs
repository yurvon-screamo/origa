use crate::ui_components::{
    Button, ButtonVariant, DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode,
    KanjiWritingSection, MarkdownText, MarkdownVariant,
};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn LessonCardQuestion(
    question_text: String,
    kanji: Option<String>,
    is_reversed: bool,
    on_show_answer: Callback<()>,
) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let question = StoredValue::new(question_text);
    let kanji_stored = StoredValue::new(kanji);

    view! {
        <div class="text-center">
            <Show when=move || kanji_stored.get_value().is_none()>
                <div class="mb-4">
                    <Heading level=HeadingLevel::H2>
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
                </div>
            </Show>

            <Show when=move || kanji_stored.get_value().is_some()>
                {move || {
                    kanji_stored.get_value().map(|k| {
                        let k_clone = k.clone();
                        view! {
                            <div class="mb-6">
                                <DisplayText>
                                    {k_clone}
                                </DisplayText>
                            </div>
                            <KanjiWritingSection kanji=k mode=KanjiViewMode::Animation />
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
    }
}
