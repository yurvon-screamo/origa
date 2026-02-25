use crate::app::update_current_user;
use crate::repository::SupabaseUserRepository;
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{ImportWellKnownSetUseCase, ListWellKnownSetsUseCase};
use origa::domain::{User, WellKnownSets};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SetType {
    Jlpt,
    Migii,
}

impl SetType {
    fn label(&self) -> &'static str {
        match self {
            SetType::Jlpt => "JLPT",
            SetType::Migii => "Migii",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum JlptLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JlptLevel {
    fn label(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "N5",
            JlptLevel::N4 => "N4",
            JlptLevel::N3 => "N3",
            JlptLevel::N2 => "N2",
            JlptLevel::N1 => "N1",
        }
    }
}

#[derive(Clone, PartialEq)]
struct SetInfo {
    set: WellKnownSets,
    title: String,
    description: String,
    word_count: usize,
    set_type: SetType,
    level: JlptLevel,
}

fn classify_set(set: &WellKnownSets) -> (SetType, JlptLevel) {
    match set {
        WellKnownSets::JlptN5 => (SetType::Jlpt, JlptLevel::N5),
        WellKnownSets::JlptN4 => (SetType::Jlpt, JlptLevel::N4),
        WellKnownSets::JlptN3 => (SetType::Jlpt, JlptLevel::N3),
        WellKnownSets::JlptN2 => (SetType::Jlpt, JlptLevel::N2),
        WellKnownSets::JlptN1 => (SetType::Jlpt, JlptLevel::N1),
        WellKnownSets::MigiiN5Lesson1
        | WellKnownSets::MigiiN5Lesson2
        | WellKnownSets::MigiiN5Lesson3
        | WellKnownSets::MigiiN5Lesson4
        | WellKnownSets::MigiiN5Lesson5
        | WellKnownSets::MigiiN5Lesson6
        | WellKnownSets::MigiiN5Lesson7
        | WellKnownSets::MigiiN5Lesson8
        | WellKnownSets::MigiiN5Lesson9
        | WellKnownSets::MigiiN5Lesson10
        | WellKnownSets::MigiiN5Lesson11
        | WellKnownSets::MigiiN5Lesson12
        | WellKnownSets::MigiiN5Lesson13
        | WellKnownSets::MigiiN5Lesson14
        | WellKnownSets::MigiiN5Lesson15
        | WellKnownSets::MigiiN5Lesson16
        | WellKnownSets::MigiiN5Lesson17
        | WellKnownSets::MigiiN5Lesson18
        | WellKnownSets::MigiiN5Lesson19
        | WellKnownSets::MigiiN5Lesson20 => (SetType::Migii, JlptLevel::N5),
        WellKnownSets::MigiiN4Lesson1
        | WellKnownSets::MigiiN4Lesson2
        | WellKnownSets::MigiiN4Lesson3
        | WellKnownSets::MigiiN4Lesson4
        | WellKnownSets::MigiiN4Lesson5
        | WellKnownSets::MigiiN4Lesson6
        | WellKnownSets::MigiiN4Lesson7
        | WellKnownSets::MigiiN4Lesson8
        | WellKnownSets::MigiiN4Lesson9
        | WellKnownSets::MigiiN4Lesson10
        | WellKnownSets::MigiiN4Lesson11 => (SetType::Migii, JlptLevel::N4),
        WellKnownSets::MigiiN3Lesson1
        | WellKnownSets::MigiiN3Lesson2
        | WellKnownSets::MigiiN3Lesson3
        | WellKnownSets::MigiiN3Lesson4
        | WellKnownSets::MigiiN3Lesson5
        | WellKnownSets::MigiiN3Lesson6
        | WellKnownSets::MigiiN3Lesson7
        | WellKnownSets::MigiiN3Lesson8
        | WellKnownSets::MigiiN3Lesson9
        | WellKnownSets::MigiiN3Lesson10
        | WellKnownSets::MigiiN3Lesson11
        | WellKnownSets::MigiiN3Lesson12
        | WellKnownSets::MigiiN3Lesson13
        | WellKnownSets::MigiiN3Lesson14
        | WellKnownSets::MigiiN3Lesson15
        | WellKnownSets::MigiiN3Lesson16
        | WellKnownSets::MigiiN3Lesson17
        | WellKnownSets::MigiiN3Lesson18
        | WellKnownSets::MigiiN3Lesson19
        | WellKnownSets::MigiiN3Lesson20
        | WellKnownSets::MigiiN3Lesson21
        | WellKnownSets::MigiiN3Lesson22
        | WellKnownSets::MigiiN3Lesson23
        | WellKnownSets::MigiiN3Lesson24
        | WellKnownSets::MigiiN3Lesson25
        | WellKnownSets::MigiiN3Lesson26
        | WellKnownSets::MigiiN3Lesson27
        | WellKnownSets::MigiiN3Lesson28
        | WellKnownSets::MigiiN3Lesson29
        | WellKnownSets::MigiiN3Lesson30
        | WellKnownSets::MigiiN3Lesson31 => (SetType::Migii, JlptLevel::N3),
        WellKnownSets::MigiiN2Lesson1
        | WellKnownSets::MigiiN2Lesson2
        | WellKnownSets::MigiiN2Lesson3
        | WellKnownSets::MigiiN2Lesson4
        | WellKnownSets::MigiiN2Lesson5
        | WellKnownSets::MigiiN2Lesson6
        | WellKnownSets::MigiiN2Lesson7
        | WellKnownSets::MigiiN2Lesson8
        | WellKnownSets::MigiiN2Lesson9
        | WellKnownSets::MigiiN2Lesson10
        | WellKnownSets::MigiiN2Lesson11
        | WellKnownSets::MigiiN2Lesson12
        | WellKnownSets::MigiiN2Lesson13
        | WellKnownSets::MigiiN2Lesson14
        | WellKnownSets::MigiiN2Lesson15
        | WellKnownSets::MigiiN2Lesson16
        | WellKnownSets::MigiiN2Lesson17
        | WellKnownSets::MigiiN2Lesson18
        | WellKnownSets::MigiiN2Lesson19
        | WellKnownSets::MigiiN2Lesson20
        | WellKnownSets::MigiiN2Lesson21
        | WellKnownSets::MigiiN2Lesson22
        | WellKnownSets::MigiiN2Lesson23
        | WellKnownSets::MigiiN2Lesson24
        | WellKnownSets::MigiiN2Lesson25
        | WellKnownSets::MigiiN2Lesson26
        | WellKnownSets::MigiiN2Lesson27
        | WellKnownSets::MigiiN2Lesson28
        | WellKnownSets::MigiiN2Lesson29
        | WellKnownSets::MigiiN2Lesson30
        | WellKnownSets::MigiiN2Lesson31 => (SetType::Migii, JlptLevel::N2),
        _ => (SetType::Migii, JlptLevel::N1),
    }
}

#[component]
pub fn SetsContent() -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");
    let repository = use_context::<SupabaseUserRepository>().expect("repository context");
    let llm_service = use_context::<origa::infrastructure::LlmServiceInvoker>().expect("llm_service context");

    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let importing: RwSignal<Option<WellKnownSets>> = RwSignal::new(None);
    let import_result: RwSignal<Option<String>> = RwSignal::new(None);

