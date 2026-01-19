use dioxus::prelude::*;

use crate::components::app_ui::{Card, LoadingState, Paragraph, Pill, SectionHeader, StateTone};
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::switch::{Switch, SwitchThumb};
use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use origa::application::{
    ExportAnkiPackUseCase, ImportWellKnownSetUseCase, ListWellKnownSetsUseCase,
    SyncDuolingoWordsUseCase, WellKnownSetInfo,
};
use origa::domain::WellKnownSets;
use origa::infrastructure::HttpDuolingoClient;
use origa::settings::ApplicationEnvironment;

#[derive(Clone, PartialEq)]
pub enum OperationStatus {
    Idle,
    Loading,
    Success(String),
    Error(String),
}

impl OperationStatus {
    pub fn to_pill_text(&self) -> String {
        match self {
            OperationStatus::Idle => "Готово".to_string(),
            OperationStatus::Loading => "Выполняется...".to_string(),
            OperationStatus::Success(msg) => format!("Успешно: {}", msg),
            OperationStatus::Error(msg) => format!("Ошибка: {}", msg),
        }
    }

    pub fn to_tone(&self) -> StateTone {
        match self {
            OperationStatus::Idle => StateTone::Neutral,
            OperationStatus::Loading => StateTone::Info,
            OperationStatus::Success(_) => StateTone::Success,
            OperationStatus::Error(_) => StateTone::Warning,
        }
    }
}

#[component]
pub fn Import() -> Element {
    let mut active_tab = use_signal(|| "duolingo".to_string());

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Импорт".to_string(),
                subtitle: Some(
                    "Сюда собраны все инструменты импорта/перестройки"
                        .to_string(),
                ),
                actions: None,
            }

            Tabs {
                value: Some(active_tab()),
                on_value_change: move |value| active_tab.set(value),
                default_value: "duolingo".to_string(),

                TabList { class: "grid w-full grid-cols-3",
                    TabTrigger { value: "duolingo".to_string(), index: 0usize, "Duolingo" }
                    TabTrigger { value: "anki".to_string(), index: 1usize, "Anki" }
                    TabTrigger { value: "well_known".to_string(), index: 2usize, "Well Known" }
                }

                TabContent { value: "duolingo".to_string(), index: 0usize, ImportDuolingoTool {} }
                TabContent { value: "anki".to_string(), index: 1usize, ImportAnkiTool {} }
                TabContent { value: "well_known".to_string(), index: 2usize, ImportWellKnownTool {} }
            }
        }
    }
}

#[component]
fn ToolHeader(title: &'static str, subtitle: &'static str) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1",
            Paragraph { class: Some("text-lg font-bold text-slate-800".to_string()), "{title}" }
            Paragraph { class: Some("text-sm text-slate-500".to_string()), "{subtitle}" }
        }
    }
}

#[component]
fn LogsCard(log: Signal<Vec<String>>) -> Element {
    rsx! {
        Card { class: Some("p-3 bg-slate-50 border border-slate-100 rounded-2xl space-y-2".to_string()),
            Paragraph { class: Some("text-sm font-semibold text-slate-700".to_string()), "Логи" }
            for entry in log().iter().rev() {
                Paragraph { class: Some("text-sm text-slate-600".to_string()), {entry.clone()} }
            }
            if log().is_empty() {
                Paragraph { class: Some("text-sm text-slate-500".to_string()), "История пустая." }
            }
        }
    }
}

