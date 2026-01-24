use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn Grammar() -> impl IntoView {
    let search_query = create_rw_signal(String::new());
    let selected_filter = create_rw_signal("all".to_string());
    
    // Grammar cards similar to vocabulary but for grammar rules
    let grammar_cards = create_local_resource(
        move || selected_filter.get(),
        move |filter| async move { 
            // This would call get_grammar_cards in real implementation
            get_vocabulary_cards(filter).await 
        },
    );

    view! {
        <MobileLayout>
            <div class="grammar-page">
                <div class="grammar-header">
                    <h2>"Грамматика"</h2>
                    
                    <div class="grammar-filters" style="margin: 16px 0;">
                        <Input
                            placeholder="Поиск правил..."
                            value=search_query
                            on_change=move |v| search_query.set(v)
                            style="margin-bottom: 12px;"
                        />
                        
                        <Select value=selected_filter style="width: 100%;">
                            <SelectOption value="all">"Все правила"</SelectOption>
                            <SelectOption value="particles">"Частицы"</SelectOption>
                            <SelectOption value="verbs">"Глаголы"</SelectOption>
                            <SelectOption value="adjectives">"Прилагательные"</SelectOption>
                        </Select>
                    </div>
                </div>

                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || {
                        grammar_cards.get()
                            .map(|result| {
                                match result {
                                    Ok(cards) => {
                                        // Mock grammar cards based on vocabulary structure
                                        let grammar_rules = vec![
                                            GrammarRule {
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
                                            }
                                        ];
                                        
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

                                        if filtered_rules.is_empty() {
                                            view! {
                                                <Card style="margin: 16px;">
                                                    <CardBody style="text-align: center;">
                                                        <p>"Правил не найдено"</p>
                                                    </CardBody>
                                                </Card>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <ResponsiveGrid>
                                                    {filtered_rules.iter().map(|rule| {
                                                        view! {
                                                            <GrammarCard rule=rule.clone() />
                                                        }
                                                    }).collect_view()}
                                                </ResponsiveGrid>
                                            }.into_view()
                                        }
                                    }.into_view(),
                                    Err(err) => view! {
                                        <ErrorMessage message=err />
                                    }.into_view(),
                                }
                            })
                    }}
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
    let navigate = leptos_router::use_navigate();
    
    view! {
        <Card class="grammar-card" hoverable=true>
            <CardBody>
                <div class="rule-header">
                    <h3>{rule.title}</h3>
                    <Badge 
                        color=match rule.level.as_str() {
                            "N5" => "success",
                            "N4" => "primary", 
                            "N3" => "warning",
                            _ => "danger"
                        }
                    >
                        {rule.level}
                    </Badge>
                </div>
                <p class="rule-description">{rule.description}</p>
                <div class="rule-footer">
                    <Badge color="default">{rule.category}</Badge>
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
            </CardBody>
        </Card>
    }
}