use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::dictionary::grammar::{GrammarRule, iter_grammar_rules};
use origa::domain::{Card, JapaneseLevel, NativeLanguage};
use origa::traits::UserRepository;
use std::collections::HashSet;
use ulid::Ulid;

#[derive(Clone)]
pub struct ModalState {
    pub selected_level: RwSignal<JapaneseLevel>,
    pub available_rules: RwSignal<Vec<&'static GrammarRule>>,
    pub native_language: RwSignal<NativeLanguage>,
    pub selected_rule_ids: RwSignal<HashSet<Ulid>>,
    pub is_loading_rules: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub search_query: RwSignal<String>,
    pub refresh_trigger: RwSignal<u32>,
}

impl ModalState {
    pub fn new(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let selected_rule_ids = RwSignal::new(HashSet::new());

        Effect::new({
            let selected_rule_ids_clone = selected_rule_ids;
            move |_| {
                if is_open.get() {
                    selected_rule_ids_clone.set(HashSet::new());
                }
            }
        });

        Self {
            selected_level: RwSignal::new(JapaneseLevel::N5),
            available_rules: RwSignal::new(Vec::new()),
            native_language: RwSignal::new(NativeLanguage::Russian),
            selected_rule_ids,
            is_loading_rules: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            search_query: RwSignal::new(String::new()),
            refresh_trigger,
        }
    }

    pub fn load_rules(&self) {
        let level = self.selected_level.get();
        let repository = self.repository.clone();
        let available_rules = self.available_rules;
        let native_language = self.native_language;
        let is_loading = self.is_loading_rules;
        let error = self.error_message;
        let disposed = StoredValue::new(());

        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            match repository.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let lang = *user.native_language();
                    native_language.set(lang);

                    let existing_rule_ids: HashSet<Ulid> = user
                        .knowledge_set()
                        .study_cards()
                        .values()
                        .filter_map(|card| match card.card() {
                            Card::Grammar(rule) => Some(*rule.rule_id()),
                            _ => None,
                        })
                        .collect();

                    let rules: Vec<&'static GrammarRule> = iter_grammar_rules()
                        .filter(|rule| rule.level() == &level)
                        .filter(|rule| !existing_rule_ids.contains(rule.rule_id()))
                        .collect();
                    available_rules.set(rules);
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some("Пользователь не найден".to_string()));
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some(format!("Ошибка загрузки пользователя: {}", e)));
                },
            }
            if disposed.is_disposed() {
                return;
            }
            is_loading.set(false);
        });
    }

    pub fn select_level(&self, level: JapaneseLevel) {
        self.selected_level.set(level);
        self.selected_rule_ids.set(HashSet::new());
        self.load_rules();
    }

    pub fn reset(&self) {
        self.error_message.set(None);
        self.selected_rule_ids.set(HashSet::new());
    }

    pub fn select_all(&self) {
        let query = self.search_query.get().to_lowercase();
        let rules = self.available_rules.get();
        let lang = self.native_language.get();
        let filtered_ids: HashSet<Ulid> = rules
            .iter()
            .filter(|rule| {
                let content = rule.content(&lang);
                query.is_empty()
                    || content.title().to_lowercase().contains(&query)
                    || content.short_description().to_lowercase().contains(&query)
            })
            .map(|rule| *rule.rule_id())
            .collect();
        self.selected_rule_ids.set(filtered_ids);
    }
}
