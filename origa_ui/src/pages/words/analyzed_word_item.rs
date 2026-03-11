use crate::pages::icons::{
    CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN, ICON_CLASS_NEW, PLUS_CIRCLE_ICON, TOOLTIP_KNOWN,
    TOOLTIP_NEW,
};
use crate::ui_components::{Checkbox, FuriganaText, Text, TextSize, Tooltip, TypographyVariant};
use leptos::prelude::*;
use origa::domain::User;
use origa::use_cases::AnalyzedWord;
use std::collections::HashSet;

#[component]
pub fn AnalyzedWordItem(
    analyzed_word: AnalyzedWord,
    selected_words: RwSignal<HashSet<String>>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let base_form = analyzed_word.base_form.clone();
    let is_selected = Memo::new(move |_| selected_words.get().contains(&base_form));

    let (status_icon, tooltip_text, icon_class) = if analyzed_word.is_known {
        (CHECK_CIRCLE_ICON, TOOLTIP_KNOWN, ICON_CLASS_KNOWN)
    } else {
        (PLUS_CIRCLE_ICON, TOOLTIP_NEW, ICON_CLASS_NEW)
    };

    view! {
        <div class="flex justify-between items-center py-1 px-2 rounded bg-[var(--bg-secondary)]">
            <div class="flex items-center gap-2">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get())
                    on_change=Callback::new(move |_| on_toggle.run(()))
                />
                <FuriganaText text=analyzed_word.base_form.clone() known_kanji=known_kanji.get()/>
                <Tooltip text=Signal::derive(|| tooltip_text.to_string())>
                    <span class=icon_class inner_html=status_icon />
                </Tooltip>
            </div>
            {move || {
                analyzed_word.known_meaning.clone().map(|meaning| {
                    view! {
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {meaning}
                        </Text>
                    }
                })
            }}
        </div>
    }
}
