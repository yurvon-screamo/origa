use dioxus::prelude::*;
use dioxus_heroicons::{IconButton, solid::Shape};
use dioxus_logger::tracing;
use origa::application::{CreateKanjiCardUseCase, KanjiListUseCase, KnowledgeSetCardsUseCase};
use origa::domain::{Card, JapaneseLevel, KanjiInfo};

use crate::components::app_ui::Card as UiCard;
use crate::components::sheet::{Sheet, SheetContent, SheetHeader, SheetSide, SheetTitle};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus_primitives::toast::{ToastOptions, use_toast};
use origa::settings::ApplicationEnvironment;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
struct KanjiReferenceCard {
    kanji: String,
    info: KanjiInfo,
    added: bool,
}

async fn fetch_user_kanji_set() -> Result<HashSet<String>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let cards = KnowledgeSetCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)?;

    tracing::debug!("User has {} cards total", cards.len());

    let kanji_set: HashSet<String> = cards
        .into_iter()
        .filter_map(|card| match card.card() {
            Card::Kanji(kanji_card) => {
                let kanji_str = kanji_card.kanji().text().to_string();
                tracing::debug!("Found kanji card: '{}'", kanji_str);
                Some(kanji_str)
            }
            _ => None,
        })
        .collect();

    tracing::debug!("User has {} kanji cards", kanji_set.len());
    Ok(kanji_set)
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
            let kanji_str = kanji.kanji().to_string();
            tracing::debug!(
                "Dictionary kanji: '{}' (char: {})",
                kanji_str,
                kanji.kanji()
            );
            all_kanjis.push(KanjiReferenceCard {
                kanji: kanji_str,
                info: kanji,
                added: false,
            });
        }
    }
    tracing::debug!("Total kanji in dictionary: {}", all_kanjis.len());
    Ok(all_kanjis)
}

async fn create_single_kanji_card(kanji: String) -> Result<(), String> {
    tracing::debug!("Attempting to create kanji card for: '{}'", kanji);
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    match CreateKanjiCardUseCase::new(repo)
        .execute(user_id, vec![kanji.clone()])
        .await
    {
        Ok(cards) => {
            tracing::debug!("Successfully created {} kanji cards", cards.len());
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to create kanji card '{}': {}", kanji, e);
            Err(e.to_string())
        }
    }
}

#[component]
pub fn Kanji() -> Element {
    let kanjis_resource = use_resource(fetch_all_kanjis);
    let user_kanjis = use_resource(fetch_user_kanji_set);
    let _toast = use_toast();

    match (kanjis_resource.read().as_ref(), user_kanjis.read().as_ref()) {
        (Some(Ok(kanjis)), Some(Ok(user_kanji_set))) => {
            let levels: Vec<(JapaneseLevel, Vec<KanjiReferenceCard>)> = vec![
                (
                    JapaneseLevel::N5,
                    kanjis
                        .iter()
                        .filter(|k| k.info.jlpt() == &JapaneseLevel::N5)
                        .map(|k| KanjiReferenceCard {
                            kanji: k.kanji.clone(),
                            info: k.info.clone(),
                            added: user_kanji_set.contains(&k.kanji),
                        })
                        .collect(),
                ),
                (
                    JapaneseLevel::N4,
                    kanjis
                        .iter()
                        .filter(|k| k.info.jlpt() == &JapaneseLevel::N4)
                        .map(|k| KanjiReferenceCard {
                            kanji: k.kanji.clone(),
                            info: k.info.clone(),
                            added: user_kanji_set.contains(&k.kanji),
                        })
                        .collect(),
                ),
                (
                    JapaneseLevel::N3,
                    kanjis
                        .iter()
                        .filter(|k| k.info.jlpt() == &JapaneseLevel::N3)
                        .map(|k| KanjiReferenceCard {
                            kanji: k.kanji.clone(),
                            info: k.info.clone(),
                            added: user_kanji_set.contains(&k.kanji),
                        })
                        .collect(),
                ),
                (
                    JapaneseLevel::N2,
                    kanjis
                        .iter()
                        .filter(|k| k.info.jlpt() == &JapaneseLevel::N2)
                        .map(|k| KanjiReferenceCard {
                            kanji: k.kanji.clone(),
                            info: k.info.clone(),
                            added: user_kanji_set.contains(&k.kanji),
                        })
                        .collect(),
                ),
                (
                    JapaneseLevel::N1,
                    kanjis
                        .iter()
                        .filter(|k| k.info.jlpt() == &JapaneseLevel::N1)
                        .map(|k| KanjiReferenceCard {
                            kanji: k.kanji.clone(),
                            info: k.info.clone(),
                            added: user_kanji_set.contains(&k.kanji),
                        })
                        .collect(),
                ),
            ];

            rsx! {
                KanjiContent { levels }
            }
        }
        (Some(Err(err)), _) | (_, Some(Err(err))) => {
            rsx! {
                UiCard { class: Some("p-8 text-center".to_string()),
                    div { class: "text-red-500", "Ошибка загрузки: {err}" }
                }
            }
        }
        _ => {
            rsx! {
                div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
            }
        }
    }
}

