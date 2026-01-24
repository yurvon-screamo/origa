use crate::components::*;
use crate::services::*;
use leptos::prelude::*;
use leptos::suspense::Suspense;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn Vocabulary() -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_filter = RwSignal::new("all".to_string());

    // Create LocalResource for vocabulary cards that reacts to filter changes
    let vocabulary_cards = LocalResource::new(move || {
        let filter = selected_filter.get();
        get_vocabulary_cards(filter)
    });

    view! {
        <MobileLayout>
            <div class="vocabulary-page">
                <div class="vocabulary-header">
                    <h2>"Словарь"</h2>

                    <div class="vocabulary-filters" style="margin: 16px 0;">
                        <Input
                            placeholder="Поиск слов..."
                            value=search_query
                        />

                        <Select value=selected_filter>
                            <option value="all">"Все слова"</option>
                            <option value="new">"Новые"</option>
                            <option value="learning">"Изучаемые"</option>
                            <option value="learned">"Изученные"</option>
                        </Select>
                    </div>
                </div>

                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || Suspend::new(async move {
                        match vocabulary_cards.await {
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

                                view! {
                                    <div>
                                        {if filtered_cards.is_empty() {
                                            view! {
                                                <Card>
                                                    <p>"Слов не найдено"</p>
                                                </Card>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="cards-grid">
                                                    {filtered_cards.iter().map(|card| {
                                                        view! {
                                                            <VocabularyCard card=(*card).clone() />
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>
                                }
                                }.into_any(),
                            Err(_err) => view! {
                                <div>
                                    {move || view! {
                                        <Card>
                                            <p>"Ошибка загрузки словаря"</p>
                                        </Card>
                                    }.into_any()}
                                </div>
                                }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </MobileLayout>
    }
}

#[component]
fn VocabularyCard(card: VocabularyCard) -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <Card class="vocabulary-card">
            <div class="word-header">
                <h3>{card.word}</h3>
                <span class="reading">{card.reading}</span>
            </div>
            <p class="meaning">{card.meaning}</p>
            <div class="card-footer">
                <Badge
                    color=match card.difficulty {
                        1 => BadgeColor::Success,
                        2 => BadgeColor::Warning,
                        _ => BadgeColor::Danger
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
                        "new" => BadgeColor::Subtle,
                        "learning" => BadgeColor::Warning,
                        "learned" => BadgeColor::Success,
                        _ => BadgeColor::Subtle
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
        </Card>
    }
}
