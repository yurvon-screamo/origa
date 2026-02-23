use crate::repository::SupabaseUserRepository;
use leptos::prelude::*;
use origa::domain::Card;
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Clone, PartialEq, Default)]
pub struct LessonState {
    pub cards: HashMap<Ulid, Card>,
    pub card_ids: Vec<Ulid>,
    pub current_index: usize,
    pub showing_answer: bool,
    pub new_count: usize,
    pub review_count: usize,
}

#[derive(Clone)]
pub struct LessonContext {
    pub repository: SupabaseUserRepository,
    pub lesson_state: RwSignal<LessonState>,
    pub is_completed: RwSignal<bool>,
}
