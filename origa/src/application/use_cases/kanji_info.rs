use tracing::{debug, info};
use ulid::Ulid;

use crate::{
    application::UserRepository,
    domain::{Card, JapaneseLevel, KanjiInfo, OrigaError, get_kanji_info, get_kanji_list},
};

pub struct KanjiInfoUseCase;

impl Default for KanjiInfoUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl KanjiInfoUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, kanji: &str) -> Result<KanjiInfo, OrigaError> {
        debug!(kanji = %kanji, "Getting kanji info");
        let result = get_kanji_info(kanji)?.to_owned();
        info!(kanji = %kanji, "Kanji info retrieved");
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KanjiItemInfo {
    pub kanji: char,
    pub level: JapaneseLevel,
    pub description: String,
    pub radicals: Vec<char>,
    pub popular_words: Vec<String>,
    pub on_readings: Vec<String>,
    pub kun_readings: Vec<String>,
}

#[derive(Clone)]
pub struct KanjiInfoListUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> KanjiInfoListUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        level: &JapaneseLevel,
    ) -> Result<Vec<KanjiItemInfo>, OrigaError> {
        debug!(user_id = %user_id, level = ?level, "Getting kanji info list");

        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let learned_kanji: std::collections::HashSet<String> = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter_map(|(_, card)| {
                if let Card::Kanji(kanji_card) = card.card() {
                    Some(kanji_card.kanji().text().to_string())
                } else {
                    None
                }
            })
            .collect();

        let result: Vec<KanjiItemInfo> = get_kanji_list(level)
            .into_iter()
            .filter(|kanji_info| !learned_kanji.contains(&kanji_info.kanji().to_string()))
            .map(|kanji_info| KanjiItemInfo {
                kanji: kanji_info.kanji(),
                level: *kanji_info.jlpt(),
                description: kanji_info.description().to_string(),
                radicals: kanji_info.radicals_chars().to_vec(),
                popular_words: kanji_info.popular_words().to_vec(),
                on_readings: kanji_info.on_readings().to_vec(),
                kun_readings: kanji_info.kun_readings().to_vec(),
            })
            .collect();

        info!(count = result.len(), "Kanji info list retrieved");
        Ok(result)
    }
}
