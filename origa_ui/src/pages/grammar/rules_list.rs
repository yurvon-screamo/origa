use super::rule_item::RuleItem;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::use_cases::GrammarRuleItem;
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RulesList(
    rules: Vec<GrammarRuleItem>,
    selected_ids: RwSignal<HashSet<Ulid>>,
    search_query: RwSignal<String>,
) -> impl IntoView {
    let filtered_rules = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            return rules.clone();
        }
        rules
            .iter()
            .filter(|rule| {
                rule.title.to_lowercase().contains(&query)
                    || rule.short_description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="space-y-2 max-h-64 overflow-y-auto">
            {move || {
                let filtered = filtered_rules();
                if filtered.is_empty() {
                    view! {
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Нет правил для выбранного уровня"
                        </Text>
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || filtered.clone()
                            key=|rule| rule.rule_id
                            children=move |rule| {
                                view! { <RuleItem rule=rule selected_ids=selected_ids /> }
                            }
                        />
                    }.into_any()
                }
            }}
        </div>
    }
}
