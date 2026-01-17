use crate::components::button::{Button, ButtonVariant};
use crate::components::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};
use crate::components::sheet::{
    Sheet, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus::prelude::*;
use origa::application::use_cases::grammar_info::GrammarRuleItem;
use origa::application::{CreateGrammarCardUseCase, GrammarRuleInfoUseCase, UserRepository};
use origa::domain::{GrammarRuleContent, GrammarRuleInfo, JapaneseLevel, get_rule_by_id};
use origa::settings::ApplicationEnvironment;
use std::collections::HashMap;
use ulid::Ulid;

#[component]
pub fn GrammarCreateModal(
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
    on_error: EventHandler<String>,
    loading: bool,
) -> Element {
    let mut selected_level = use_signal(|| JapaneseLevel::N5);
    let selected_rules = use_signal(Vec::<Ulid>::new);
    let rules_resource = use_resource(move || fetch_rules(selected_level()));

    rsx! {
        Sheet {
            open: true,
            on_open_change: move |v: bool| {
                if !v {
                    on_close.call(())
                }
            },
            SheetContent { side: SheetSide::Right,
                SheetHeader {
                    SheetTitle { "Создать карточки грамматики" }
                }

                div { class: "space-y-4",
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Уровень JLPT" }
                        Select::<JapaneseLevel> {
                            value: Some(Some(selected_level())),
                            on_value_change: move |v: Option<JapaneseLevel>| {
                                if let Some(level) = v {
                                    selected_level.set(level);
                                }
                            },
                            placeholder: "Выберите уровень...",
                            SelectTrigger {
                                aria_label: "Уровень JLPT",
                                width: "100%",
                                SelectValue {}
                            }
                            SelectList { aria_label: "Уровень JLPT",
                                SelectOption::<JapaneseLevel> { index: 0usize, value: JapaneseLevel::N5,
                                    "N5"
                                    SelectItemIndicator {}
                                }
                                SelectOption::<JapaneseLevel> { index: 1usize, value: JapaneseLevel::N4,
                                    "N4"
                                    SelectItemIndicator {}
                                }
                                SelectOption::<JapaneseLevel> { index: 2usize, value: JapaneseLevel::N3,
                                    "N3"
                                    SelectItemIndicator {}
                                }
                                SelectOption::<JapaneseLevel> { index: 3usize, value: JapaneseLevel::N2,
                                    "N2"
                                    SelectItemIndicator {}
                                }
                                SelectOption::<JapaneseLevel> { index: 4usize, value: JapaneseLevel::N1,
                                    "N1"
                                    SelectItemIndicator {}
                                }
                            }
                        }
                    }

                    match rules_resource.read().as_ref() {
                        Some(Ok(rules)) => {
                            if rules.is_empty() {
                                rsx! {
                                    div { class: "text-center py-8 text-slate-500",
                                        "Нет правил для выбранного уровня"
                                    }
                                }
                            } else {
                                rsx! {
                                    div { class: "space-y-2",
                                        label { class: "text-sm font-medium", "Выберите правила" }
                                        div { class: "grid grid-cols-1 gap-2 max-h-[400px] overflow-y-auto",
                                            for rule in rules.iter() {
                                                GrammarRuleCheckbox {
                                                    rule: rule.clone(),
                                                    selected: selected_rules().contains(&rule.rule_id),
                                                    on_toggle: {
                                                        let rule_id = rule.rule_id;
                                                        let mut selected_rules = selected_rules;
                                                        move |_| {
                                                            let mut rules = selected_rules();
                                                            if rules.contains(&rule_id) {
                                                                rules.retain(|id| id != &rule_id);
                                                            } else {
                                                                rules.push(rule_id);
                                                            }
                                                            selected_rules.set(rules);
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(err)) => {
                            rsx! {
                                div { class: "text-center py-8 text-red-500", "Ошибка: {err}" }
                            }
                        }
                        None => {
                            rsx! {
                                div { class: "text-center py-8 text-slate-500", "Загрузка правил..." }
                            }
                        }
                    }
                }

                SheetFooter {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| on_close.call(()),
                        "Отмена"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: loading || selected_rules().is_empty(),
                        onclick: move |_| {
                            let on_success = on_success;
                            let on_error = on_error;
                            let rule_ids = selected_rules().clone();
                            spawn(async move {
                                match create_grammar_cards(rule_ids).await {
                                    Ok(count) => {
                                        on_success
                                            .call(format!("Создано {} карточек", count));
                                    }
                                    Err(e) => {
                                        on_error.call(format!("Ошибка: {}", e));
                                    }
                                }
                            });
                        },
                        {
                            if loading {
                                "Создание...".to_string()
                            } else {
                                format!("Создать ({} шт)", selected_rules().len())
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GrammarRuleCheckbox(
    rule: GrammarRuleItem,
    selected: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    rsx! {
        label { class: "flex items-start gap-3 p-3 bg-white rounded-lg border border-slate-200 hover:bg-slate-50 cursor-pointer",
            input {
                r#type: "checkbox",
                class: "mt-1",
                checked: selected,
                onchange: move |_| on_toggle.call(()),
            }
            div { class: "flex-1",
                div { class: "font-medium text-slate-800", "{rule.title}" }
                div { class: "text-sm text-slate-600", "{rule.short_description}" }
            }
        }
    }
}

async fn fetch_rules(level: JapaneseLevel) -> Result<Vec<GrammarRuleItem>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    GrammarRuleInfoUseCase::new(repo)
        .execute(user_id, &level)
        .await
        .map_err(to_error)
}

async fn create_grammar_cards(rule_ids: Vec<Ulid>) -> Result<usize, String> {
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

    for rule_id in rule_ids {
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
    }

    let cards = CreateGrammarCardUseCase::new(repo)
        .execute(user_id, grammar_rules)
        .await
        .map_err(to_error)?;

    Ok(cards.len())
}
