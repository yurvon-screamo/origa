use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use origa::domain::Card;
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum LessonMode {
    #[default]
    Lesson,
    Fixation,
}

impl LessonMode {
    pub fn title(&self) -> &'static str {
        match self {
            LessonMode::Lesson => "Урок",
            LessonMode::Fixation => "Закрепление",
        }
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct LessonState {
    pub cards: HashMap<Ulid, Card>,
    pub card_ids: Vec<Ulid>,
    pub current_index: usize,
    pub showing_answer: bool,
    pub review_count: usize,
}

#[derive(Clone)]
pub struct LessonContext {
    pub repository: HybridUserRepository,
    pub lesson_state: RwSignal<LessonState>,
    pub is_completed: RwSignal<bool>,
    pub mode: LessonMode,
}
