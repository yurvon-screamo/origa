use super::complete_screen::LessonCompleteScreen;
use super::header::LessonHeader;
use super::lesson_card_container::LessonCardContainer;
use super::lesson_progress_view::LessonProgressView;
use super::lesson_state::{LessonContext, LessonMode, LessonState};
use crate::repository::HybridUserRepository;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{Spinner, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use origa::domain::NativeLanguage;
use origa::traits::UserRepository;
use origa::use_cases::{SelectCardsToFixationUseCase, SelectCardsToLessonUseCase};
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn LessonContent() -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

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
    let known_kanji = RwSignal::new(HashSet::<String>::new());
    let native_language = RwSignal::new(NativeLanguage::Russian);

    let is_disposed = StoredValue::new(false);
    on_cleanup(move || is_disposed.set_value(true));
    provide_context(is_disposed);

    let lesson_ctx = LessonContext {
        repository: repository.clone(),
        lesson_state,
        is_completed,
        reload_trigger,
        is_muted,
        known_kanji,
        native_language,
    };
    provide_context(lesson_ctx);

    let repo_for_user_data = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_user_data.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if is_disposed.get_value() {
                    return;
                }
                known_kanji.set(user.knowledge_set().get_known_kanji());
                native_language.set(*user.native_language());
            }
        });
    });

    Effect::new(move |_| {
        if !is_loading.get_untracked() {
            is_syncing_cards.set(true);
        }

        reload_trigger.set(reload_trigger.get_untracked() + 1);
    });

    Effect::new(move |_| {
        reload_trigger.get();

        if auth_store.is_checking_session.get() || !auth_store.is_data_loaded.get() {
            return;
        }

        let repo = repository.clone();
        let current_mode = mode;
        spawn_local(async move {
            if is_disposed.get_value() {
                return;
            }
            is_loading.set(true);

            let cards: Result<
                std::collections::HashMap<ulid::Ulid, origa::domain::LessonCardView>,
                _,
            > = match current_mode {
                LessonMode::Lesson => {
                    let use_case = SelectCardsToLessonUseCase::new(&repo);
                    use_case.execute().await
                }
                LessonMode::Fixation => {
                    let use_case = SelectCardsToFixationUseCase::new(&repo);
                    use_case.execute().await
                }
            };

            tracing::info!("Cards len: {}", cards.iter().count());

            if is_disposed.get_value() {
                return;
            }

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
                            selected_yesno_answer: None,
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
    });

    view! {
        <LessonHeader />

        <Show when=move || is_loading.get()>
            <div class="flex flex-col items-center py-8 gap-4">
                <Spinner />
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    "Подготовка карточек для урока..."
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
                is_completed
                review_count=lesson_state.get().review_count
            />
        </Show>

        <Show when=move || !is_loading.get() && !is_completed.get() && error_message.get().is_none()>
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
