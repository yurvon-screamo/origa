use origa::domain::Rating;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LessonCallback {
    #[serde(rename = "rating")]
    Rating { rating: Rating },

    #[serde(rename = "next_card")]
    NextCard,

    #[serde(rename = "abort_lesson")]
    AbortLesson,

    #[serde(rename = "back_to_main")]
    BackToMain,
}

impl LessonCallback {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize callback data")
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }

    pub fn rating_button_text(rating: Rating) -> &'static str {
        match rating {
            Rating::Again => "Не знаю ❌",
            Rating::Hard => "Плохо 😐",
            Rating::Good => "Знаю ✅",
            Rating::Easy => "Идеально 🌟",
        }
    }

    pub const NEXT_CARD: &str = "Далее ➡️";
    pub const ABORT_LESSON: &str = "Прервать";
    pub const BACK_TO_MAIN: &str = "🏠 На главную";
    pub const LESSON_COMPLETE: &str = "🎉 Урок завершён!";
    pub const FIXATION_COMPLETE: &str = "🎉 Закрепление завершено!";
    pub const LESSON_STARTED: &str = "🎯 Урок начат";
    pub const FIXATION_STARTED: &str = "🔒 Закрепление начато";
    pub const CARDS: &str = "Карточек";
    pub const PROGRESS: &str = "Прогресс";
    pub const NO_CARDS: &str =
        "Нет карточек для урока. Добавьте новые слова или подождите повторения.";
    pub const NO_FIXATION_CARDS: &str = "Нет сложных карточек для закрепления.";
    pub const LESSON_ABORTED: &str = "Урок прерван.";
    pub const CARD: &str = "Карточка";
    pub const NEW: &str = "Новых";
    pub const REVIEWED: &str = "Повторено";
    pub const TRANSLATION: &str = "Перевод";
    pub const EXAMPLES: &str = "Примеры";
    pub const MEANINGS: &str = "Значения";
    pub const BRIEFLY: &str = "Кратко";
    pub const EXAMPLE_SENTENCE: &str = "日本語を勉強しています。(Изучаю японский язык.)";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_rating() {
        let callback = LessonCallback::Rating {
            rating: Rating::Again,
        };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"rating""#));
        assert!(json.contains(r#""rating":"Again""#));
    }

    #[test]
    fn test_serialize_next_card() {
        let callback = LessonCallback::NextCard;
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"next_card""#));
    }

    #[test]
    fn test_deserialize_rating() {
        let json = r#"{"kind":"rating","rating":"Good"}"#;
        let callback = LessonCallback::from_json(json).unwrap();
        assert_eq!(
            callback,
            LessonCallback::Rating {
                rating: Rating::Good
            }
        );
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = LessonCallback::Rating {
            rating: Rating::Hard,
        };
        let json = original.to_json();
        let deserialized = LessonCallback::from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_all_variants_serializable() {
        let variants = vec![
            LessonCallback::Rating {
                rating: Rating::Easy,
            },
            LessonCallback::NextCard,
            LessonCallback::AbortLesson,
            LessonCallback::BackToMain,
        ];

        for variant in variants {
            let json = variant.to_json();
            let deserialized = LessonCallback::from_json(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_try_from_json_valid() {
        let json = r#"{"kind":"abort_lesson"}"#;
        let callback = LessonCallback::try_from_json(json);
        assert_eq!(callback, Some(LessonCallback::AbortLesson));
    }

    #[test]
    fn test_try_from_json_invalid() {
        let json = r#"{"kind":"unknown"}"#;
        let callback = LessonCallback::try_from_json(json);
        assert!(callback.is_none());
    }
}
