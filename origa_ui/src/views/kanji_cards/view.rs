use dioxus::prelude::*;
use origa::application::{CreateKanjiCardUseCase, KanjiListUseCase};
use origa::domain::JapaneseLevel;

use crate::components::app_ui::{Card, H2};
use crate::components::button::{Button, ButtonVariant};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus_primitives::toast::{ToastOptions, use_toast};
use origa::settings::ApplicationEnvironment;

#[derive(Clone, PartialEq)]
struct KanjiReferenceCard {
    kanji: String,
    description: String,
    level: JapaneseLevel,
}

#[component]
pub fn KanjiCards() -> Element {
    let kanjis_resource = use_resource(fetch_all_kanjis);

    match kanjis_resource.read().as_ref() {
        Some(Ok(kanjis)) => {
            let levels: Vec<(JapaneseLevel, Vec<KanjiReferenceCard>)> = vec![
                (
                    JapaneseLevel::N5,
                    kanjis
                        .iter()
                        .filter(|k: &&KanjiReferenceCard| &k.level == &JapaneseLevel::N5)
                        .cloned()
                        .collect(),
                ),
                (
                    JapaneseLevel::N4,
                    kanjis
                        .iter()
                        .filter(|k: &&KanjiReferenceCard| &k.level == &JapaneseLevel::N4)
                        .cloned()
                        .collect(),
                ),
                (
                    JapaneseLevel::N3,
                    kanjis
                        .iter()
                        .filter(|k: &&KanjiReferenceCard| &k.level == &JapaneseLevel::N3)
                        .cloned()
                        .collect(),
                ),
                (
                    JapaneseLevel::N2,
                    kanjis
                        .iter()
                        .filter(|k: &&KanjiReferenceCard| &k.level == &JapaneseLevel::N2)
                        .cloned()
                        .collect(),
                ),
                (
                    JapaneseLevel::N1,
                    kanjis
                        .iter()
                        .filter(|k: &&KanjiReferenceCard| &k.level == &JapaneseLevel::N1)
                        .cloned()
                        .collect(),
                ),
            ];

            rsx! {
                KanjiContent {
                    levels: levels,
                }
            }
        }
        Some(Err(err)) => {
            rsx! {
                Card { class: Some("p-8 text-center".to_string()),
                    div { class: "text-red-500", "Ошибка загрузки: {err}" }
                }
            }
        }
        None => {
            rsx! {
                div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
            }
        }
    }
}

#[component]
fn KanjiContent(levels: Vec<(JapaneseLevel, Vec<KanjiReferenceCard>)>) -> Element {
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            H2 { "Кандзи справочник" }

            for (level, level_kanjis) in levels {
                KanjiLevelSection {
                    level: level.to_string(),
                    kanjis: level_kanjis,
                }
            }
        }
    }
}

#[component]
fn KanjiLevelSection(level: String, kanjis: Vec<KanjiReferenceCard>) -> Element {
    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-2xl font-bold text-slate-800", "JLPT {level}" }
            if kanjis.is_empty() {
                div { class: "text-slate-500 italic", "Нет кандзи для этого уровня" }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4",
                    for kanji in kanjis {
                        KanjiReferenceCardComponent { kanji_info: kanji.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn KanjiReferenceCardComponent(kanji_info: KanjiReferenceCard) -> Element {
    let added = use_signal(|| false);
    let loading = use_signal(|| false);
    let toast = use_toast();
    let kanji_char = kanji_info.kanji.clone();

    rsx! {
        Card { class: Some("space-y-3".to_string()),
            div { class: "flex justify-between items-start gap-4",
                div { class: "flex-1",
                    div { class: "text-4xl font-bold text-slate-800", "{kanji_char}" }
                    div { class: "text-sm text-slate-600 mt-1", "{kanji_info.description}" }
                }
                Button {
                    variant: if added() { ButtonVariant::Outline } else { ButtonVariant::Primary },
                    disabled: loading() || added(),
                    onclick: move |_| {
                        let kanji = kanji_char.clone();
                        let mut added = added;
                        let mut loading = loading;
                        let toast = toast.clone();
                        loading.set(true);
                        spawn(async move {
                            match create_single_kanji_card(kanji).await {
                                Ok(_) => {
                                    added.set(true);
                                    toast.success("Карточка создана".to_string(), ToastOptions::new());
                                }
                                Err(e) => {
                                    if e.contains("DuplicateCard") {
                                        added.set(true);
                                        toast.info("Карточка уже существует".to_string(), ToastOptions::new());
                                    } else {
                                        toast.error(e, ToastOptions::new());
                                    }
                                }
                            }
                            loading.set(false);
                        });
                    },
                    if added() { "Добавлено" } else { "Добавить" }
                }
            }

            Button {
                variant: ButtonVariant::Ghost,
                onclick: move |_| {
                },
                "Подробнее..."
            }
        }
    }
}

async fn fetch_all_kanjis() -> Result<Vec<KanjiReferenceCard>, String> {
    let mut all_kanjis = Vec::new();
    for level in &[
        JapaneseLevel::N5,
        JapaneseLevel::N4,
        JapaneseLevel::N3,
        JapaneseLevel::N2,
        JapaneseLevel::N1,
    ] {
        let kanjis = KanjiListUseCase::new().execute(level).map_err(to_error)?;
        for kanji in kanjis {
            all_kanjis.push(KanjiReferenceCard {
                kanji: kanji.kanji().to_string(),
                description: kanji.description().to_string(),
                level: *level,
            });
        }
    }
    Ok(all_kanjis)
}

async fn create_single_kanji_card(kanji: String) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    CreateKanjiCardUseCase::new(repo)
        .execute(user_id, vec![kanji])
        .await
        .map_err(to_error)?;

    Ok(())
}
