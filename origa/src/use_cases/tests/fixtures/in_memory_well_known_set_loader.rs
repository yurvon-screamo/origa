use std::collections::HashMap;

use crate::domain::{JapaneseLevel, OrigaError};
use crate::traits::{SetType, WellKnownSet, WellKnownSetLoader, WellKnownSetMeta};

pub struct InMemoryWellKnownSetLoader {
    sets: HashMap<String, WellKnownSet>,
    meta_list: Vec<WellKnownSetMeta>,
}

impl Default for InMemoryWellKnownSetLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryWellKnownSetLoader {
    pub fn new() -> Self {
        let mut sets = HashMap::new();
        let mut meta_list = Vec::new();

        let n5_words = vec![
            "あさって".to_string(),
            "こんな".to_string(),
            "そう".to_string(),
            "そして".to_string(),
            "で".to_string(),
        ];

        sets.insert(
            "jltp_n5".to_string(),
            WellKnownSet::new(JapaneseLevel::N5, n5_words),
        );
        meta_list.push(WellKnownSetMeta {
            id: "jltp_n5".to_string(),
            set_type: SetType::Jlpt,
            level: JapaneseLevel::N5,
            title_ru: "Рекомендованные слова для уровня N5".to_string(),
            title_en: "Recommended words for N5 level".to_string(),
            desc_ru: "Список слов, рекомендованных для изучения языка на уровне N5.".to_string(),
            desc_en: "Words list, recommended for learning language for N5 level.".to_string(),
        });

        let n4_words = vec![
            "あいにく".to_string(),
            "あえて".to_string(),
            "あらかじめ".to_string(),
        ];

        sets.insert(
            "jltp_n4".to_string(),
            WellKnownSet::new(JapaneseLevel::N4, n4_words),
        );
        meta_list.push(WellKnownSetMeta {
            id: "jltp_n4".to_string(),
            set_type: SetType::Jlpt,
            level: JapaneseLevel::N4,
            title_ru: "Рекомендованные слова для уровня N4".to_string(),
            title_en: "Recommended words for N4 level".to_string(),
            desc_ru: "Список слов, рекомендованных для изучения языка на уровне N4.".to_string(),
            desc_en: "Words list, recommended for learning language for N4 level.".to_string(),
        });

        Self { sets, meta_list }
    }
}

impl WellKnownSetLoader for InMemoryWellKnownSetLoader {
    async fn load_meta_list(&self) -> Result<Vec<WellKnownSetMeta>, OrigaError> {
        Ok(self.meta_list.clone())
    }

    async fn load_set(&self, id: String) -> Result<WellKnownSet, OrigaError> {
        self.sets
            .get(&id)
            .cloned()
            .ok_or_else(|| OrigaError::RepositoryError {
                reason: format!("Set {} not found", id),
            })
    }
}
