use super::add_drawer_handlers::DrawerHandlers;
use super::add_drawer_state::DrawerState;
use super::counter_list::CounterList;
use crate::i18n::{t, use_i18n};
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonSize, ButtonVariant, Drawer, ErrorAlert, Input, LevelSelector, SelectedCount,
    Spinner, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::dictionary::counters::gloss_for;
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
pub fn AddCountersDrawer(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
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

    let known_counters = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_counters())
            .unwrap_or_default()
    });

    let native_language =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let state = DrawerState::new(is_open, refresh_trigger);
    let handlers = DrawerHandlers::new(&state, is_open);

    let filtered_counters = Memo::new(move |_| {
        let query = state.search_query.get().to_lowercase();
        let counter_list = state.available_counters.get();
        let lang = native_language.get();
        if query.is_empty() {
            counter_list
        } else {
            counter_list
                .into_iter()
                .filter(|entry| {
                    let suffix = entry.suffix().to_lowercase();
                    let gloss = gloss_for(entry, lang).to_lowercase();
                    suffix.contains(&query) || gloss.contains(&query)
                })
                .collect()
        }
    });

    Effect::new({
        let state = state.clone();
        move |_| {
            if is_open.get() {
                state.load_counters();
            }
        }
    });

    view! {
        <Drawer
            is_open=is_open
            title=Signal::derive(move || i18n.get_keys().counters().add_counters().inner().to_string())
            test_id="counters-add-drawer"
            action_button=Arc::new(move || {
                view! {
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        disabled=Signal::derive(move || {
                            state.selected_counters.get().is_empty() || state.is_creating.get()
                        })
                        on_click=handlers.on_add
                        test_id="counters-drawer-add-btn"
                    >
                        {move || if state.is_creating.get() { t!(i18n, counters.adding).into_any() } else { t!(i18n, counters.add).into_any() }}
                    </Button>
                }.into_any()
            })
        >
            <div class="space-y-4">
                <LevelSelector
                    levels=JLPT_LEVELS.to_vec()
                    selected_level=state.selected_level
                    on_select={let state = state.clone(); Callback::new(move |level| state.select_level(level))}
                    test_id_prefix="counters-level"
                />

                <Input
                    value=state.search_query
                    placeholder=Signal::derive(move || i18n.get_keys().common().search().inner().to_string())
                    test_id="counters-drawer-search"
                />

                <div>
                    <div class="flex items-center justify-between mb-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, counters.available_counters)}
                        </Text>
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Ghost)
                            size=Signal::derive(|| ButtonSize::Small)
                            on_click=Callback::new({
                                let state = state.clone();
                                move |_| state.select_all()
                            })
                            test_id="counters-drawer-select-all-btn"
                        >
                            {t!(i18n, common.select_all)}
                        </Button>
                    </div>
                    {move || {
                        let is_loading = state.is_loading_counters.get();

                        if is_loading {
                            view! {
                                <div class="flex flex-col items-center py-4 gap-3">
                                    <Spinner />
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        {t!(i18n, counters.searching_counters)}
                                    </Text>
                                </div>
                            }.into_any()
                        } else {
                            let counter_list = filtered_counters.get();
                            view! {
                                <CounterList
                                    counter_list=counter_list
                                    native_language=native_language
                                    selected_counters=state.selected_counters
                                    known_counters=known_counters.get()
                                />
                            }.into_any()
                        }
                    }}
                </div>

                <SelectedCount count=Signal::derive(move || state.selected_counters.get().len()) />

                <ErrorAlert message=state.error_message />
            </div>
        </Drawer>
    }
}
