use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn Vocabulary() -> impl IntoView {
    let search_query = create_rw_signal(String::new());
    let selected_filter = create_rw_signal("all".to_string());
    
    let vocabulary_cards = create_local_resource(
        move || selected_filter.get(),
        move |filter| async move { get_vocabulary_cards(filter).await },
    );

    view! {
        <MobileLayout>
            <div class="vocabulary-page">
                <div class="vocabulary-header">
                    <h2>"Словарь"</h2>
                    
                    <div class="vocabulary-filters" style="margin: 16px 0;">
                        <Input
                            placeholder="Поиск слов..."
                            value=search_query
                            on_change=move |v| search_query.set(v)
                            style="margin-bottom: 12px;"
                        />
                        
                        <Select value=selected_filter style="width: 100%;">
                            <SelectOption value="all">"Все слова"</SelectOption>
                            <SelectOption value="new">"Новые"</SelectOption>
                            <SelectOption value="learning">"Изучаемые"</SelectOption>
                            <SelectOption value="learned">"Изученные"</SelectOption>
                        </Select>
                    </div>
                </div>

                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || {
                        vocabulary_cards.get()
                            .map(|result| {
                                match result {
                                    Ok(cards) => {
                                        let filtered_cards = cards
                                            .iter()
                                            .filter(|card| {
                                                let query = search_query.get();
                                                if query.is_empty() {
                                                    true
                                                } else {
                                                    card.word.to_lowercase().contains(&query.to_lowercase()) ||
                                                    card.reading.to_lowercase().contains(&query.to_lowercase()) ||
                                                    card.meaning.to_lowercase().contains(&query.to_lowercase())
                                                }
                                            })
                                            .collect::<Vec<_>>();

                                        if filtered_cards.is_empty() {
                                            view! {
                                                <Card style="margin: 16px;">
                                                    <CardBody style="text-align: center;">
                                                        <p>"Слов не найдено"</p>
                                                    </CardBody>
                                                </Card>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <ResponsiveGrid>
                                                    {filtered_cards.iter().map(|card| {
                                                        view! {
                                                            <VocabularyCard card=card.clone() />
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

#[component]
fn VocabularyCard(card: VocabularyCard) -> impl IntoView {
    let navigate = leptos_router::use_navigate();
    
    view! {
        <Card class="vocabulary-card" hoverable=true>
            <CardBody>
                <div class="word-header">
                    <h3>{card.word}</h3>
                    <span class="reading">{card.reading}</span>
                </div>
                <p class="meaning">{card.meaning}</p>
                <div class="card-footer">
                    <Badge 
                        color=match card.difficulty {
                            1 => "success",
                            2 => "warning", 
                            _ => "danger"
                        }
                    >
                        {match card.difficulty {
                            1 => "N5",
                            2 => "N4", 
                            _ => "N3+"
                        }}
                    </Badge>
                    <Badge 
                        color=match card.status.as_str() {
                            "new" => "default",
                            "learning" => "warning",
                            "learned" => "success",
                            _ => "default"
                        }
                    >
                        {match card.status.as_str() {
                            "new" => "Новое",
                            "learning" => "Изучается",
                            "learned" => "Изучено",
                            _ => "Неизвестно"
                        }}
                    </Badge>
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