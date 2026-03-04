use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::application::GrammarRuleItem;
use origa::domain::User;
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RuleItem(rule: GrammarRuleItem, selected_ids: RwSignal<HashSet<Ulid>>) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let rule_id = rule.rule_id;
    let is_selected = move || selected_ids.get().contains(&rule_id);

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
            on:click=on_click
        >
            <div class="font-bold text-sm font-mono"><FuriganaText text=rule.title known_kanji=known_kanji.get()/></div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {rule.short_description}
            </Text>
        </div>
    }
}