#[component]
fn ImportDuolingoTool() -> Element {
    let log = use_signal(Vec::<String>::new);
    let status = use_signal(|| OperationStatus::Idle);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "Duolingo синхронизация",
                    subtitle: "Импорт изученных слов из Duolingo",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
            }

            Paragraph { class: Some("text-sm text-slate-600".to_string()),
                "Синхронизация слов, нужен JWT в настройках CLI."
            }


            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Синхронизация с Duolingo...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading),
                    onclick: move |_| {
                        let mut log = log;
                        let mut status = status;
                        status.set(OperationStatus::Loading);
                        spawn(async move {
                            match run_duolingo().await {
                                Ok(msg) => {
                                    status.set(OperationStatus::Success(msg.clone()));
                                    log.write().push(msg);
                                }
                                Err(e) => {
                                    let error_msg = format!("Ошибка: {e}");
                                    status.set(OperationStatus::Error(e.to_string()));
                                    log.write().push(error_msg);
                                }
                            }
                        });
                    },
                    "Синхронизировать"
                }
            }

            LogsCard { log }
        }
    }
}

#[component]
fn ImportAnkiTool() -> Element {
    let file_path = use_signal(|| "deck.apkg".to_string());
    let word_tag = use_signal(|| "Word".to_string());
    let translation_tag = use_signal(|| "Translation".to_string());
    let mut dry_run = use_signal(|| true);
    let log = use_signal(Vec::<String>::new);
    let status = use_signal(|| OperationStatus::Idle);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "Anki импорт",
                    subtitle: "Импорт слов из Anki файла",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
            }

            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ПУТЬ К ФАЙЛУ" }
                Input {
                    placeholder: "anki.apkg",
                    value: file_path(),
                    oninput: {
                        let mut file_path = file_path;
                        move |e: FormEvent| file_path.set(e.value())
                    },
                    disabled: matches!(status(), OperationStatus::Loading),
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ТЕГ ВОПРОСА" }
                Input {
                    placeholder: "Word",
                    value: word_tag(),
                    oninput: {
                        let mut word_tag = word_tag;
                        move |e: FormEvent| word_tag.set(e.value())
                    },
                    disabled: matches!(status(), OperationStatus::Loading),
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ТЕГ ПЕРЕВОДА" }
                Input {
                    placeholder: "Translation",
                    value: translation_tag(),
                    oninput: {
                        let mut translation_tag = translation_tag;
                        move |e: FormEvent| translation_tag.set(e.value())
                    },
                    disabled: matches!(status(), OperationStatus::Loading),
                }
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm font-medium",
                    "Dry-run (показать без сохранения)"
                }
                Switch {
                    aria_label: "Dry-run",
                    checked: dry_run(),
                    on_checked_change: move |v| dry_run.set(v),
                    disabled: matches!(status(), OperationStatus::Loading),
                    SwitchThumb {}
                }
            }

            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Импорт из Anki файла...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading),
                    onclick: {
                        move |_| {
                            let file = file_path();
                            let word = word_tag();
                            let translation = translation_tag();
                            let is_dry = dry_run();
                            let mut log = log;
                            let mut status = status;
                            status.set(OperationStatus::Loading);
                            spawn(async move {
                                let res = if is_dry {
                                    run_anki_dry(file.clone(), word.clone(), translation.clone()).await
                                } else {
                                    run_anki(file.clone(), word.clone(), translation.clone()).await
                                };
                                match res {
                                    Ok(msg) => {
                                        status.set(OperationStatus::Success(msg.clone()));
                                        log.write().push(msg);
                                    }
                                    Err(e) => {
                                        let error_msg = format!("Ошибка: {e}");
                                        status.set(OperationStatus::Error(e.to_string()));
                                        log.write().push(error_msg);
                                    }
                                }
                            });
                        }
                    },
                    "Запустить"
                }
            }

            LogsCard { log }
        }
    }
}

