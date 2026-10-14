use crate::i18n::*;
use crate::ui_components::{Text, TextSize};
use leptos::prelude::*;

/// The «Не знаю [Space]» fallback of the quiz-family cards: a full-width
/// raw button under the options grid, ring-highlighted while selected.
/// Every caller renders it only on the question side (gated behind
/// `!show_result`), so it has two visual states: selected ring / neutral.
#[component]
pub fn DontKnowButton(
    dont_know_selected: Signal<bool>,
    /// Clicks pass through only while enabled; callers gate this with
    /// `!show_result` (the lesson state flips to the result side on answer).
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// The quiz options and the phrase quiz share `quiz-dont-know-btn`;
    /// the yes/no card passes its own id.
    #[prop(default = Signal::derive(|| "quiz-dont-know-btn".to_string()), into)]
    test_id: Signal<String>,
    on_click: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <button
            data-testid=test_id
            class=move || {
                let base = "w-full mt-2 p-2 sm:p-4 border text-center transition-all cursor-pointer flex items-center justify-center gap-2";
                if dont_know_selected.get() {
                    format!("{} quiz-option-neutral ring-2 ring-[var(--accent-olive)]", base)
                } else {
                    format!("{} quiz-option-neutral", base)
                }
            }
            on:click=move |_| {
                if !disabled.get() {
                    on_click.run(());
                }
            }
        >
            <Text size=TextSize::Default>{t!(i18n, lesson.dont_know)}</Text>
            <span class="kbd-hint text-[var(--fg-muted)] text-xs font-mono">
                {t!(i18n, lesson.space_key)}
            </span>
        </button>
    }
}
