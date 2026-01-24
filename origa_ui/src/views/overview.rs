use crate::components::*;
use crate::services::*;
use leptos::prelude::*;
use leptos::suspense::Suspense;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn Overview() -> impl IntoView {
    let navigate = use_navigate();

    // Create LocalResource for user info
    let user_info = LocalResource::new(get_user_info);

    view! {
        <MobileLayout>
            <div class="overview-page">
                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || Suspend::new({
                        let navigate = navigate.clone();
                        async move {
                            match user_info.await {
                                Ok(user) => view! {
                                    <div class="overview-content">
                                        <Card>
                                            <CardHeader>
                                                <h2>"Добро пожаловать, " {user.name.clone()} "!"</h2>
                                                <p>"Приложение для изучения японского языка"</p>
                                            </CardHeader>
                                        </Card>

                                        <Card>
                                            <CardHeader>
                                                <h3>"Ваша статистика"</h3>
                                            </CardHeader>
                                            <div class="user-stats-grid">
                                                <div class="stat-item">
                                                    <div class="stat-value">{user.study_streak}</div>
                                                    <div class="stat-label">"Дней изучения подряд"</div>
                                                </div>
                                                <div class="stat-item">
                                                    <div class="stat-value">{user.cards_learned}</div>
                                                    <div class="stat-label">"Изученных карточек"</div>
                                                </div>
                                            </div>
                                        </Card>

                                        <div class="quick-actions" style="margin-top: 24px;">
                                            <h3>"Быстрые действия"</h3>
                                            <div class="overview-actions-grid">
                                                <Button
                                                    appearance=ButtonAppearance::Primary
                                                    on_click={
                                                        let navigate = navigate.clone();
                                                        move |_| {
                                                            navigate("/learn", Default::default());
                                                        }
                                                    }
                                                >
                                                    "Начать урок"
                                                </Button>
                                                <Button
                                                    appearance=ButtonAppearance::Subtle
                                                    on_click={
                                                        let navigate = navigate.clone();
                                                        move |_| {
                                                            navigate("/import", Default::default());
                                                        }
                                                    }
                                                >
                                                    "Импорт"
                                                </Button>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view(),
                                Err(err) => view! {
                                    <div class="overview-content">
                                        <Card>
                                            <CardHeader>
                                                <h2>"Ошибка"</h2>
                                                <p>{err}</p>
                                            </CardHeader>
                                        </Card>

                                        <Card>
                                            <CardHeader>
                                                <h3>"Ваша статистика"</h3>
                                            </CardHeader>
                                            <div class="user-stats-grid">
                                                <div class="stat-item">
                                                    <div class="stat-value">"--"</div>
                                                    <div class="stat-label">"Дней изучения подряд"</div>
                                                </div>
                                                <div class="stat-item">
                                                    <div class="stat-value">"--"</div>
                                                    <div class="stat-label">"Изученных карточек"</div>
                                                </div>
                                            </div>
                                        </Card>

                                        <div class="quick-actions" style="margin-top: 24px;">
                                            <h3>"Быстрые действия"</h3>
                                            <div class="overview-actions-grid">
                                                <Button
                                                    appearance=ButtonAppearance::Primary
                                                    disabled=true
                                                >
                                                    "Начать урок"
                                                </Button>
                                                <Button
                                                    appearance=ButtonAppearance::Subtle
                                                    on_click={
                                                        let navigate = navigate.clone();
                                                        move |_| {
                                                            navigate("/import", Default::default());
                                                        }
                                                    }
                                                >
                                                    "Импорт"
                                                </Button>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view(),
                            }
                        }
                    })}
                </Suspense>
            </div>
        </MobileLayout>
    }
}
