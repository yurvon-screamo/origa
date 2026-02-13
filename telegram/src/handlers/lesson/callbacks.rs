use origa::domain::Rating;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LessonCallback {
    #[serde(rename = "rating")]
    Rating { rating: Rating },

    #[serde(rename = "next_card")]
    NextCard,

    #[serde(rename = "back_to_main")]
    BackToMain,
}

impl LessonCallback {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }

    pub const NEXT_CARD: &str = "Далее ➡️";
    pub const BACK_TO_MAIN: &str = "🏠 На главную";
    pub const RATING_AGAIN: &str = "Не знаю ❌";
    pub const RATING_HARD: &str = "Плохо 😐";
    pub const RATING_GOOD: &str = "Знаю ✅";
    pub const RATING_EASY: &str = "Идеально 🌟";
    pub const LESSON_COMPLETE: &str = "🎉 Урок завершён!";
    pub const FIXATION_COMPLETE: &str = "🎉 Закрепление завершено!";
    pub const LESSON_STARTED: &str = "🎯 Урок начат";
    pub const FIXATION_STARTED: &str = "🔒 Закрепление начато";
    pub const CARDS: &str = "Карточек";
    pub const PROGRESS: &str = "Прогресс";
    pub const NO_CARDS: &str =
        "Нет карточек для урока. Добавьте новые слова или подождите повторения.";
    pub const NO_FIXATION_CARDS: &str = "Нет сложных карточек для закрепления.";
    pub const CARD: &str = "Карточка";
    pub const NEW: &str = "Новых";
    pub const REVIEWED: &str = "Повторено";
    pub const TRANSLATION: &str = "Перевод";
    pub const MEANINGS: &str = "Значения";
    pub const BRIEFLY: &str = "Кратко";
}
