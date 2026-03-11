use crate::pages::icons::{
    CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN, ICON_CLASS_NEW, PLUS_CIRCLE_ICON, TOOLTIP_KNOWN,
    TOOLTIP_NEW,
};
use crate::ui_components::{Checkbox, FuriganaText, Text, TextSize, Tooltip, TypographyVariant};
use leptos::prelude::*;
use origa::domain::User;
use std::collections::HashSet;

#[component]
pub fn SetWordItem(
    word: String,
    known_meaning: Option<String>,
    is_known: bool,
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

    let word_for_memo = word.clone();
    let is_selected = Memo::new(move |_| selected_words.get().contains(&word_for_memo));

    let (status_icon, tooltip_text, icon_class) = if is_known {
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
                <FuriganaText text=word.clone() known_kanji=known_kanji.get()/>
                <Tooltip text=Signal::derive(|| tooltip_text.to_string())>
                    <span class=icon_class inner_html=status_icon />
                </Tooltip>
            </div>
            {move || {
                known_meaning.clone().map(|meaning| {
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
