use dioxus::prelude::*;
use origa::application::{CreateGrammarCardUseCase, GrammarRuleInfoUseCase, UserRepository};
use origa::domain::{GrammarRuleContent, GrammarRuleInfo, JapaneseLevel, get_rule_by_id};
use origa::settings::ApplicationEnvironment;
use std::collections::HashMap;

use crate::components::app_ui::{Card, H2};
use crate::components::button::{Button, ButtonVariant};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus_primitives::toast::{ToastOptions, use_toast};
use ulid::Ulid;

#[derive(Clone, PartialEq)]
struct GrammarRuleForSelection {
    rule_id: Ulid,
    title: String,
    short_description: String,
    md_description: String,
    level: JapaneseLevel,
}

#[component]
pub fn GrammarReference() -> Element {
    let rules_resource = use_resource(fetch_all_rules);

    match rules_resource.read().as_ref() {
        Some(Ok(rules)) => {
            rsx! {
                GrammarReferenceContent { rules: rules.clone() }
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
fn GrammarReferenceContent(rules: Vec<GrammarRuleForSelection>) -> Element {
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            H2 { "Грамматика справочник" }

            for level in &[
                JapaneseLevel::N5,
                JapaneseLevel::N4,
                JapaneseLevel::N3,
                JapaneseLevel::N2,
                JapaneseLevel::N1,
            ]
            {
                GrammarLevelSection {
                    level: level.to_string(),
                    rules: rules.iter().filter(|r| r.level == *level).cloned().collect(),
                }
            }
        }
    }
}

#[component]
fn GrammarLevelSection(level: String, rules: Vec<GrammarRuleForSelection>) -> Element {
    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-2xl font-bold text-slate-800 mb-4", "JLPT {level}" }
            if rules.is_empty() {
                div { class: "text-slate-500 italic",
                    "Нет правил для этого уровня"
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    for rule in rules {
                        GrammarReferenceCard { rule }
                    }
                }
            }
        }
    }
}

#[component]
fn GrammarReferenceCard(rule: GrammarRuleForSelection) -> Element {
    let added = use_signal(|| false);
    let loading = use_signal(|| false);
    let toast = use_toast();

    rsx! {
        Card { class: Some("space-y-3".to_string()),
            div { class: "flex justify-between items-start gap-4",
                div { class: "flex-1",
                    h4 { class: "text-lg font-semibold text-slate-800", "{rule.title}" }
                    p { class: "text-sm text-slate-600 mt-1", "{rule.short_description}" }
                }
                Button {
                    variant: if added() { ButtonVariant::Outline } else { ButtonVariant::Primary },
                    disabled: loading() || added(),
                    onclick: move |_| {
                        let rule_id = rule.rule_id;
                        let mut added = added;
                        let mut loading = loading;
                        let toast = toast;
                        loading.set(true);
                        spawn(async move {
                            match create_single_grammar_card(rule_id).await {
                                Ok(_) => {
                                    added.set(true);
                                    toast
                                        .success(
                                            "Карточка создана".to_string(),
                                            ToastOptions::new(),
                                        );
                                }
                                Err(e) => {
                                    if e.contains("DuplicateCard") {
                                        added.set(true);
                                        toast
                                            .info(
                                                "Карточка уже существует".to_string(),
                                                ToastOptions::new(),
                                            );
                                    } else {
                                        toast.error(e, ToastOptions::new());
                                    }
                                }
                            }
                            loading.set(false);
                        });
                    },
                    if added() {
                        "Добавлено"
                    } else {
                        "Добавить"
                    }
                }
            }

            if added() {
                div { class: "mt-4 p-4 bg-slate-50 rounded-lg border border-slate-200",
                    h5 { class: "text-sm font-semibold text-slate-700 mb-2",
                        "Подробное описание"
                    }
                    div { class: "text-sm text-slate-600 prose prose-sm max-w-none",
                        "{rule.md_description}"
                    }
                }
            } else {
                Button { variant: ButtonVariant::Ghost, onclick: move |_| {}, "Подробнее..." }
            }
        }
    }
}

async fn fetch_all_rules() -> Result<Vec<GrammarRuleForSelection>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let mut all_rules = Vec::new();

    for level in &[
        JapaneseLevel::N5,
        JapaneseLevel::N4,
        JapaneseLevel::N3,
        JapaneseLevel::N2,
        JapaneseLevel::N1,
    ] {
        let rules = GrammarRuleInfoUseCase::new(repo)
            .execute(user_id, level)
            .await
            .map_err(to_error)?;

        for rule in rules {
            all_rules.push(GrammarRuleForSelection {
                rule_id: rule.rule_id,
                title: rule.title,
                short_description: rule.short_description,
                md_description: rule.md_description,
                level: *level,
            });
        }
    }

    Ok(all_rules)
}

async fn create_single_grammar_card(rule_id: Ulid) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let user: origa::domain::User = repo
        .find_by_id(user_id)
        .await
        .map_err(to_error)?
        .ok_or_else(|| "User not found".to_string())?;

    let lang = user.native_language();
    let mut grammar_rules: Vec<GrammarRuleInfo> = Vec::new();

    if let Some(grammar_rule) = get_rule_by_id(&rule_id) {
        let info = grammar_rule.info();
        let content = info.content(lang);

        let mut content_map = HashMap::new();
        content_map.insert(
            lang.clone(),
            GrammarRuleContent::new(
                content.title().to_string(),
                content.short_description().to_string(),
                content.md_description().to_string(),
            ),
        );

        grammar_rules.push(GrammarRuleInfo::new(
            *info.rule_id(),
            *info.level(),
            info.apply_to().to_vec(),
            content_map,
        ));
    }

    CreateGrammarCardUseCase::new(repo)
        .execute(user_id, grammar_rules)
        .await
        .map_err(to_error)?;

    Ok(())
}
