use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use chrono::{DateTime, Utc};
use rand::seq::SliceRandom;
use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, CardType, KnowledgeSet, StudyCard};
use crate::domain::{JapaneseLevel, JlptContent, NativeLanguage};

const MIN_LESSON_SIZE: usize = 15;

/// Максимум карт одного урока. Публична: размер урока — доменное
/// правило, его читает и отбор руки знакомства (влезает ли рука
/// поверх due-хвоста).
pub const MAX_LESSON_SIZE: usize = 22;

// Разбиение билдера урока на этапы пайплайна; фасад (паблик-функции ниже)
// сохраняет прежние пути `lesson_builder::...` для внешних callers.
// Дубль-механизм (multi-show expansion) удалён: закрепление провалов
// переехало в добивания (docs/plans/ghost-relearning.md §3.5).
mod drop_new_cards;
mod interleave;
mod phrases;
mod policy;
mod selection;
mod slots;
mod spacing;

pub(crate) use drop_new_cards::drop_new_cards;
pub(crate) use interleave::interleave_core_by_type;
pub(crate) use phrases::add_phrases;
pub use policy::NewCardPolicy;
pub(crate) use policy::excluded_by_new_card_policy;
pub(crate) use selection::build_lesson_core;
pub(crate) use slots::{distribute_new_cards, jlpt_sort_key};
pub(crate) use spacing::redistribute_core_for_spacing;

#[cfg(test)]
mod tests;
