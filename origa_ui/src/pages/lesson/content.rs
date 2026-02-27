use super::complete_screen::LessonCompleteScreen;
use super::header::LessonHeader;
use super::lesson_card_container::LessonCardContainer;
use super::lesson_progress_view::LessonProgressView;
use super::lesson_state::{LessonContext, LessonMode, LessonState};
use crate::repository::HybridUserRepository;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use origa::application::use_cases::{SelectCardsToFixationUseCase, SelectCardsToLessonUseCase};
use origa::domain::User;
use ulid::Ulid;

#[component]
pub fn LessonContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let query = use_query_map();
    let mode = match query.read_untracked().get("mode").as_deref() {
        Some("fixation") => LessonMode::Fixation,
        _ => LessonMode::Lesson,
    };

    let lesson_state = RwSignal::new(LessonState::default());
    let is_loading = RwSignal::new(true);
    let is_completed = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);

    let lesson_ctx = LessonContext {
        repository: repository.clone(),
        lesson_state,
        is_completed,
        mode,
    };
    provide_context(lesson_ctx);

    Effect::new(move |_| {
        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repository.clone();
            let current_mode = mode;
            spawn_local(async move {
                is_loading.set(true);

                let cards = match current_mode {
                    LessonMode::Lesson => {
                        let use_case = SelectCardsToLessonUseCase::new(&repo);
                        use_case.execute(user_id).await
                    }
                    LessonMode::Fixation => {
                        let use_case = SelectCardsToFixationUseCase::new(&repo);
                        use_case.execute(user_id).await
                    }
                };

                web_sys::console::log_1(&format!("Cards len: {}", cards.iter().count()).into());

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
                            });
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Ошибка загрузки карточек: {}", e)));
                    }
                }

                is_loading.set(false);
            });
        }
    });

    view! {
        <LessonHeader />

        <Show when=move || is_loading.get()>
            <div class="text-center py-8">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    "Загрузка урока..."
                </Text>
            </div>
        </Show>

        <Show when=move || error_message.get().is_some() && !is_loading.get()>
            <div class="text-center py-8">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {move || error_message.get().unwrap_or_default()}
                </Text>
            </div>
        </Show>

        <Show when=move || is_completed.get()>
            <LessonCompleteScreen
                review_count=lesson_state.get().review_count
            />
        </Show>

        <Show when=move || !is_loading.get() && !is_completed.get() && error_message.get().is_none()>
            <LessonProgressView />
            <LessonCardContainer />
        </Show>
    }
}
