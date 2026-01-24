use crate::components::*;
use crate::services::*;
use leptos::*;
use leptos_router::*;
use thaw::*;

#[component]
pub fn Kanji() -> impl IntoView {
    let selected_jlpt = create_rw_signal("n5".to_string());
    let search_query = create_rw_signal(String::new());
    let selected_kanji = create_rw_signal(None::<KanjiInfo>);

    let kanji_list = create_local_resource(
        move || selected_jlpt.get(),
        move |jlpt| async move { get_kanji_list(jlpt).await },
    );

    view! {
        <MobileLayout>
            <div class="kanji-page">
                <div class="kanji-header">
                    <h2>"Кандзи"</h2>

                    <div class="kanji-filters" style="margin: 16px 0;">
                        <Input
                            placeholder="Поиск кандзи..."
                            value=search_query
                            on_change=move |v| search_query.set(v)
                            style="margin-bottom: 12px;"
                        />

                        <Select value=selected_jlpt style="width: 100%;">
                            <SelectOption value="n5">"JLPT N5"</SelectOption>
                            <SelectOption value="n4">"JLPT N4"</SelectOption>
                            <SelectOption value="n3">"JLPT N3"</SelectOption>
                            <SelectOption value="n2">"JLPT N2"</SelectOption>
                            <SelectOption value="n1">"JLPT N1"</SelectOption>
                        </Select>
                    </div>
                </div>

                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || {
                        kanji_list.get()
                            .map(|result| {
                                match result {
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

                                        if filtered_kanji.is_empty() {
                                            view! {
                                                <Card style="margin: 16px;">
                                                    <CardBody style="text-align: center;">
                                                        <p>"Кандзи не найдено"</p>
                                                    </CardBody>
                                                </Card>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <ResponsiveGrid>
                                                    {filtered_kanji.iter().map(|kanji| {
                                                        view! {
                                                            <KanjiCard
                                                                kanji=kanji.clone()
                                                                on_select=move |_| selected_kanji.set(Some(kanji.clone()))
                                                            />
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

                // Kanji Detail Drawer
                <Drawer
                    open=selected_kanji.read().is_some()
                    on_close=move |_| selected_kanji.set(None)
                    position="bottom"
                    title=move || selected_kanji.read().map(|k| k.character).unwrap_or_default()
                >
                    {move || selected_kanji.read().map(|kanji| {
                        view! {
                            <div style="padding: 20px;">
                                <div style="text-align: center; margin-bottom: 20px;">
                                    <h1 style="font-size: 48px; margin: 0;">{kanji.character}</h1>
                                </div>

                                <div style="margin-bottom: 16px;">
                                    <h3>"Чтения:"</h3>
                                    <p>{kanji.readings.join(", ")}</p>
                                </div>

                                <div style="margin-bottom: 16px;">
                                    <h3>"Значения:"</h3>
                                    <p>{kanji.meanings.join(", ")}</p>
                                </div>

                                <div style="margin-bottom: 16px;">
                                    <p><strong>"Чертежи: "</strong>{kanji.strokes}</p>
                                    <p><strong>"JLPT: "</strong>{kanji.jlpt}</p>
                                </div>

                                <div style="display: flex; gap: 8px;">
                                    {if kanji.added {
                                        view! {
                                            <Button appearance=ButtonAppearance::Subtle disabled=true>
                                                "Добавлено"
                                            </Button>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <Button appearance=ButtonAppearance::Primary>
                                                "Добавить в карточки"
                                            </Button>
                                        }.into_view()
                                    }}
                                </div>
                            </div>
                        }
                    })}
                </Drawer>
            </div>
        </MobileLayout>
    }
}

#[component]
fn KanjiCard(kanji: KanjiInfo, on_select: leptos::Callback<()>) -> impl IntoView {
    view! {
        <Card
            class="kanji-card"
            hoverable=true
            on:click=move |_| on_select.run(())
        >
            <CardBody style="text-align: center;">
                <div class="kanji-character">
                    <h1 style="font-size: 36px; margin: 0;">{kanji.character}</h1>
                </div>
                <div class="kanji-info">
                    <p style="font-size: 12px; color: var(--thaw-text-color-secondary);">
                        {kanji.strokes} " чертежей"
                    </p>
                    <p style="font-size: 12px; color: var(--thaw-text-color-secondary);">
                        {kanji.readings.first().unwrap_or(&String::new())}
                    </p>
                </div>
                <div style="margin-top: 12px;">
                    <Badge
                        color=match kanji.jlpt.as_str() {
                            "N5" => "success",
                            "N4" => "primary",
                            "N3" => "warning",
                            "N2" => "danger",
                            "N1" => "danger",
                            _ => "default"
                        }
                    >
                        {kanji.jlpt}
                    </Badge>
                </div>
                {if kanji.added {
                    view! {
                        <div style="margin-top: 8px;">
                            <Badge color="success">"Добавлено"</Badge>
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </CardBody>
        </Card>
    }
}
