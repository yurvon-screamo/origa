use super::complete_screen::LessonCompleteScreen;
use super::header::LessonHeader;
use super::lesson_card_container::LessonCardContainer;
use super::lesson_progress_view::LessonProgressView;
use super::lesson_state::{LessonContext, LessonState};
use crate::i18n::*;
use crate::repository::HybridUserRepository;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{Spinner, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::traits::UserRepository;
use origa::use_cases::SelectCardsToLessonUseCase;
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn LessonContent() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    let lesson_state = RwSignal::new(LessonState::default());
    let is_loading = RwSignal::new(true);
    let is_completed = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let reload_trigger = RwSignal::new(0u32);
    let is_muted = RwSignal::new(false);
    let is_syncing_cards = RwSignal::new(false);
    let known_kanji = RwSignal::new(HashSet::<String>::new());
    let native_language = RwSignal::new(crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let is_disposed = StoredValue::new(());
    provide_context(is_disposed);

    Effect::new(move |_| {
        native_language.set(crate::i18n::locale_to_native_language(&i18n.get_locale()));
    });

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
                if is_disposed.is_disposed() {
                    return;
                }
                known_kanji.set(user.knowledge_set().get_known_kanji());
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

        if !auth_store.is_dictionary_loaded.get() {
            return;
        }

        let repo = repository.clone();
        spawn_local(async move {
            if is_disposed.is_disposed() {
                return;
            }
            is_loading.set(true);

            let use_case = SelectCardsToLessonUseCase::new(&repo);
            let cards = use_case.execute().await;

            tracing::info!("Cards len: {}", cards.iter().count());

            if is_disposed.is_disposed() {
                return;
            }

            match cards {
                Ok(cards) => {
                    let card_ids: Vec<Ulid> = cards.keys().cloned().collect();
                    if cards.is_empty() {
                        error_message.set(Some(
                            i18n.get_keys().lesson().no_cards().inner().to_string(),
                        ));
                    } else {
                        lesson_state.set(LessonState {
                            cards,
                            card_ids,
                            current_index: 0,
                            showing_answer: false,
                            review_count: 0,
                            selected_quiz_option: None,
                            selected_yesno_answer: None,
                            dont_know_selected: false,
                        });
                    }
                },
                Err(e) => {
                    error_message.set(Some(
                        i18n.get_keys()
                            .lesson()
                            .load_error()
                            .inner()
                            .replace("{}", &e.to_string()),
                    ));
                },
            }

            is_loading.set(false);
            is_syncing_cards.set(false);
        });
    });

    view! {
        <LessonHeader />

        <Show when=move || is_loading.get()>
            <div data-testid="lesson-loading" class="flex flex-col items-center py-8 gap-4">
                <Spinner test_id="lesson-spinner" />
                <Text size=TextSize::Default variant=TypographyVariant::Muted test_id="lesson-loading-text">
                    {t!(i18n, lesson.loading)}
                </Text>
            </div>
        </Show>

        <Show when=move || error_message.get().is_some() && !is_loading.get()>
            <div data-testid="lesson-error" class="text-center py-8">
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
            <div data-testid="lesson-content" class="relative">
                <Show when=move || is_syncing_cards.get()>
                    <div data-testid="lesson-sync-indicator" class="absolute top-0 right-0 flex items-center gap-1 text-sm text-muted-foreground p-2">
                        <Spinner test_id="lesson-sync-spinner" class=Signal::derive(|| "".to_string()) size=Signal::derive(|| "sm".to_string()) />
                        {t!(i18n, lesson.syncing)}
                    </div>
                </Show>

                <LessonProgressView />
                <LessonCardContainer />
            </div>
        </Show>
    }
}
