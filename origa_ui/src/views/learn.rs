use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn Learn() -> impl IntoView {
    let current_card_index = create_rw_signal(0);
    let show_answer = create_rw_signal(false);
    let loading = create_rw_signal(false);
    
    let lesson_cards = create_local_resource(
        || (),
        move |_| async move { select_cards_to_lesson(10).await },
    );

    let handle_rating = move |rating: i32| {
        let cards = lesson_cards.get();
        if let Some(Ok(cards)) = cards {
            let current_idx = current_card_index.get();
            if current_idx < cards.len() {
                let card = &cards[current_idx];
                let card_id = card.id.clone();
                
                loading.set(true);
                spawn_local(async move {
                    if let Err(err) = rate_card(card_id, rating).await {
                        logging::log!("Error rating card: {}", err);
                    }
                    loading.set(false);
                    current_card_index.update(|n| *n += 1);
                    show_answer.set(false);
                });
            }
        }
    };

    let handle_keypress = move |event: web_sys::KeyboardEvent| {
        match &event.key()[..] {
            " " | "Space" => {
                event.prevent_default();
                show_answer.update(|v| *v = !*v);
            }
            "1" => handle_rating(1), // Again
            "2" => handle_rating(2), // Hard
            "3" => handle_rating(3), // Good
            "4" => handle_rating(4), // Easy
            "ArrowLeft" => {
                if current_card_index.get() > 0 {
                    current_card_index.update(|n| *n -= 1);
                    show_answer.set(false);
                }
            }
            "ArrowRight" => {
                let cards = lesson_cards.get();
                if let Some(Ok(cards)) = cards {
                    if current_card_index.get() < cards.len() - 1 {
                        current_card_index.update(|n| *n += 1);
                        show_answer.set(false);
                    }
                }
            }
            _ => {}
        }
    };

    view! {
        <MobileLayout>
            <div class="learn-page" on:keydown=handlekeypress tabindex="0">
                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || {
                        lesson_cards.get()
                            .map(|result| {
                                match result {
                                    Ok(cards) => {
                                        if cards.is_empty() {
                                            view! {
                                                <Card class="lesson-card">
                                                    <CardBody>
                                                        <div class="empty-state">
                                                            <h3>"Карточек для изучения нет"</h3>
                                                            <p>"Импортируйте новые карточки или вернитесь позже"</p>
                                                            <Button
                                                                appearance=ButtonAppearance::Primary
                                                                on_click=move |_| {
                                                                    leptos_router::use_navigate()("/import", Default::default());
                                                                }
                                                            >
                                                                "Импортировать карточки"
                                                            </Button>
                                                        </div>
                                                    </CardBody>
                                                </Card>
                                            }.into_view()
                                        } else {
                                            let current_idx = current_card_index.get();
                                            if current_idx >= cards.len() {
                                                view! {
                                                    <Card class="lesson-card">
                                                        <CardBody>
                                                            <div class="empty-state">
                                                                <h3>"Урок завершен!"</h3>
                                                                <p>"Отличная работа!"</p>
                                                                <Button
                                                                    appearance=ButtonAppearance::Primary
                                                                    on_click=move |_| {
                                                                        current_card_index.set(0);
                                                                    }
                                                                >
                                                                    "Начать заново"
                                                                </Button>
                                                            </div>
                                                        </CardBody>
                                                    </Card>
                                                }.into_view()
                                            } else {
                                                let card = &cards[current_idx];
                                                view! {
                                                    <Card class="lesson-card">
                                                        <CardBody>
                                                            <div class="card-content">
                                                                <div class="question">
                                                                    <h2>{card.question.clone()}</h2>
                                                                    {card.furigana.as_ref().map(|f| {
                                                                        view! {
                                                                            <div class="furigana">{f}</div>
                                                                        }
                                                                    })}
                                                                </div>
                                                                
                                                                <div class="actions">
                                                                    <Button
                                                                        appearance=ButtonAppearance::Primary
                                                                        on_click=move |_| show_answer.update(|v| *v = !*v)
                                                                    >
                                                                        {if show_answer.get() { "Скрыть" } else { "Показать" }}
                                                                    </Button>
                                                                </div>

                                                                <Show when=move || show_answer.get()>
                                                                    <div class="answer">
                                                                        <h3>{card.answer.clone()}</h3>
                                                                        {card.example.as_ref().map(|example| {
                                                                            view! {
                                                                                <div class="example">
                                                                                    <p><strong>"Пример: "</strong>{example.text.clone()}</p>
                                                                                    <p class="translation">{example.translation.clone()}</p>
                                                                                </div>
                                                                            }
                                                                        })}
                                                                    </div>
                                                                </Show>

                                                                <Show when=move || show_answer.get()>
                                                                    <div class="rating-buttons">
                                                                        <Button
                                                                            appearance=ButtonAppearance::Subtle
                                                                            on_click=move |_| handle_rating(1)
                                                                            disabled=loading.get()
                                                                        >
                                                                            "Снова"
                                                                        </Button>
                                                                        <Button
                                                                            appearance=ButtonAppearance::Subtle
                                                                            on_click=move |_| handle_rating(2)
                                                                            disabled=loading.get()
                                                                        >
                                                                            "Трудно"
                                                                        </Button>
                                                                        <Button
                                                                            appearance=ButtonAppearance::Subtle
                                                                            on_click=move |_| handle_rating(3)
                                                                            disabled=loading.get()
                                                                        >
                                                                            "Хорошо"
                                                                        </Button>
                                                                        <Button
                                                                            appearance=ButtonAppearance::Subtle
                                                                            on_click=move |_| handle_rating(4)
                                                                            disabled=loading.get()
                                                                        >
                                                                            "Легко"
                                                                        </Button>
                                                                    </div>
                                                                </Show>

                                                                <div class="card-progress">
                                                                    <p>"Карточка " {current_idx + 1} " из " {cards.len()}</p>
                                                                    <Progress max={cards.len()} value={current_idx + 1} />
                                                                </div>
                                                            </div>
                                                        </CardBody>
                                                    </Card>
                                                }.into_view()
                                            }
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