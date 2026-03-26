mod onboarding_state;

mod apps_step;
mod intro_step;
mod jlpt_step;
mod progress;
mod summary_step;

use crate::loaders::WellKnownSetLoaderImpl;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant, Spinner,
    Stepper, StepperStep, Text, TextSize, TypographyVariant,
};
use apps_step::AppsStep;
use intro_step::IntroStep;
use jlpt_step::JlptStep;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use onboarding_state::{OnboardingState, OnboardingStep};
use origa::domain::User;
use origa::traits::{UserRepository, WellKnownSetLoader};
use origa::use_cases::ImportOnboardingSetsUseCase;
use progress::ProgressStep;
use summary_step::SummaryStep;

#[component]
pub fn Onboarding() -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let navigate = use_navigate();
    let navigate_for_import = navigate.clone();

    let state = RwSignal::new(OnboardingState::new());
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let sets_loaded = RwSignal::new(false);
    let is_loading = RwSignal::new(true);
    let is_importing = RwSignal::new(false);

    provide_context(state);

    let steps: Signal<Vec<StepperStep>> = Signal::derive(move || {
        vec![
            StepperStep {
                number: 1,
                label: "Приветствие".to_string(),
            },
            StepperStep {
                number: 2,
                label: "Уровень".to_string(),
            },
            StepperStep {
                number: 3,
                label: "Приложения".to_string(),
            },
            StepperStep {
                number: 4,
                label: "Прогресс".to_string(),
            },
            StepperStep {
                number: 5,
                label: "Импорт".to_string(),
            },
        ]
    });

    let active_step: RwSignal<usize> = RwSignal::new(0);

    Effect::new(move |_| {
        let step = state.get().current_step.as_usize();
        active_step.set(step);
    });

    let repo_for_init = repository.clone();
    let loader = WellKnownSetLoaderImpl::new();
    let initialized = RwSignal::new(false);

    Effect::new(move |_| {
        if initialized.get() {
            return;
        }
        initialized.set(true);

        let repo = repo_for_init.clone();
        let loader = loader.clone();
        let nav = navigate.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    current_user.set(Some(user.clone()));
                    if !user.imported_sets().is_empty() {
                        nav("/home", Default::default());
                        return;
                    }
                },
                Ok(None) => {
                    tracing::warn!("Onboarding: user not found");
                    nav("/login", Default::default());
                    return;
                },
                Err(e) => {
                    tracing::error!("Onboarding: get_current_user error: {:?}", e);
                    nav("/login", Default::default());
                    return;
                },
            };

            match loader.load_meta_list().await {
                Ok(meta_list) => {
                    state.update(|s| {
                        s.set_available_sets(meta_list);
                        sets_loaded.set(true);
                    });
                },
                Err(e) => {
                    tracing::error!("Onboarding: load_meta_list error: {:?}", e);
                },
            }
            is_loading.set(false);
        });
    });

    let on_next = Callback::new(move |_: ()| {
        state.update(|s| {
            s.go_to_next_step();
        });
    });

    let on_prev = Callback::new(move |_: ()| {
        state.update(|s| {
            s.go_to_prev_step();
        });
    });

    let on_start_import = {
        let nav = navigate_for_import.clone();
        Callback::new(move |_: ()| {
            let repo = repository.clone();
            let loader = WellKnownSetLoaderImpl::new();
            let nav = nav.clone();
            is_importing.set(true);

            spawn_local(async move {
                let set_ids = state.get().get_final_sets();

                if set_ids.is_empty() {
                    tracing::warn!("No sets selected for import");
                    is_importing.set(false);
                    return;
                }

                let Some(user) = current_user.get() else {
                    tracing::error!("User not loaded");
                    is_importing.set(false);
                    return;
                };

                let use_case = ImportOnboardingSetsUseCase::new(&repo, &loader);
                let result = use_case.execute(user.id(), set_ids).await;

                match result {
                    Ok(import_result) => {
                        tracing::info!(
                            "Imported: {} vocabulary, {} kanji, {} duplicates skipped",
                            import_result.created_vocabulary,
                            import_result.created_kanji,
                            import_result.skipped_duplicates
                        );
                        is_importing.set(false);
                        nav("/home", Default::default());
                    },
                    Err(e) => {
                        tracing::error!("Import failed: {:?}", e);
                        is_importing.set(false);
                    },
                }
            });
        })
    };

    let can_proceed = Memo::new(move |_| state.get().can_proceed());

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="onboarding-page">
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8" test_id="onboarding-card">
                <Show when=move || is_loading.get()>
                    <div class="flex flex-col items-center py-8 gap-4">
                        <Spinner test_id="onboarding-spinner" />
                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "onboarding-loading-text".to_string())>
                            "Загрузка..."
                        </Text>
                    </div>
                </Show>

                <Show when=move || !is_loading.get()>
                    <div class="onboarding-container">
                        <Stepper steps=steps active=active_step test_id="onboarding-stepper" />

                        <div class="onboarding-content mt-8">
                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Intro)>
                                <IntroStep test_id=Signal::derive(|| "onboarding-intro-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Jlpt)>
                                <JlptStep test_id=Signal::derive(|| "onboarding-jlpt-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Apps)>
                                <AppsStep test_id=Signal::derive(|| "onboarding-apps-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Progress)>
                                <ProgressStep test_id=Signal::derive(|| "onboarding-progress-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Summary)>
                                <SummaryStep test_id=Signal::derive(|| "onboarding-summary-step".to_string()) />
                            </Show>
                        </div>

                        <div class="onboarding-actions mt-8 flex justify-between">
                            <div>
                                <Show when=move || state.get().is_first_step()>
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            state.update(|s| {
                                                s.current_step = OnboardingStep::Summary;
                                            });
                                        })
                                        test_id="onboarding-skip"
                                    >
                                        "Пропустить"
                                    </Button>
                                </Show>

                                <Show when=move || !state.get().is_first_step() && !state.get().is_last_step()>
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_prev.run(());
                                        })
                                        test_id="onboarding-prev"
                                    >
                                        "Назад"
                                    </Button>
                                </Show>
                            </div>

                            <div>
                                <Show when=move || !state.get().is_last_step()>
                                    <Button
                                        variant=ButtonVariant::Olive
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_next.run(());
                                        })
                                        disabled=Signal::derive(move || !can_proceed.get())
                                        test_id="onboarding-next"
                                    >
                                        "Далее"
                                    </Button>
                                </Show>

                                <Show when=move || state.get().is_last_step()>
                                    <Button
                                        variant=ButtonVariant::Olive
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_start_import.run(());
                                        })
                                        disabled=Signal::derive(move || is_importing.get() || !can_proceed.get())
                                        test_id="onboarding-import"
                                    >
                                        {move || if is_importing.get() { "Импорт..." } else { "Начать импорт" }}
                                    </Button>
                                </Show>
                            </div>
                        </div>
                    </div>
                </Show>
            </CardLayout>
        </PageLayout>
    }
}