#[component]
fn KanjiContent(levels: Vec<(JapaneseLevel, Vec<KanjiReferenceCard>)>) -> Element {
    let mut selected_kanji = use_signal(|| None::<KanjiReferenceCard>);
    let refresh_trigger = use_signal(|| 0i32);

    let user_kanjis = use_resource(move || async move {
        let _ = refresh_trigger();
        fetch_user_kanji_set().await
    });

    let updated_levels = if let Some(Ok(user_kanji_set)) = user_kanjis.read().as_ref() {
        levels
            .into_iter()
            .map(|(level, kanjis)| {
                (
                    level,
                    kanjis
                        .into_iter()
                        .map(|k| {
                            let kanji_str = k.kanji.clone();
                            KanjiReferenceCard {
                                kanji: kanji_str,
                                info: k.info,
                                added: user_kanji_set.contains(&k.kanji),
                            }
                        })
                        .collect(),
                )
            })
            .collect()
    } else {
        levels
    };

    match selected_kanji.read().as_ref() {
        Some(kanji) => {
            let user_kanji_set = user_kanjis
                .read()
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .cloned()
                .unwrap_or_default();
            let updated_kanji = KanjiReferenceCard {
                kanji: kanji.kanji.clone(),
                info: kanji.info.clone(),
                added: user_kanji_set.contains(&kanji.kanji),
            };
            rsx! {
                div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
                    for (level , level_kanjis) in updated_levels {
                        KanjiLevelSection {
                            level: level.to_string(),
                            kanjis: level_kanjis,
                            on_select: move |k| selected_kanji.set(Some(k)),
                        }
                    }
                }

                KanjiDetailDrawer {
                    kanji: updated_kanji,
                    on_close: move || selected_kanji.set(None),
                }
            }
        }
        _ => {
            rsx! {
                div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
                    for (level , level_kanjis) in updated_levels {
                        KanjiLevelSection {
                            level: level.to_string(),
                            kanjis: level_kanjis,
                            on_select: move |k| selected_kanji.set(Some(k)),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn KanjiLevelSection(
    level: String,
    kanjis: Vec<KanjiReferenceCard>,
    on_select: EventHandler<KanjiReferenceCard>,
) -> Element {
    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-2xl font-bold text-slate-800", "JLPT {level}" }
            if kanjis.is_empty() {
                div { class: "text-slate-500 italic",
                    "Нет кандзи для этого уровня"
                }
            } else {
                div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-3",
                    for kanji in kanjis {
                        KanjiCardCompact {
                            kanji_info: kanji.clone(),
                            on_click: move || on_select.call(kanji.clone()),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn KanjiCardCompact(kanji_info: KanjiReferenceCard, on_click: EventHandler<()>) -> Element {
    rsx! {
        div {
            onclick: move |_| on_click.call(()),
            class: "p-3 cursor-pointer hover:bg-slate-50 transition-colors rounded-lg border border-slate-200",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    div { class: "text-3xl font-bold text-slate-800", "{kanji_info.kanji}" }
                    if kanji_info.added {
                        IconButton {
                            class: "p-0.5 text-green-600".to_string(),
                            icon: Shape::CheckCircle,
                        }
                    }
                }
            }
            div { class: "text-sm text-slate-600 truncate", "{kanji_info.info.description()}" }
        }
    }
}

#[component]
fn KanjiDetailDrawer(kanji: KanjiReferenceCard, on_close: EventHandler<()>) -> Element {
    let kanji_char = kanji.kanji.clone();
    let kanji_char_for_display = kanji.kanji.clone();
    let added = use_signal(|| kanji.added);
    let loading = use_signal(|| false);
    let toast = use_toast();
    let info = kanji.info.clone();

    let handle_add = move |_| {
        let kanji_clone = kanji_char.clone();
        let mut added = added;
        let mut loading = loading;
        let toast = toast;

        loading.set(true);
        spawn(async move {
            match create_single_kanji_card(kanji_clone).await {
                Ok(_) => {
                    added.set(true);
                    toast.success("Добавлено".to_string(), ToastOptions::new());
                }
                Err(e) => {
                    if e.contains("DuplicateCard") {
                        added.set(true);
                        toast.info("Уже есть".to_string(), ToastOptions::new());
                    } else {
                        toast.error(e, ToastOptions::new());
                    }
                }
            }
            loading.set(false);
        });
    };

    let radicals_info: Vec<_> = info.radicals();
    let popular_words_list: Vec<_> = info.popular_words().to_vec();

    rsx! {
        Sheet {
            open: true,
            on_open_change: move |v: bool| {
                if !v {
                    on_close.call(())
                }
            },
            SheetContent { side: SheetSide::Right, class: Some("h-full".to_string()),
                div { class: "h-full flex flex-col overflow-hidden",
                    SheetHeader {
                        SheetTitle { id: "kanji-detail-title",
                            div { class: "flex items-center gap-3",
                                div { class: "text-5xl font-bold text-slate-800",
                                    "{kanji_char_for_display}"
                                }
                                if *added.read() {
                                    IconButton {
                                        class: "p-1 text-green-600".to_string(),
                                        icon: Shape::CheckCircle,
                                    }
                                }
                            }
                        }
                    }

                    div { class: "flex-1 overflow-y-auto p-6 space-y-6",
                        div { class: "space-y-2",
                            div { class: "text-sm font-medium text-slate-500", "Значение" }
                            div { class: "text-lg text-slate-800", "{info.description()}" }
                        }

                        if !radicals_info.is_empty() {
                            div { class: "space-y-2",
                                div { class: "text-sm font-medium text-slate-500", "Радикалы" }
                                div { class: "flex flex-wrap gap-2",
                                    for radical in radicals_info {
                                        div { class: "flex items-center gap-2 px-3 py-1.5 bg-amber-50 text-amber-700 rounded-lg",
                                            div { class: "text-lg font-bold", "{radical.radical()}" }
                                            div { class: "text-sm text-amber-600", "{radical.name()}" }
                                        }
                                    }
                                }
                            }
                        }

                        if !popular_words_list.is_empty() {
                            div { class: "space-y-2",
                                div { class: "text-sm font-medium text-slate-500", "Слова" }
                                div { class: "space-y-1",
                                    for word in popular_words_list {
                                        div { class: "text-slate-700", "{word}" }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "p-6 border-t border-slate-200",
                        if !*added.read() {
                            button {
                                disabled: loading(),
                                onclick: handle_add,
                                class: "w-full px-4 py-2.5 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed",
                                if loading() {
                                    "Добавление..."
                                } else {
                                    "Добавить в карточки"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
