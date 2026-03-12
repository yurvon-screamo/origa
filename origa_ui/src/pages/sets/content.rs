use super::filters::{ImportFilter, LevelFilter, TypeFilter};
use super::import_set_preview_modal::ImportSetPreviewModal;
use super::sets_level_group::SetsLevelGroup;
use super::types::SetInfo;
use crate::ui_components::{Input, Spinner, Tag, TagVariant, Text, TextSize, TypographyVariant};
use crate::well_known_set::WellKnownSetLoaderImpl;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{JapaneseLevel, User};
use origa::use_cases::ListWellKnownSetsUseCase;

#[component]
pub fn SetsContent() -> impl IntoView {
    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let is_loading: RwSignal<bool> = RwSignal::new(true);
    let preview_modal_open = RwSignal::new(false);
    let preview_set_id = RwSignal::new(String::new());
    let preview_set_title = RwSignal::new(String::new());
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");
    let level_filter = RwSignal::new(LevelFilter::default());
    let type_filter = RwSignal::new(TypeFilter::default());
    let import_filter = RwSignal::new(ImportFilter::default());
    let search = RwSignal::new(String::new());

    let loader = WellKnownSetLoaderImpl::new();
    let sets_for_load = sets;
    let user_for_load = current_user;

    spawn_local(async move {
        let use_case = ListWellKnownSetsUseCase::new(&loader);
        if let Ok(set_infos) = use_case.execute().await {
            let set_list: Vec<SetInfo> = set_infos
                .into_iter()
                .map(|info| {
                    let is_imported = user_for_load
                        .get()
                        .map(|u| u.is_set_imported(&info.meta.id))
                        .unwrap_or(false);

                    SetInfo {
                        set_id: info.meta.id,
                        title: info.meta.title_ru,
                        description: info.meta.desc_ru,
                        word_count: info.word_count,
                        set_type: info.meta.set_type,
                        level: info.meta.level,
                        is_imported,
                    }
                })
                .collect();
            sets_for_load.set(set_list);
            is_loading.set(false);
        }
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
                let matches_type = type_f.matches(s.set_type);
                let matches_import = import_f.matches(s.is_imported);
                let matches_search = query.is_empty()
                    || s.title.to_lowercase().contains(&query)
                    || s.description.to_lowercase().contains(&query);
                matches_level && matches_type && matches_import && matches_search
            })
            .collect::<Vec<_>>()
    });

    let on_import = Callback::new(move |(set_id, title): (String, String)| {
        preview_set_id.set(set_id);
        preview_set_title.set(title);
        preview_modal_open.set(true);
    });

    view! {
        <div class="sets-page">
            <Show when=move || is_loading.get()>
                <div class="flex flex-col items-center py-8 gap-4">
                    <Spinner />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Загрузка списков слов..."
                    </Text>
                </div>
            </Show>
            <Show when=move || !is_loading.get()>
                <div class="space-y-4 mb-6">
                    <Input
                        value=search
                        placeholder=Signal::derive(|| "Поиск наборов...".to_string())
                    />

                    <div class="flex flex-wrap gap-2">
                        <Tag
                            variant=Signal::derive(move || {
                                if level_filter.get() == LevelFilter::All {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
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
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N1);
                            })
                        >
                            {LevelFilter::N1.label()}
                        </Tag>
                    </div>

                    <div class="flex flex-wrap gap-2">
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get() == TypeFilter::All {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::All);
                            })
                        >
                            {TypeFilter::All.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get() == TypeFilter::Jlpt {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::Jlpt);
                            })
                        >
                            {TypeFilter::Jlpt.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get() == TypeFilter::Migii {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::Migii);
                            })
                        >
                            {TypeFilter::Migii.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get() == TypeFilter::SpyFamily {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::SpyFamily);
                            })
                        >
                            {TypeFilter::SpyFamily.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get() == TypeFilter::DuolingoRu {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::DuolingoRu);
                            })
                        >
                            {TypeFilter::DuolingoRu.label()}
                        </Tag>
                        <Tag
                            variant=Signal::derive(move || {
                                if type_filter.get() == TypeFilter::DuolingoEn {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::DuolingoEn);
                            })
                        >
                            {TypeFilter::DuolingoEn.label()}
                        </Tag>
                    </div>

                    <div class="flex flex-wrap gap-2">
                        <Tag
                            variant=Signal::derive(move || {
                                if import_filter.get() == ImportFilter::All {
                                    TagVariant::Filled
                                } else {
                                    TagVariant::Default
                                }
                            })
                            class=Signal::derive(|| "cursor-pointer".to_string())
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
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::New);
                            })
                        >
                            {ImportFilter::New.label()}
                        </Tag>
                    </div>
                </div>

                <SetsLevelGroup
                    level=JapaneseLevel::N5
                    sets=filtered_sets
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N4
                    sets=filtered_sets
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N3
                    sets=filtered_sets
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N2
                    sets=filtered_sets
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N1
                    sets=filtered_sets
                    type_filter=type_filter
                    on_import=on_import
                />
            </Show>
            <ImportSetPreviewModal
                is_open=preview_modal_open
                set_id=Signal::derive(move || preview_set_id.get())
                set_title=Signal::derive(move || preview_set_title.get())
                on_import_result=Callback::new(move |_| {})
            />
        </div>
    }
}
