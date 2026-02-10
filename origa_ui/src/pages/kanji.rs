use crate::components::cards::kanji_card::{KanjiCard, KanjiCardData};
use crate::components::cards::kanji_detail::{KanjiDetail, KanjiDetailData};
use crate::components::forms::jlpt_level_filter::JlptLevelFilter;
use crate::components::forms::search_bar::SearchBar;
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::services::kanji_service::{KanjiListData, KanjiService};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::JapaneseLevel;
use ulid::Ulid;

#[component]
pub fn Kanji() -> impl IntoView {
    // Get kanji service from context
    let kanji_service = expect_context::<KanjiService>();

    // User ID - in a real app, this would come from auth context
    let user_id = Ulid::new();

    // Search and filter state
    let (search_query, set_search_query) = signal("".to_string());
    let (selected_level, set_selected_level) = signal(JapaneseLevel::N5);

    // Loading states
    let (is_loading, set_is_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Kanji data from service
    let (kanji_data, set_kanji_data) = signal(Vec::<KanjiListData>::new());

    // Detail view state
    let (selected_kanji_detail, set_selected_kanji_detail) = signal(None::<KanjiDetailData>);
    let (_is_loading_detail, set_is_loading_detail) = signal(false);

    // Load kanji data when level changes
    let kanji_service_for_load = kanji_service.clone();
    let set_is_loading_clone = set_is_loading;
    let set_error_clone = set_error;
    let set_kanji_data_clone = set_kanji_data;
    let load_kanji = Action::new(move |level: &JapaneseLevel| {
        let service = kanji_service_for_load.clone();
        let user = user_id;
        let level = *level;
        let set_is_loading = set_is_loading_clone;
        let set_error = set_error_clone;
        let set_kanji_data = set_kanji_data_clone;
        spawn_local(async move {
            set_is_loading.set(true);
            set_error.set(None);

            match service.get_user_kanji_by_level(user, level).await {
                Ok(kanji) => {
                    set_kanji_data.set(kanji);
                    set_is_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Ошибка загрузки кандзи: {}", e)));
                    set_is_loading.set(false);
                }
            }
        });
        async {}
    });

    // Load initial kanji data
    Effect::new(move |_| {
        load_kanji.dispatch(selected_level.get());
    });

    // Filter kanji based on search and JLPT level
    let filtered_kanji = Signal::derive(move || {
        let search = search_query.get().to_lowercase();
        let data = kanji_data.get();

        data.iter()
            .filter(|kanji| {
                // Apply search filter (level is already filtered by service)

                search.is_empty()
                    || kanji.character.to_lowercase().contains(&search)
                    || kanji
                        .meanings
                        .iter()
                        .any(|m| m.to_lowercase().contains(&search))
                    || kanji.radicals.iter().any(|r| {
                        r.character.to_lowercase().contains(&search)
                            || r.meaning.to_lowercase().contains(&search)
                    })
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    let handle_search = Callback::new(move |query: String| {
        set_search_query.set(query);
    });

    let handle_level_select = Callback::new(move |level: JapaneseLevel| {
        set_selected_level.set(level);
    });

    let kanji_service_for_handle = kanji_service.clone();
    let handle_add_kanji = Callback::new(move |kanji_id: String| {
        // Find kanji data by id
        let kanji_data = kanji_data.get().iter().find(|k| k.id == kanji_id).cloned();
        if let Some(kanji) = kanji_data {
            let service = kanji_service_for_handle.clone();
            let level = selected_level.get();
            let user_id = ulid::Ulid::new(); // TODO: получить реальный user_id
            spawn_local(async move {
                match service
                    .add_kanji_to_knowledge_set(user_id, kanji.character.clone())
                    .await
                {
                    Ok(()) => {
                        // Reload kanji data
                        load_kanji.dispatch(level);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Не удалось добавить кандзи: {}", e)));
                    }
                }
            });
        }
    });

    let kanji_service_for_handle2 = kanji_service.clone();
    let handle_remove_kanji = Callback::new(move |kanji_id: String| {
        // Find kanji data by id
        let kanji_data = kanji_data.get().iter().find(|k| k.id == kanji_id).cloned();
        if let Some(kanji) = kanji_data {
            let service = kanji_service_for_handle2.clone();
            let user_id_local = ulid::Ulid::new(); // TODO: получить реальный user_id
            let level = selected_level.get();
            spawn_local(async move {
                match service
                    .remove_kanji_from_knowledge_set(user_id_local, kanji.character.clone())
                    .await
                {
                    Ok(()) => {
                        // Reload kanji data
                        load_kanji.dispatch(level);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Не удалось удалить кандзи: {}", e)));
                    }
                }
            });
        }
    });

    let kanji_service_for_detail = kanji_service.clone();
    let handle_kanji_tap = Callback::new(move |kanji_id: String| {
        // Find kanji character by id
        let kanji_data = kanji_data.get();
        if let Some(kanji) = kanji_data.iter().find(|k| k.id == kanji_id) {
            let service = kanji_service_for_detail.clone();
            let user = user_id;
            let kanji_char = kanji.character.clone();
            set_is_loading_detail.set(true);
            set_selected_kanji_detail.set(None);

            spawn_local(async move {
                match service.get_kanji_detail(kanji_char, user).await {
                    Ok(detail) => {
                        set_selected_kanji_detail.set(Some(detail));
                        set_is_loading_detail.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Ошибка загрузки деталей: {}", e)));
                        set_is_loading_detail.set(false);
                    }
                }
            });
        }
    });

    let handle_back_from_detail = Callback::new(move |_| {
        set_selected_kanji_detail.set(None);
    });

    view! {
        <AppLayout active_tab="kanji"
            .to_string()>
            {move || {
                if let Some(detail) = selected_kanji_detail.get() {
                    view! {
                        <KanjiDetail
                            kanji=detail
                            on_add=handle_add_kanji
                            on_remove=handle_remove_kanji
                            on_back=handle_back_from_detail
                        />
                    }
                        .into_any()
                } else {
                    view! {
                        <PageHeader
                            title=Signal::derive(|| "Кандзи".to_string())
                            subtitle="Изучите японские иероглифы"
                                .to_string()
                        />

                        // Search Bar
                        <SearchBar
                            placeholder="Поиск кандзи или значения"
                            value=search_query
                            on_change=handle_search
                        />

                        // JLPT Level Filter
                        <div class="section">
                            <JlptLevelFilter
                                selected_level=selected_level
                                on_select=handle_level_select
                            />
                        </div>

                        // Kanji List
                        <div class="section">
                            <div class="section-header">
                                <div>
                                    <h2 class="section-title">Список кандзи</h2>
                                    <p class="section-subtitle">
                                        {move || {
                                            let level = selected_level.get();
                                            let count = filtered_kanji.get().len();
                                            format!("{} кандзи уровня {}", count, level)
                                        }}
                                    </p>
                                </div>
                            </div>

                            <div class="kanji-grid">
                                <For
                                    each=move || filtered_kanji.get()
                                    key=|kanji| kanji.id.clone()
                                    children=move |kanji| {
                                        let card_data: KanjiCardData = kanji.into();
                                        view! {
                                            <KanjiCard
                                                card=card_data
                                                on_add=handle_add_kanji
                                                on_remove=handle_remove_kanji
                                                on_tap=handle_kanji_tap
                                            />
                                        }
                                    }
                                />
                            </div>

                            // Loading state
                            <Show when=move || is_loading.get()>
                                <div class="loading-state">
                                    <div class="spinner"></div>
                                    <p class="loading-text">Загрузка кандзи...</p>
                                </div>
                            </Show>

                            // Error state
                            <Show when=move || error.get().is_some()>
                                <div class="error-state">
                                    <div class="error-icon">{"⚠"}</div>
                                    <h3 class="error-title">Ошибка загрузки</h3>
                                    <p class="error-description">
                                        {move || error.get().clone().unwrap_or_default()}
                                    </p>
                                </div>
                            </Show>

                            // Empty state
                            <Show when=move || {
                                !is_loading.get() && error.get().is_none()
                                    && filtered_kanji.get().is_empty()
                            }>
                                <div class="empty-state">
                                    <div class="empty-icon">{"🈁"}</div>
                                    <h3 class="empty-title">Кандзи не найдены</h3>
                                    <p class="empty-description">
                                        {move || {
                                            if search_query.get().is_empty() {
                                                format!(
                                                    "В уровне {} пока нет кандзи",
                                                    selected_level.get(),
                                                )
                                            } else {
                                                format!(
                                                    "По запросу \"{}\" ничего не найдено",
                                                    search_query.get(),
                                                )
                                            }
                                        }}
                                    </p>
                                </div>
                            </Show>
                        </div>
                    }
                        .into_any()
                }
            }}
        </AppLayout>
    }
}
