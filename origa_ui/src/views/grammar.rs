use crate::components::*;
use crate::services::*;
use leptos::prelude::*;
use leptos::suspense::Suspense;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn Grammar() -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_filter = RwSignal::new("all".to_string());

    // Grammar cards similar to vocabulary but for grammar rules
    let grammar_cards = LocalResource::new(move || {
        let filter = selected_filter.get();
        // This would call get_grammar_cards in real implementation
        get_vocabulary_cards(filter)
    });

    view! {
        <MobileLayout>
            <div class="grammar-page">
                <div class="grammar-header">
                    <h2>"Грамматика"</h2>

                    <div class="grammar-filters" style="margin: 16px 0;">
                        <Input
                            placeholder="Поиск правил..."
                            value=search_query
                        />

                        <Select value=selected_filter>
                            <option value="all">"Все правила"</option>
                            <option value="particles">"Частицы"</option>
                            <option value="verbs">"Глаголы"</option>
                            <option value="adjectives">"Прилагательные"</option>
                        </Select>
                    </div>
                </div>

                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || Suspend::new(async move {
                        match grammar_cards.await {
                                Ok(_cards) => {
                                // Mock grammar cards based on vocabulary structure
                                let grammar_rules = [GrammarRule {
                                        id: "grammar1".to_string(),
                                        title: "Частица は (wa)".to_string(),
                                        description: "Указывает на тему предложения".to_string(),
                                        level: "N5".to_string(),
                                        category: "particles".to_string(),
                                    },
                                    GrammarRule {
                                        id: "grammar2".to_string(),
                                        title: "Частица が (ga)".to_string(),
                                        description: "Указывает на подлежащее".to_string(),
                                        level: "N5".to_string(),
                                        category: "particles".to_string(),
                                    },
                                    GrammarRule {
                                        id: "grammar3".to_string(),
                                        title: "Вежливая форма глаголов (ます形)".to_string(),
                                        description: "Формальная форма глаголов".to_string(),
                                        level: "N5".to_string(),
                                        category: "verbs".to_string(),
                                    },
                                    GrammarRule {
                                        id: "grammar4".to_string(),
                                        title: "И-прилагательные".to_string(),
                                        description: "Прилагательные, оканчивающиеся на い".to_string(),
                                        level: "N5".to_string(),
                                        category: "adjectives".to_string(),
                                    }];

                                let filtered_rules = grammar_rules
                                    .iter()
                                    .filter(|rule| {
                                        let query = search_query.get();
                                        if query.is_empty() {
                                            true
                                        } else {
                                            rule.title.to_lowercase().contains(&query.to_lowercase()) ||
                                            rule.description.to_lowercase().contains(&query.to_lowercase())
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                view! {
                                    <div>
                                        {if filtered_rules.is_empty() {
                                            view! {
                                                <Card>
                                                    <p>"Правил не найдено"</p>
                                                </Card>
                                            }.into_any()
                                        } else {
                                            view! {
                                                "Временно недоступно"
                                            }.into_any()
                                        }}
                                    </div>
                                }
                            },
                            Err(_err) => view! {
                                <div>
                                    {view! {
                                        <Card>
                                            <p>"Ошибка загрузки грамматики"</p>
                                        </Card>
                                    }.into_any()}
                                </div>
                            },
                        }
                    })}
                </Suspense>
            </div>
        </MobileLayout>
    }
}

#[derive(Clone)]
struct GrammarRule {
    id: String,
    title: String,
    description: String,
    level: String,
    category: String,
}

#[component]
fn GrammarCard(rule: GrammarRule) -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <Card class="grammar-card">
            <div class="rule-header">
                <h3>{rule.title}</h3>
                <Badge
                    color=match rule.level.as_str() {
                        "N5" => BadgeColor::Success,
                        "N4" => BadgeColor::Brand,
                        "N3" => BadgeColor::Warning,
                        _ => BadgeColor::Danger
                    }
                >
                    {rule.level}
                </Badge>
            </div>
            <p class="rule-description">{rule.description}</p>
            <div class="rule-footer">
                <Badge color=BadgeColor::Subtle>{rule.category}</Badge>
            </div>
            <div style="margin-top: 12px;">
                <Button
                    appearance=ButtonAppearance::Primary
                    size=ButtonSize::Small
                    on_click=move |_| navigate("/learn", Default::default())
                >
                    "Учить"
                </Button>
            </div>
        </Card>
    }
}
