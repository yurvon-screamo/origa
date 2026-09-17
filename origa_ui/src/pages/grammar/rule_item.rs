use crate::ui_components::FuriganaText;
use leptos::prelude::*;
use origa::dictionary::grammar::GrammarRule;
use origa::domain::NativeLanguage;
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RuleItem(
    rule: &'static GrammarRule,
    native_language: NativeLanguage,
    selected_ids: RwSignal<HashSet<Ulid>>,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let rule_id = *rule.rule_id();
    let is_selected = move || selected_ids.get().contains(&rule_id);
    let content = rule.content(&native_language);
    let title = content.pattern().to_string();
    let short_description = content.short_description().to_string();

    let on_click = move |_| {
        selected_ids.update(|ids| {
            if ids.contains(&rule_id) {
                ids.remove(&rule_id);
            } else {
                ids.insert(rule_id);
            }
        });
    };

    view! {
        <div
            class=move || format!(
                "p-3 border cursor-pointer mb-2 transition-all {}",
                if is_selected() {
                    "border-[var(--accent-olive)] bg-[var(--bg-warm)] shadow-[2px_2px_0_var(--accent-olive)]"
                } else {
                    "border-[var(--border-dark)] bg-[var(--bg-paper)]"
                }
            )
            data-testid=move || test_id.get()
            on:click=on_click
        >
            <div class="font-serif font-bold text-sm text-[var(--fg-black)]">
                {short_description}
            </div>
            <div class="font-mono text-sm text-[var(--fg-muted)]">
                <FuriganaText text=title known_kanji=known_kanji.clone()/>
            </div>
        </div>
    }
}