    let repository_for_load = repository.clone();
    let current_user_for_load = current_user;
    let sets_for_load = sets;

    spawn_local(async move {
        if let Some(user) = current_user_for_load.get_untracked() {
            let use_case = ListWellKnownSetsUseCase::new(&repository_for_load);
            if let Ok(set_infos) = use_case.execute(user.id()).await {
                let set_list: Vec<SetInfo> = set_infos
                    .into_iter()
                    .map(|info| {
                        let (set_type, level) = classify_set(&info.set);
                        let word_count = origa::domain::load_well_known_set(&info.set)
                            .map(|s| s.words().len())
                            .unwrap_or(0);
                        SetInfo {
                            set: info.set,
                            title: info.title,
                            description: info.description,
                            word_count,
                            set_type,
                            level,
                        }
                    })
                    .collect();
                sets_for_load.set(set_list);
            }
        }
    });

    let jlpt_levels = [JlptLevel::N5, JlptLevel::N4, JlptLevel::N3, JlptLevel::N2, JlptLevel::N1];

    let on_import_base = {
        let repo = repository.clone();
        let llm = llm_service.clone();
        let current_user = current_user;
        let importing = importing;
        let import_result = import_result;
        move |set: WellKnownSets| {
            let repo = repo.clone();
            let llm = llm.clone();
            let current_user = current_user;
            let importing = importing;
            let import_result = import_result;
            spawn_local(async move {
                if let Some(user) = current_user.get_untracked() {
                    importing.set(Some(set));
                    import_result.set(None);
                    let use_case = ImportWellKnownSetUseCase::new(&repo, &llm);
                    match use_case.execute(user.id(), set).await {
                        Ok(result) => {
                            import_result.set(Some(format!(
                                "Импортировано {} слов",
                                result.total_created_count
                            )));
                            update_current_user(repo.clone(), current_user);
                        }
                        Err(e) => {
                            import_result.set(Some(format!("Ошибка: {}", e)));
                        }
                    }
                    importing.set(None);
                }
            });
        }
    };
    let on_import_base = StoredValue::new(on_import_base);

