use super::complete_screen::LessonCompleteScreen;
use super::header::LessonHeader;
use super::lesson_card_container::LessonCardContainer;
use super::lesson_progress_view::LessonProgressView;
use super::lesson_state::{LessonContext, LessonMode, LessonState};
use crate::app::AuthContext;
use crate::repository::session;
use crate::repository::{HybridUserRepository, SyncContext};
use crate::ui_components::{Spinner, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use origa::domain::User;
use origa::use_cases::{SelectCardsToFixationUseCase, SelectCardsToLessonUseCase};
use ulid::Ulid;

#[component]
pub fn LessonContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let sync_context =
        use_context::<SyncContext>().expect("sync_context not provided");

    let query = use_query_map();
    let mode = match query.read_untracked().get("mode").as_deref() {
        Some("fixation") => LessonMode::Fixation,
        _ => LessonMode::Lesson,
    };

    let lesson_state = RwSignal::new(LessonState::default());
    let is_loading = RwSignal::new(true);
    let is_completed = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let reload_trigger = RwSignal::new(0u32);
    let is_muted = RwSignal::new(false);
    let is_syncing_cards = RwSignal::new(false);
    let is_initial_syncing = RwSignal::new(true);

    let lesson_ctx = LessonContext {
        repository: repository.clone(),
        lesson_state,
        is_completed,
        mode,
        reload_trigger,
        is_muted,
    };
    provide_context(lesson_ctx);

    let repo_for_sync = repository.clone();
    let sync_ctx_for_sync = sync_context;
    let user_for_sync = current_user;
    Effect::new(move |_| {
        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repo_for_sync.clone();
            let sync_ctx = sync_ctx_for_sync;
            let current_user_signal = user_for_sync;
            spawn_local(async move {
                sync_ctx.start_sync();

                match repo.force_sync(user_id).await {
                    Ok(Some(merged_user)) => {
                        current_user_signal.set(Some(merged_user));
                        tracing::info!("Lesson: force_sync completed");
                    }
                    Ok(None) => {
                        tracing::debug!("Lesson: force_sync - no changes");
                    }
                    Err(e) => {
                        tracing::error!("Lesson: force_sync error: {:?}", e);
                    }
                }

                session::set_last_sync_time(
                    js_sys::Date::now() as u64 / 1000,
                );
                sync_ctx.complete_sync();
                is_initial_syncing.set(false);
            });
        }
    });

    Effect::new(move |_| {
        sync_context.sync_trigger.get();

        if !is_loading.get_untracked() {
            is_syncing_cards.set(true);
        }

        reload_trigger.set(reload_trigger.get_untracked() + 1);
    });

    Effect::new(move |_| {
        reload_trigger.get();

        if auth_ctx.is_session_loading.get() {
            return;
        }

        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repository.clone();
            let current_mode = mode;
            spawn_local(async move {
                is_loading.set(true);

                let cards: Result<
                    std::collections::HashMap<ulid::Ulid, origa::domain::LessonCardView>,
                    _,
                > = match current_mode {
                    LessonMode::Lesson => {
                        let use_case = SelectCardsToLessonUseCase::new(&repo);
                        use_case.execute(user_id).await
                    }
                    LessonMode::Fixation => {
                        let use_case = SelectCardsToFixationUseCase::new(&repo);
                        use_case.execute(user_id).await
                    }
                };

                tracing::info!("Cards len: {}", cards.iter().count());

                match cards {
                    Ok(cards) => {
                        let card_ids: Vec<Ulid> = cards.keys().cloned().collect();
                        if cards.is_empty() {
                            error_message.set(Some("Нет карточек для изучения".to_string()));
                        } else {
                            lesson_state.set(LessonState {
                                cards,
                                card_ids,
                                current_index: 0,
                                showing_answer: false,
                                review_count: 0,
                                selected_quiz_option: None,
                            });
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Ошибка загрузки карточек: {}", e)));
                    }
                }

                is_loading.set(false);
                is_syncing_cards.set(false);
            });
        } else {
            error_message.set(Some("Пользователь не найден".to_string()));
            is_loading.set(false);
            is_syncing_cards.set(false);
        }
    });

    view! {
        <LessonHeader />

        <Show when=move || is_initial_syncing.get()>
            <div class="flex flex-col items-center py-8 gap-4">
                <Spinner />
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    "Синхронизация..."
                </Text>
            </div>
        </Show>

        <Show when=move || !is_initial_syncing.get() && is_loading.get()>
            <div class="flex flex-col items-center py-8 gap-4">
                <Spinner />
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    "Подготовка карточек для урока..."
                </Text>
            </div>
        </Show>

        <Show when=move || error_message.get().is_some() && !is_loading.get() && !is_initial_syncing.get()>
            <div class="text-center py-8">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {move || error_message.get().unwrap_or_default()}
                </Text>
            </div>
        </Show>

        <Show when=move || is_completed.get()>
            <LessonCompleteScreen
                is_completed
                review_count=lesson_state.get().review_count
            />
        </Show>

        <Show when=move || !is_initial_syncing.get() && !is_loading.get() && !is_completed.get() && error_message.get().is_none()>
            <div class="relative">
                <Show when=move || is_syncing_cards.get()>
                    <div class="absolute top-0 right-0 flex items-center gap-1 text-sm text-muted-foreground p-2">
                        <Spinner class=Signal::derive(|| "".to_string()) size=Signal::derive(|| "sm".to_string()) />
                        "Синхронизация..."
                    </div>
                </Show>

                <LessonProgressView />
                <LessonCardContainer />
            </div>
        </Show>
    }
}
