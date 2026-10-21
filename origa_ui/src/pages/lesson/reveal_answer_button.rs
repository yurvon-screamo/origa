use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

/// The universal «Показать ответ [Space]» reveal of the lesson: every
/// card/slide that hides its answer renders this button, and the shared
/// keyboard handlers bind Space to the same action. The single home of
/// the reveal-button markup and its kbd hint.
///
/// The default test id (`lesson-show-answer-btn`) is the standard lesson
/// card reveal; surfaces with their own DOM contract (acquaintance
/// training, audio recall, example recall, counter bindings) pass a
/// distinct id.
#[component]
pub fn RevealAnswerButton(
    #[prop(default = Signal::derive(|| "lesson-show-answer-btn".to_string()), into)]
    test_id: Signal<String>,
    /// Most surfaces reveal «Показать ответ»; the example recall reveals
    /// the sentence translation and passes its own label. An empty label
    /// falls back to the standard text (the Card/Button prop convention).
    #[prop(optional, into)]
    label: Signal<String>,
    on_click: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let label = Signal::derive(move || {
        let custom = label.get();
        if custom.is_empty() {
            t_string!(i18n, lesson.show_answer).to_string()
        } else {
            custom
        }
    });
    view! {
        <Button
            variant=Signal::derive(|| ButtonVariant::Filled)
            test_id=test_id
            on_click=Callback::new(move |_| on_click.run(()))
        >
            {move || label.get()} <span class="kbd-hint">{t!(i18n, lesson.space_key)}</span>
        </Button>
    }
}