    view! {
        <div class="sets-page">
            <Heading level=HeadingLevel::H2 class="mb-6">
                "Наборы для изучения"
            </Heading>

            <Show when=move || import_result.get().is_some()>
                <div class="mb-4 p-4 border border-[var(--border-dark)] bg-[var(--bg-paper)]">
                    <Text size=TextSize::Default>
                        {move || import_result.get().unwrap_or_default()}
                    </Text>
                </div>
            </Show>

            <For
                each=move || jlpt_levels
                key=|level| format!("{:?}", level)
                children=move |level| {
                    let sets_for_level = Memo::new(move |_| {
                        sets.get()
                            .into_iter()
                            .filter(|s| s.level == level)
                            .collect::<Vec<_>>()
                    });

                    view! {
                        <Show when=move || !sets_for_level.get().is_empty()>
                            <div class="sets-group">
                                <div class="sets-group-title">
                                    {format!("Уровень {}", level.label())}
                                </div>

                                <For
                                    each=move || [SetType::Jlpt, SetType::Migii]
                                    key=|t| format!("{:?}", t)
                                    children=move |set_type| {
                                        let sets_for_type = Memo::new(move |_| {
                                            sets_for_level.get()
                                                .into_iter()
                                                .filter(|s| s.set_type == set_type)
                                                .collect::<Vec<_>>()
                                        });

                                        view! {
                                            <Show when=move || !sets_for_type.get().is_empty()>
                                                <div class="mb-4">
                                                    <Text
                                                        size=TextSize::Small
                                                        variant=TypographyVariant::Muted
                                                        class="mb-2"
                                                    >
                                                        {set_type.label()}
                                                    </Text>
                                                    <div class="sets-list">
                                                        <For
                                                            each=move || sets_for_type.get()
                                                            key=|s| format!("{:?}", s.set)
                                                            children=move |set_info| {
                                                                let is_importing = Memo::new(move |_| {
                                                                    importing.get() == Some(set_info.set)
                                                                });
                                                                let on_import = on_import_base.get_value().clone();

                                                                view! {
                                                                    <div class="set-card">
                                                                        <div class="set-card-title">
                                                                            {set_info.title.clone()}
                                                                        </div>
                                                                        <div class="set-card-description">
                                                                            {set_info.description.clone()}
                                                                        </div>
                                                                        <div class="set-card-footer">
                                                                            <span class="set-card-count">
                                                                                {format!("{} слов", set_info.word_count)}
                                                                            </span>
                                                                            <Button
                                                                                variant=Signal::derive(|| ButtonVariant::Filled)
                                                                                on_click=Callback::new({
                                                                                    let set = set_info.set;
                                                                                    let on_import = on_import.clone();
                                                                                    move |_| on_import(set)
                                                                                })
                                                                                disabled=is_importing
                                                                            >
                                                                                {move || if is_importing.get() {
                                                                                    "Импорт..."
                                                                                } else {
                                                                                    "Импорт"
                                                                                }}
                                                                            </Button>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </Show>
                                        }
                                    }
                                />
                            </div>
                        </Show>
                    }
                }
            />
        </div>
    }
}
