use crate::pages::icons::{
    CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN, ICON_CLASS_NEW, PLUS_CIRCLE_ICON, TOOLTIP_KNOWN,
    TOOLTIP_NEW,
};
use crate::ui_components::{Checkbox, FuriganaText, MarkdownText, MarkdownVariant, Tooltip};
use leptos::prelude::*;
use origa::use_cases::AnalyzedWord;
use std::collections::HashSet;

#[component]
pub fn AnalyzedWordItem(
    analyzed_word: AnalyzedWord,
    selected_words: RwSignal<HashSet<String>>,
    known_kanji: HashSet<String>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let base_form = analyzed_word.base_form.clone();
    let is_selected = Memo::new(move |_| selected_words.get().contains(&base_form));

    let (status_icon, tooltip_text, icon_class) = if analyzed_word.is_known {
        (CHECK_CIRCLE_ICON, TOOLTIP_KNOWN, ICON_CLASS_KNOWN)
    } else {
        (PLUS_CIRCLE_ICON, TOOLTIP_NEW, ICON_CLASS_NEW)
    };

    view! {
        <div
            class="group flex items-start gap-4 py-3 px-4 border-b border-[var(--border-light)] hover:bg-[var(--bg-aged)] transition-colors cursor-pointer"
            on:click=move |_| on_toggle.run(())
        >
            <div class="pt-1">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get())
                    on_change=Callback::new(move |_| on_toggle.run(()))
                />
            </div>

            <div class="flex-1 flex flex-col gap-1">
                <div class="flex items-center gap-2">
                    <div class="text-xl font-serif tracking-wide">
                        <FuriganaText
                            text=analyzed_word.base_form.clone()
                            known_kanji=known_kanji.clone()
                        />
                    </div>

                        <Tooltip text=Signal::derive(|| tooltip_text.to_string())>
                        <span class=format!("{} opacity-60 group-hover:opacity-100 transition-opacity", icon_class)
                              inner_html=status_icon
                        />
                    </Tooltip>
                </div>

                {move || {
                    let known_kanji = known_kanji.clone();
                    analyzed_word.meaning.clone().map(move |meaning| {
                        view! {
                            <div class="max-w-md">
                                <MarkdownText
                                    content=Signal::derive(move || meaning.clone())
                                    known_kanji=known_kanji
                                    variant=MarkdownVariant::Compact
                                    class="text-[var(--fg-muted)]"
                                />
                            </div>
                        }
                    })
                }}
            </div>
        </div>
    }
}
