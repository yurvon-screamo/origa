use std::collections::{HashMap, HashSet};

use origa::domain::JapaneseLevel;
use origa::traits::WellKnownSetMeta;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Intro,
    JLPT,
    Apps,
    Progress,
    Summary,
}

impl OnboardingStep {
    pub fn as_usize(&self) -> usize {
        match self {
            OnboardingStep::Intro => 0,
            OnboardingStep::JLPT => 1,
            OnboardingStep::Apps => 2,
            OnboardingStep::Progress => 3,
            OnboardingStep::Summary => 4,
        }
    }

    #[allow(dead_code)]
    pub fn from_usize(value: usize) -> Self {
        match value {
            0 => OnboardingStep::Intro,
            1 => OnboardingStep::JLPT,
            2 => OnboardingStep::Apps,
            3 => OnboardingStep::Progress,
            _ => OnboardingStep::Summary,
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            OnboardingStep::Intro => "Добро пожаловать",
            OnboardingStep::JLPT => "Уровень",
            OnboardingStep::Apps => "Приложения",
            OnboardingStep::Progress => "Прогресс",
            OnboardingStep::Summary => "Итог",
        }
    }

    pub fn is_first(&self) -> bool {
        matches!(self, OnboardingStep::Intro)
    }

    pub fn is_last(&self) -> bool {
        matches!(self, OnboardingStep::Summary)
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            OnboardingStep::Intro => Some(OnboardingStep::JLPT),
            OnboardingStep::JLPT => Some(OnboardingStep::Apps),
            OnboardingStep::Apps => Some(OnboardingStep::Progress),
            OnboardingStep::Progress => Some(OnboardingStep::Summary),
            OnboardingStep::Summary => None,
        }
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            OnboardingStep::Intro => None,
            OnboardingStep::JLPT => Some(OnboardingStep::Intro),
            OnboardingStep::Apps => Some(OnboardingStep::JLPT),
            OnboardingStep::Progress => Some(OnboardingStep::Apps),
            OnboardingStep::Summary => Some(OnboardingStep::Progress),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OnboardingState {
    pub current_step: OnboardingStep,
    pub selected_level: Option<JapaneseLevel>,
    pub selected_apps: HashSet<String>,
    pub apps_progress: HashMap<String, String>,
    pub sets_to_import: Vec<WellKnownSetMeta>,
    pub excluded_sets: HashSet<String>,
    pub available_sets: Vec<WellKnownSetMeta>,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            current_step: OnboardingStep::Intro,
            selected_level: None,
            selected_apps: HashSet::new(),
            apps_progress: HashMap::new(),
            sets_to_import: Vec::new(),
            excluded_sets: HashSet::new(),
            available_sets: Vec::new(),
        }
    }
}

impl OnboardingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_app(&mut self, app_id: &str) {
        self.selected_apps.insert(app_id.to_string());
    }

    pub fn remove_app(&mut self, app_id: &str) {
        self.selected_apps.remove(app_id);
    }

    pub fn set_app_selection(&mut self, app_id: &str, set_id: &str) {
        self.apps_progress
            .insert(app_id.to_string(), set_id.to_string());
    }

    pub fn add_set_to_import(&mut self, set_meta: WellKnownSetMeta) {
        let set_id = set_meta.id.clone();
        if !self.excluded_sets.contains(&set_id)
            && !self.sets_to_import.iter().any(|s| s.id == set_id)
        {
            self.sets_to_import.push(set_meta);
        }
    }

    pub fn remove_set_from_import(&mut self, set_id: &str) {
        self.sets_to_import.retain(|s| s.id != set_id);
        self.excluded_sets.insert(set_id.to_string());
    }

    pub fn reset_exclusion(&mut self, set_id: &str) {
        self.excluded_sets.remove(set_id);
    }

    pub fn go_to_next_step(&mut self) -> bool {
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            true
        } else {
            false
        }
    }

    pub fn go_to_prev_step(&mut self) -> bool {
        if let Some(prev) = self.current_step.prev() {
            self.current_step = prev;
            true
        } else {
            false
        }
    }

    pub fn set_available_sets(&mut self, sets: Vec<WellKnownSetMeta>) {
        self.available_sets = sets;
    }

    pub fn get_final_sets(&self) -> Vec<String> {
        self.sets_to_import.iter().map(|s| s.id.clone()).collect()
    }

    pub fn is_first_step(&self) -> bool {
        self.current_step.is_first()
    }

    pub fn is_last_step(&self) -> bool {
        self.current_step.is_last()
    }

    pub fn can_proceed(&self) -> bool {
        match self.current_step {
            OnboardingStep::Intro => true,
            OnboardingStep::JLPT => self.selected_level.is_some(),
            OnboardingStep::Apps => true,
            OnboardingStep::Progress => true,
            OnboardingStep::Summary => !self.sets_to_import.is_empty(),
        }
    }

    fn is_jlpt_set_id(id: &str) -> bool {
        id.starts_with("jlpt_n")
    }

    pub fn clear_previous_jlpt_selections(&mut self) {
        self.sets_to_import.retain(|s| !Self::is_jlpt_set_id(&s.id));
    }

    pub fn set_jlpt_level(&mut self, level: Option<JapaneseLevel>) {
        self.selected_level = level;
        self.clear_previous_jlpt_selections();

        if let Some(_) = level {
            if let Some(jlpt_n5_set) = self.available_sets.iter().find(|s| s.id == "jlpt_n5") {
                self.sets_to_import.push(jlpt_n5_set.clone());
            }
        }
    }
}
