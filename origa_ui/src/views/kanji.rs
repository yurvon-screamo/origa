use crate::components::*;
use crate::services::*;
use leptos::prelude::*;
use leptos::suspense::Suspense;
use thaw::*;

#[component]
pub fn Kanji() -> impl IntoView {
    let selected_jlpt = RwSignal::new("n5".to_string());
    let search_query = RwSignal::new(String::new());

    // Create LocalResource for kanji list that reacts to JLPT level changes
    let kanji_list = LocalResource::new(move || {
        let jlpt = selected_jlpt.get();
        get_kanji_list(jlpt)
    });

    view! {
        <MobileLayout>
            <div class="kanji-page">
                <div class="kanji-header">
                    <h2>"Кандзи"</h2>

                    <div class="kanji-filters" style="margin: 16px 0;">
                        <Input
                            placeholder="Поиск кандзи..."
                            value=search_query
                        />

                        <Select value=selected_jlpt>
                            <option value="n5">"JLPT N5"</option>
                            <option value="n4">"JLPT N4"</option>
                            <option value="n3">"JLPT N3"</option>
                            <option value="n2">"JLPT N2"</option>
                            <option value="n1">"JLPT N1"</option>
                        </Select>
                    </div>
                </div>

                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || Suspend::new(async move {
                        match kanji_list.await {
                            Ok(kanji_vec) => {
                                let filtered_kanji = kanji_vec
                                    .iter()
                                    .filter(|kanji| {
                                        let query = search_query.get();
                                        if query.is_empty() {
                                            true
                                        } else {
                                            kanji.character.contains(&query) ||
                                            kanji.readings.iter().any(|r| r.contains(&query)) ||
                                            kanji.meanings.iter().any(|m| m.contains(&query))
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                view! {
                                    <div>
                                        {if filtered_kanji.is_empty() {
                                            view! {
                                                <Card>
                                                    <p>"Кандзи не найдено"</p>
                                                </Card>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="cards-grid">
                                                    {filtered_kanji.iter().map(|kanji| {
                                                        view! {
                                                            <KanjiCard kanji=(*kanji).clone() />
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
                                            <p>"Ошибка загрузки кандзи"</p>
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
fn KanjiCard(kanji: KanjiInfo) -> impl IntoView {
    view! {
        <Card class="kanji-card">
            <div style="text-align: center;">
                <div class="kanji-character">
                    <h1 style="font-size: 36px; margin: 0;">{kanji.character}</h1>
                </div>
                <div class="kanji-info">
                    <p style="font-size: 12px; color: var(--thaw-text-color-secondary);">
                        {kanji.strokes} " чертежей"
                    </p>
                    <p style="font-size: 12px; color: var(--thaw-text-color-secondary);">
                        {kanji.readings.first().cloned().unwrap_or_default()}
                    </p>
                </div>
                <div style="margin-top: 12px;">
                    <Badge
                        color=match kanji.jlpt.as_str() {
                            "N5" => BadgeColor::Success,
                            "N4" => BadgeColor::Brand,
                            "N3" => BadgeColor::Warning,
                            "N2" => BadgeColor::Danger,
                            "N1" => BadgeColor::Danger,
                            _ => BadgeColor::Subtle
                        }
                    >
                        {kanji.jlpt}
                    </Badge>
                </div>
                <div style="margin-top: 8px;">
                    {if kanji.added {
                        view! {
                            <Badge color=BadgeColor::Success>"Добавлено"</Badge>
                        }.into_any()
                    } else {
                        view! {
                            <span></span>
                        }.into_any()
                    }}
                </div>
            </div>
        </Card>
    }
}
