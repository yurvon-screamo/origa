use crate::components::*;
use crate::services::*;
use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos::suspense::Suspense;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn Learn() -> impl IntoView {
    let _navigate = use_navigate();
    let current_card_index = RwSignal::new(0);
    let show_answer = RwSignal::new(false);
    let loading = RwSignal::new(false);

    // Create LocalResource for lesson cards
    let lesson_cards = LocalResource::new(move || select_cards_to_lesson(10));

    // Create a signal to store the cards for rating handling
    let cards_for_rating = RwSignal::new(Vec::<LessonCard>::new());

    let handle_rating = move |rating: i32| {
        let cards = cards_for_rating.get();
        let current_idx = current_card_index.get();
        if current_idx < cards.len() {
            let card = &cards[current_idx];
            let card_id = card.id.clone();

            loading.set(true);
            spawn_local(async move {
                if let Err(err) = rate_card(card_id, rating).await {
                    leptos::logging::log!("Error rating card: {}", err);
                }
                loading.set(false);
                current_card_index.update(|n| *n += 1);
                show_answer.set(false);
            });
        }
    };

    let handle_keypress = move |event: leptos::web_sys::KeyboardEvent| {
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
                let cards = cards_for_rating.get();
                if current_card_index.get() < cards.len() - 1 {
                    current_card_index.update(|n| *n += 1);
                    show_answer.set(false);
                }
            }
            _ => {}
        }
    };

    view! {
        <MobileLayout>
            <div class="learn-page" on:keydown=handle_keypress tabindex="0">
                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || Suspend::new(async move {
                        match lesson_cards.await {
                            Ok(cards) => {
                                // Update the cards signal for rating handler
                                cards_for_rating.set(cards.clone());

                                if cards.is_empty() {
                                    view! {
                                        <Card class="lesson-card">
                                            <div class="empty-state">
                                                <h3>"Карточек для изучения нет"</h3>
                                                <p>"Импортируйте новые карточки или вернитесь позже"</p>
                                                <Button
                                                    appearance=ButtonAppearance::Primary
                                                    on_click=move |_| {
                                                        // Will be handled by parent navigate
                                                    }
                                                >
                                                    "Импортировать карточки"
                                                </Button>
                                            </div>
                                        </Card>
                                    }.into_view()
                                } else {
                                    let current_idx = current_card_index.get();
                                    let total_cards = cards.len();
                                    if current_idx >= total_cards {
                                        view! {
                                            <Card class="lesson-card">
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
                                            </Card>
                                        }.into_view()
                                    } else {
                                        let card = cards[current_idx].clone();
                                        view! {
                                            <Card class="lesson-card">
                                                <div class="question">
                                                    <h2>{card.question.clone()}</h2>
                                                    {card.furigana.as_ref().map(|f| {
                                                        view! {
                                                            <div class="furigana">{f.clone()}</div>
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
                                                    <p>"Карточка " {current_idx + 1} " из " {total_cards}</p>
                                                    <ProgressBar value={
                                                        let progress = (current_idx + 1) as f64 / total_cards as f64;
                                                        RwSignal::new(progress)
                                                    } />
                                                </div>
                                            </Card>
                                        }.into_view()
                                    }
                                }
                            },
                            Err(err) => view! {
                                <Card class="lesson-card">
                                    <div class="empty-state">
                                        <h3>"Ошибка загрузки"</h3>
                                        <p>{err}</p>
                                    </div>
                                </Card>
                            }.into_view(),
                        }
                    })}
                </Suspense>
            </div>
        </MobileLayout>
    }
}
