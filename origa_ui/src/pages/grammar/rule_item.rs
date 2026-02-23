use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::application::GrammarRuleItem;
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RuleItem(rule: GrammarRuleItem, selected_ids: RwSignal<HashSet<Ulid>>) -> impl IntoView {
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
                "p-3 border cursor-pointer mb-2 {}",
                if is_selected() { "border-olive bg-warm" } else { "border-dark bg-paper" }
            )
            on:click=on_click
        >
            <div class="font-bold text-sm"><FuriganaText text=rule.title/></div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {rule.short_description}
            </Text>
        </div>
    }
}
