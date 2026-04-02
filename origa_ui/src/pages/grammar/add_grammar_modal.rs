use super::add_grammar_modal_handlers::ModalHandlers;
use super::add_grammar_modal_state::ModalState;
use super::error_alert::ErrorAlert;
use super::level_selector::LevelSelector;
use super::rules_list::RulesList;
use super::selected_count::SelectedCount;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonSize, ButtonVariant, Drawer, Search, Spinner, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{JapaneseLevel, User};
use origa::traits::UserRepository;
use std::sync::Arc;

const JLPT_LEVELS: [JapaneseLevel; 5] = [
    JapaneseLevel::N5,
    JapaneseLevel::N4,
    JapaneseLevel::N3,
    JapaneseLevel::N2,
    JapaneseLevel::N1,
];

#[component]
pub fn AddGrammarModal(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let repo_for_effect = repository.clone();

    Effect::new(move |_| {
        let repo = repo_for_effect.clone();
        let disposed = StoredValue::new(());
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if disposed.is_disposed() {
                    return;
                }
                current_user.set(Some(user));
            }
        });
    });

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let state = ModalState::new(is_open, refresh_trigger);
    let handlers = ModalHandlers::new(&state, is_open);

    Effect::new({
        let state = state.clone();
        move |_| {
            if is_open.get() {
                state.load_rules();
            }
        }
    });

    view! {
        <Drawer
            is_open=is_open
            title=Signal::derive(|| "Добавить грамматику".to_string())
            test_id="grammar-add-drawer"
            action_button=Arc::new(move || {
                view! {
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        disabled=Signal::derive(move || {
                            state.selected_rule_ids.get().is_empty() || state.is_creating.get()
                        })
                        on_click=handlers.on_add
                        test_id="grammar-drawer-add-btn"
                    >
                        {move || if state.is_creating.get() { "Добавление..." } else { "Добавить" }}
                    </Button>
                }.into_any()
            })
        >
            <div class="space-y-4">
                <LevelSelector
                    levels=JLPT_LEVELS.to_vec()
                    selected_level=state.selected_level
                    on_select={let state = state.clone(); Callback::new(move |level| state.select_level(level))}
                    test_id_prefix="grammar-level"
                />

                <Search
                    value=state.search_query
                    placeholder=Signal::derive(|| "Поиск правила...".to_string())
                    test_id="grammar-drawer-search"
                />

                <div>
                    <div class="flex items-center justify-between mb-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Доступные правила"
                        </Text>
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Ghost)
                            size=Signal::derive(|| ButtonSize::Small)
                            on_click=Callback::new({
                                let state = state.clone();
                                move |_| state.select_all()
                            })
                            test_id="grammar-drawer-select-all-btn"
                        >
                            "Выделить все"
                        </Button>
                    </div>
                    {move || {
                        let is_loading = state.is_loading_rules.get();
                        let rules = state.available_rules.get();

                        if is_loading {
                            view! {
                                <div class="flex flex-col items-center py-4 gap-3">
                                    <Spinner />
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        "Поиск правил грамматики..."
                                    </Text>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <RulesList
                                    rules=rules
                                    native_language=state.native_language.get()
                                    selected_ids=state.selected_rule_ids
                                    search_query=state.search_query
                                    known_kanji=known_kanji.get()
                                />
                            }.into_any()
                        }
                    }}
                </div>

                <SelectedCount count=Signal::derive(move || state.selected_rule_ids.get().len()) />

                <ErrorAlert message=state.error_message />
            </div>
        </Drawer>
    }
}
