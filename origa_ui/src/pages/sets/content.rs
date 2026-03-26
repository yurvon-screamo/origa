use super::filters::{available_set_types, ImportFilter, LevelFilter, TypeFilter};
use super::import_set_preview_modal::ImportSetPreviewModal;
use super::sets_level_group::SetsLevelGroup;
use super::types::SetInfo;
use crate::loaders::WellKnownSetLoaderImpl;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, Input, Spinner, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{JapaneseLevel, User};
use origa::traits::{TypeMeta, UserRepository, WellKnownSetLoader};
use std::collections::{HashMap, HashSet};

#[component]
pub fn SetsContent() -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let is_loading: RwSignal<bool> = RwSignal::new(true);
    let preview_modal_open = RwSignal::new(false);
    let preview_set_ids = RwSignal::new(Vec::<String>::new());
    let preview_set_titles = RwSignal::new(HashMap::<String, String>::new());
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let level_filter = RwSignal::new(LevelFilter::default());
    let type_filter = RwSignal::new(TypeFilter::default());
    let import_filter = RwSignal::new(ImportFilter::default());
    let search = RwSignal::new(String::new());
    let selected_sets: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let repo_for_init = repository.clone();
    let loader = WellKnownSetLoaderImpl::new();
    let sets_for_load = sets;
    let initialized = RwSignal::new(false);

    Effect::new(move |_| {
        if initialized.get() {
            return;
        }
        initialized.set(true);

        let repo = repo_for_init.clone();
        let loader = loader.clone();
        spawn_local(async move {
            let user = match repo.get_current_user().await {
                Ok(Some(user)) => {
                    current_user.set(Some(user.clone()));
                    Some(user)
                },
                Ok(None) => {
                    tracing::warn!("SetsContent: user not found");
                    None
                },
                Err(e) => {
                    tracing::error!("SetsContent: get_current_user error: {:?}", e);
                    None
                },
            };

            match loader.load_meta_list().await {
                Ok(meta_list) => {
                    let set_list: Vec<SetInfo> = meta_list
                        .into_iter()
                        .map(|meta| {
                            let is_imported = user
                                .as_ref()
                                .map(|u| u.is_set_imported(&meta.id))
                                .unwrap_or(false);

                            SetInfo {
                                set_id: meta.id,
                                title: meta.title_ru,
                                description: meta.desc_ru,
                                word_count: Some(meta.word_count),
                                set_type: meta.set_type,
                                level: meta.level,
                                is_imported,
                            }
                        })
                        .collect();
                    sets_for_load.set(set_list);
                },
                Err(e) => {
                    tracing::error!("SetsContent: load_meta_list error: {:?}", e);
                },
            }
            is_loading.set(false);
        });
    });

    let sets_for_update = sets;
    Effect::new(move |_| {
        let user = current_user.get();
        sets_for_update.update(|set_list| {
            if let Some(u) = user {
                for set in set_list.iter_mut() {
                    set.is_imported = u.is_set_imported(&set.set_id);
                }
            }
        });
    });

    let filtered_sets = Memo::new(move |_| {
        let level = level_filter.get();
        let type_f = type_filter.get();
        let import_f = import_filter.get();
        let query = search.get().to_lowercase();

        sets.get()
            .into_iter()
            .filter(|s| {
                let matches_level = level.matches(s.level);
                let matches_type = type_f.matches(&s.set_type);
                let matches_import = import_f.matches(s.is_imported);
                let matches_search = query.is_empty()
                    || s.title.to_lowercase().contains(&query)
                    || s.description.to_lowercase().contains(&query);
                matches_level && matches_type && matches_import && matches_search
            })
            .collect::<Vec<_>>()
    });

    let on_import = Callback::new(move |(set_id, title): (String, String)| {
        preview_set_ids.set(vec![set_id.clone()]);
        preview_set_titles.update(|m| {
            m.clear();
            m.insert(set_id, title);
        });
        preview_modal_open.set(true);
    });

    let on_toggle_select = Callback::new(move |set_id: String| {
        selected_sets.update(|sets| {
            if sets.contains(&set_id) {
                sets.remove(&set_id);
            } else {
                sets.insert(set_id);
            }
        });
    });

    let known_kanji_clone = known_kanji.get();

    view! {
        <div class="sets-page">
            <Show when=move || is_loading.get()>
                <div class="flex flex-col items-center py-8 gap-4" data-testid="sets-loading">
                    <Spinner />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id="sets-loading-text">
                        "Загрузка списков слов..."
                    </Text>
                </div>
            </Show>
            <Show when=move || !is_loading.get()>
                <div class="space-y-4 mb-6">
                    <Input
                        value=search
                        placeholder=Signal::derive(|| "Поиск наборов...".to_string())
                        test_id="sets-search-input"
                    />

                    <div class="flex flex-wrap gap-2" data-testid="sets-level-filters">
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::All {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-level-all"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::All);
                            })
                        >
                            {LevelFilter::All.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::N5 {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-level-n5"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N5);
                            })
                        >
                            {LevelFilter::N5.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::N4 {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-level-n4"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N4);
                            })
                        >
                            {LevelFilter::N4.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::N3 {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-level-n3"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N3);
                            })
                        >
                            {LevelFilter::N3.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::N2 {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-level-n2"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N2);
                            })
                        >
                            {LevelFilter::N2.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::N1 {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-level-n1"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N1);
                            })
                        >
                            {LevelFilter::N1.label()}
                        </Tag>
                    </div>

                    <div class="flex flex-wrap gap-2" data-testid="sets-type-filters">
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get().is_all() {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-type-all"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::all());
                            })
                        >
                            "Все типы"
                        </Tag>
                        <For
                            each=move || available_set_types()
                            key=|type_meta: &TypeMeta| type_meta.id.clone()
                            children=move |type_meta| {
                                let type_id = type_meta.id.clone();
                                let type_id_for_click = type_id.clone();
                                let type_filter = type_filter;
                                let label = type_meta.label_ru.clone();
                                let test_id_val = format!("sets-type-{}", type_id);
                                view! {
                                    <Tag
                                        variant=Signal::derive(move || {
                                            match &type_filter.get().0 {
                                                Some(filter_id) if filter_id == &type_id => TagVariant::Filled,
                                                _ => TagVariant::Default,
                                            }
                                        })
                                        class=Signal::derive(|| "cursor-pointer".to_string())
                                        test_id=Signal::derive(move || test_id_val.clone())
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            type_filter.set(TypeFilter::specific(&type_id_for_click));
                                        })
                                    >
                                        {label}
                                    </Tag>
                                }
                            }
                        />
                    </div>

                    <div class="flex flex-wrap gap-2" data-testid="sets-import-filters">
                        <Tag
                            variant=Signal::derive(move || {
                                if import_filter.get() == ImportFilter::All {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-import-all"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::All);
                            })
                        >
                            {ImportFilter::All.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if import_filter.get() == ImportFilter::Imported {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-import-imported"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::Imported);
                            })
                        >
                            {ImportFilter::Imported.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if import_filter.get() == ImportFilter::New {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            test_id="sets-import-new"
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::New);
                            })
                        >
                            {ImportFilter::New.label()}
                        </Tag>
                    </div>

                    <Show when=move || !selected_sets.get().is_empty()>
                        <div class="flex items-center gap-3 pt-2">
                            <Button
                                variant=ButtonVariant::Olive
                                test_id="sets-import-selected-btn"
                                on_click=Callback::new(move |_| {
                                    let set_ids: Vec<String> = selected_sets.get().into_iter().collect();
                                    let count = set_ids.len();
                                    if count > 0 {
                                        let all_sets = sets.get();
                                        let titles: HashMap<String, String> = all_sets
                                            .iter()
                                            .filter(|s| set_ids.contains(&s.set_id))
                                            .map(|s| (s.set_id.clone(), s.title.clone()))
                                            .collect();

                                        preview_set_ids.set(set_ids);
                                        preview_set_titles.set(titles);
                                        preview_modal_open.set(true);
                                    }
                                })
                            >
                                {move || format!("Импортировать {} наборов", selected_sets.get().len())}
                            </Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                test_id="sets-cancel-select-btn"
                                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                    selected_sets.set(HashSet::new());
                                })
                            >
                                "Отменить выбор"
                            </Button>
                        </div>
                    </Show>
                </div>

                <SetsLevelGroup
                    level=JapaneseLevel::N5
                    sets=filtered_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N4
                    sets=filtered_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N3
                    sets=filtered_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N2
                    sets=filtered_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N1
                    sets=filtered_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
            </Show>
            <ImportSetPreviewModal
                is_open=preview_modal_open
                set_ids=Signal::derive(move || preview_set_ids.get())
                set_titles=Signal::derive(move || preview_set_titles.get())
                on_import_result=Callback::new(move |imported_ids: Vec<String>| {
                    if !imported_ids.is_empty() {
                        sets.update(|list| {
                            for set in list.iter_mut() {
                                if imported_ids.contains(&set.set_id) {
                                    set.is_imported = true;
                                }
                            }
                        });
                    }
                    selected_sets.set(HashSet::new());
                })
            />
        </div>
    }
}
