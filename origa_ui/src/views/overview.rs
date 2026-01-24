use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn Overview() -> impl IntoView {
    let user_info = create_local_resource(
        || (),
        move |_| async move { get_user_info().await },
    );

    view! {
        <MobileLayout>
            <div class="overview-page">
                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || {
                        user_info.get()
                            .map(|result| {
                                match result {
                                    Ok(user) => view! {
                                        <div class="overview-content">
                                            <Card>
                                                <CardBody>
                                                    <h2>"Добро пожаловать в Origa!"</h2>
                                                    <p>"Приложение для изучения японского языка"</p>
                                                </CardBody>
                                            </Card>

                                            <Grid columns="repeat(2, 1fr)" gap="16px" margin-top="16px">
                                                <Card>
                                                    <CardBody>
                                                        <h3>{user.study_streak}</h3>
                                                        <p>"Дней подряд"</p>
                                                    </CardBody>
                                                </Card>
                                                <Card>
                                                    <CardBody>
                                                        <h3>{user.cards_learned}</h3>
                                                        <p>"Карточек изучено"</p>
                                                    </CardBody>
                                                </Card>
                                            </Grid>

                                            <div class="quick-actions" style="margin-top: 24px;">
                                                <h3>"Быстрые действия"</h3>
                                                <Grid columns="repeat(2, 1fr)" gap="16px">
                                                    <Button
                                                        appearance=ButtonAppearance::Primary
                                                        on_click=move |_| {
                                                            leptos_router::use_navigate()("/learn", Default::default());
                                                        }
                                                    >
                                                        "Начать урок"
                                                    </Button>
                                                    <Button
                                                        appearance=ButtonAppearance::Subtle
                                                        on_click=move |_| {
                                                            leptos_router::use_navigate()("/import", Default::default());
                                                        }
                                                    >
                                                        "Импорт"
                                                    </Button>
                                                </Grid>
                                            </div>
                                        </div>
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