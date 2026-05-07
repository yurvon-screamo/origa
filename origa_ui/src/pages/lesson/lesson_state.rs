use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use origa::domain::{LessonCard, NativeLanguage, Rating};
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

#[derive(Clone, PartialEq, Default)]
pub struct LessonState {
    pub cards: HashMap<Ulid, LessonCard>,
    pub card_ids: Vec<Ulid>,
    pub current_index: usize,
    pub showing_answer: bool,
    pub review_count: usize,
    pub selected_quiz_option: Option<usize>,
    pub selected_yesno_answer: Option<bool>,
    pub dont_know_selected: bool,
    pub core_count: usize,
    pub waiting_for_next: bool,
    pub pending_rating: Option<Rating>,
}

#[derive(Clone)]
pub struct LessonContext {
    pub repository: HybridUserRepository,
    pub lesson_state: RwSignal<LessonState>,
    pub is_completed: RwSignal<bool>,
    pub reload_trigger: RwSignal<u32>,
    pub is_muted: RwSignal<bool>,
    pub known_kanji: RwSignal<HashSet<String>>,
    pub native_language: RwSignal<NativeLanguage>,
    pub core_count: RwSignal<usize>,
}
