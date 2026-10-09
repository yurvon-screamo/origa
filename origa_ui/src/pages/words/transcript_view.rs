//! Transcript screen (AU-2): sentence list with selection checkboxes for
//! long audio transcriptions. Pure display — the selection state lives in
//! the modal.

use leptos::prelude::*;

use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};

#[component]
pub fn TranscriptStageView(
    /// The full sentence list of the current transcription.
    #[prop(into)]
    sentences: Signal<Vec<String>>,
    /// Selected sentence indices.
    selected: RwSignal<std::collections::HashSet<usize>>,
    /// Analyze the selected sentences.
    on_analyze_selected: Callback<()>,
    /// Analyze the whole transcript in one step.
    on_use_all: Callback<()>,
    /// Back to the manual sources without analyzing anything.
    on_back: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let selected_count = move || selected.with(|set| set.len());

    let on_toggle = move |index: usize| {
        selected.update(|set| {
            if set.contains(&index) {
                set.remove(&index);
            } else {
                set.insert(index);
            }
        });
    };

    // For's each: prepared outside the view macro (the turbofish inside the
    // attribute confuses the view parser).
    let sentences_list = move || sentences.get().into_iter().enumerate().collect::<Vec<_>>();

    view! {
        <div class="space-y-4" data-testid="words-transcript-stage">
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {move || {
                    i18n.get_keys()
                        .words()
                        .transcript()
                        .selected_count()
                        .inner()
                        .to_string()
                        .replacen("{}", &selected_count().to_string(), 1)
                }}
            </Text>

            <div
                class="space-y-2 overflow-y-auto max-h-80"
                data-testid="words-transcript-list"
            >
                <For
                    each=sentences_list
                    key=|(index, sentence)| (*index, sentence.clone())
                    children=move |(index, sentence)| {
                        let on_toggle = on_toggle;
                        view! {
                            <label class="flex gap-2 items-start p-2 cursor-pointer bg-[var(--bg-secondary)]">
                                <input
                                    type="checkbox"
                                    class="mt-1"
                                    prop:checked=move || {
                                        selected.with(|set| set.contains(&index))
                                    }
                                    on:change=move |_| on_toggle(index)
                                    data-testid="words-transcript-sentence"
                                />
                                <span class="text-sm">{sentence}</span>
                            </label>
                        }
                    }
                />
            </div>

            <div class="flex gap-2 justify-between">
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_| on_back.run(()))
                    test_id="words-transcript-back-btn"
                >
                    {t!(i18n, common.back)}
                </Button>
                <Button
                    variant=ButtonVariant::Ghost
                    disabled=Signal::derive(move || selected.with(|set| set.is_empty()))
                    on_click=Callback::new(move |_| on_use_all.run(()))
                    test_id="words-transcript-use-all-btn"
                >
                    {t!(i18n, words.transcript.use_all_text)}
                </Button>
                <Button
                    variant=ButtonVariant::Olive
                    disabled=Signal::derive(move || selected.with(|set| set.is_empty()))
                    on_click=Callback::new(move |_| on_analyze_selected.run(()))
                    test_id="words-transcript-analyze-btn"
                >
                    {t!(i18n, words.transcript.analyze_selected)}
                </Button>
            </div>
        </div>
    }
}
