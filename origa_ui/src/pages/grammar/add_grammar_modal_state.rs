use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, JapaneseLevel, User};
use origa::use_cases::{GrammarRuleInfoUseCase, GrammarRuleItem};
use std::collections::HashSet;
use ulid::Ulid;

#[derive(Clone)]
pub struct ModalState {
    pub selected_level: RwSignal<JapaneseLevel>,
    pub available_rules: RwSignal<Vec<GrammarRuleItem>>,
    pub selected_rule_ids: RwSignal<HashSet<Ulid>>,
    pub is_loading_rules: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub current_user: RwSignal<Option<User>>,
    pub repository: HybridUserRepository,
    pub search_query: RwSignal<String>,
}

impl ModalState {
    pub fn new(is_open: RwSignal<bool>) -> Self {
        let current_user =
            use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
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
            selected_rule_ids,
            is_loading_rules: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            current_user,
            repository,
            search_query: RwSignal::new(String::new()),
        }
    }

    pub fn load_rules(&self) {
        let user_id = self
            .current_user
            .with(|u| u.as_ref().map(|u| u.id()))
            .unwrap();
        let existing_rule_ids: HashSet<Ulid> = self.current_user.with(|u| {
            u.as_ref()
                .map(|u| {
                    u.knowledge_set()
                        .study_cards()
                        .values()
                        .filter_map(|sc| match sc.card() {
                            Card::Grammar(g) => Some(*g.rule_id()),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default()
        });
        let level = self.selected_level.get();
        let repository = self.repository.clone();
        let available_rules = self.available_rules;
        let is_loading = self.is_loading_rules;
        let error = self.error_message;

        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            let use_case = GrammarRuleInfoUseCase::new(&repository);
            match use_case.execute(user_id, &level, &existing_rule_ids).await {
                Ok(rules) => {
                    available_rules.set(rules);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                }
            }
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
        let filtered_ids: HashSet<Ulid> = rules
            .iter()
            .filter(|rule| {
                query.is_empty()
                    || rule.title.to_lowercase().contains(&query)
                    || rule.short_description.to_lowercase().contains(&query)
            })
            .map(|rule| rule.rule_id)
            .collect();
        self.selected_rule_ids.set(filtered_ids);
    }
}
