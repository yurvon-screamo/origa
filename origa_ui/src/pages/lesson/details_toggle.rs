use crate::i18n::*;
use leptos::prelude::*;

/// The lesson «Подробнее / Свернуть» disclosure toggle: the centered mono
/// text button with the muted underline style. The single home of the
/// markup — the kanji details, the grammar expand and the counter
/// readings disclosure render it (was three hand copies, rule of three).
#[component]
pub(super) fn LessonDetailsToggle(
    /// Expand/collapse state flipped on click; the label derives from it.
    expanded: RwSignal<bool>,
    /// Optional test id; empty = no attribute (the Card/Button convention).
    #[prop(optional, into)]
    test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let more_text =
        Signal::derive(move || i18n.get_keys().common().more_details().inner().to_string());
    let collapse_text =
        Signal::derive(move || i18n.get_keys().common().collapse().inner().to_string());
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="flex justify-center">
            <button
                class="font-mono text-sm text-[var(--fg-muted)] cursor-pointer hover:text-[var(--fg-black)] underline underline-offset-4 decoration-[var(--border-light)]"
                data-testid=test_id_val
                on:click=move |_| expanded.update(|v| *v = !*v)
            >
                {move || if expanded.get() { collapse_text.get() } else { more_text.get() }}
            </button>
        </div>
    }
}
