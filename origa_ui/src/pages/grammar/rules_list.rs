use super::rule_item::RuleItem;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::application::GrammarRuleItem;
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RulesList(
    rules: Vec<GrammarRuleItem>,
    selected_ids: RwSignal<HashSet<Ulid>>,
) -> impl IntoView {
    if rules.is_empty() {
        return view! {
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                "Нет правил для выбранного уровня"
            </Text>
        }
        .into_any();
    }

    view! {
        <div class="space-y-2">
            <For
                each=move || rules.clone()
                key=|rule| rule.rule_id
                children=move |rule| {
                    view! { <RuleItem rule=rule selected_ids=selected_ids /> }
                }
            />
        </div>
    }
    .into_any()
}