#[component]
fn ImportWellKnownTool() -> Element {
    let sets: Signal<Option<Vec<WellKnownSetInfo>>> = use_signal(|| None);
    let mut selected: Signal<Option<WellKnownSets>> = use_signal(|| None);
    let log = use_signal(Vec::<String>::new);
    let status = use_signal(|| OperationStatus::Idle);

    use_effect(move || {
        let mut sets = sets;
        spawn(async move {
            match load_well_known_sets().await {
                Ok(loaded_sets) => {
                    sets.set(Some(loaded_sets));
                }
                Err(_) => {
                    // Failed to load sets
                }
            }
        });
    });

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "Well Known Sets импорт",
                    subtitle: "Импорт слов из известных наборов (JLPT, Migii)",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
            }

            Paragraph { class: Some("text-sm text-slate-600".to_string()),
                "Выберите набор для импорта."
            }

            div { class: "space-y-2",
                label { class: "text-sm font-medium", "НАБОР" }
                if let Some(sets_list) = sets() {
                    select {
                        class: "w-full p-2 border border-slate-300 rounded-lg",
                        disabled: matches!(status(), OperationStatus::Loading),
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            if value.is_empty() {
                                selected.set(None);
                                return;
                            }
                            let index: usize = match value.parse() {
                                Ok(idx) => idx,
                                Err(_) => return,
                            };
                            if let Some(info) = sets_list.get(index) {
                                selected.set(Some(info.set));
                            }
                        },
                        option { value: "", "Выберите набор..." }
                        for (idx , info) in sets_list.iter().enumerate() {
                            option {
                                value: idx.to_string(),
                                selected: selected().map(|s| s == info.set).unwrap_or(false),
                                "{info.title}"
                            }
                        }
                    }
                } else {
                    LoadingState { message: Some("Загрузка наборов...".to_string()) }
                }
            }

            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Импорт набора...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading) || selected().is_none(),
                    onclick: move |_| {
                        if selected().is_none() {
                            return;
                        }
                        let set = selected().unwrap();
                        let mut log = log;
                        let mut status = status;
                        status.set(OperationStatus::Loading);
                        spawn(async move {
                            match run_well_known_set(set).await {
                                Ok(msg) => {
                                    status.set(OperationStatus::Success(msg.clone()));
                                    log.write().push(msg);
                                }
                                Err(e) => {
                                    let error_msg = format!("Ошибка: {e}");
                                    status.set(OperationStatus::Error(e.to_string()));
                                    log.write().push(error_msg);
                                }
                            }
                        });
                    },
                    "Импортировать"
                }
            }

            LogsCard { log }
        }
    }
}

async fn load_well_known_sets() -> Result<Vec<WellKnownSetInfo>, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let use_case = ListWellKnownSetsUseCase::new(repo);
    use_case.execute(user_id).await.map_err(to_error)
}

async fn run_duolingo() -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let client = HttpDuolingoClient::new();
    let use_case = SyncDuolingoWordsUseCase::new(repo, &llm, &client);
    let res = use_case.execute(user_id).await.map_err(to_error)?;
    Ok(format!(
        "Duolingo: создано {}, пропущено {}",
        res.total_created_count,
        res.skipped_words.len()
    ))
}

async fn run_anki_dry(
    file_path: String,
    word_tag: String,
    translation_tag: String,
) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let use_case = ExportAnkiPackUseCase::new(repo, &llm);
    let cards = use_case
        .extract_cards(&file_path, &word_tag, Some(translation_tag.as_str()))
        .await
        .map_err(to_error)?;
    Ok(format!("Dry-run: найдено {} карточек", cards.len()))
}

async fn run_anki(
    file_path: String,
    word_tag: String,
    translation_tag: String,
) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let use_case = ExportAnkiPackUseCase::new(repo, &llm);
    let result = use_case
        .execute(user_id, file_path, word_tag, Some(translation_tag))
        .await
        .map_err(to_error)?;
    Ok(format!(
        "Импорт Anki: создано {}, пропущено {}",
        result.total_created_count,
        result.skipped_words.len()
    ))
}

async fn run_well_known_set(set: WellKnownSets) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;

    let res = ImportWellKnownSetUseCase::new(repo, &llm)
        .execute(user_id, set)
        .await
        .map_err(to_error)?;

    Ok(format!(
        "Well Known Set: создано {}, пропущено {}",
        res.total_created_count,
        res.skipped_words.len()
    ))
}
