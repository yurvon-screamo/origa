use crate::i18n::{t, use_i18n};
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::dictionary::grammar::GrammarRule;
use origa::domain::NativeLanguage;
use std::collections::HashSet;
use ulid::Ulid;

use super::rule_item::RuleItem;

#[component]
pub fn RulesList(
    rules: Vec<&'static GrammarRule>,
    native_language: NativeLanguage,
    selected_ids: RwSignal<HashSet<Ulid>>,
    search_query: RwSignal<String>,
    known_kanji: HashSet<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let filtered_rules = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            return rules.clone();
        }
        rules
            .iter()
            .filter(|rule| {
                let content = rule.content(&native_language);
                content.title().to_lowercase().contains(&query)
                    || content.short_description().to_lowercase().contains(&query)
            })
            .copied()
            .collect::<Vec<_>>()
    };

    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <div class="space-y-2 overflow-y-auto">
            {move || {
                let filtered = filtered_rules();
                if filtered.is_empty() {
                    view! {
                        <div data-testid="grammar-drawer-empty">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {t!(i18n, grammar_page.no_rules_for_level)}
                            </Text>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || filtered.clone()
                            key=|rule| *rule.rule_id()
                            children=move |rule| {
                                view! {
                                    <RuleItem
                                        rule=rule
                                        test_id=Signal::derive(|| "grammar-drawer-item".to_string())
                                        native_language=native_language
                                        selected_ids=selected_ids
                                        known_kanji=known_kanji_stored.get_value()
                                    />
                                }
                            }
                        />
                    }.into_any()
                }
            }}
        </div>
    }
}
