use crate::components::cards::grammar_card::{GrammarCardData, GrammarExample};
use crate::components::cards::vocab_card::CardStatus;
use origa::application::GrammarRuleInfoUseCase;
use origa::domain::{JapaneseLevel, OrigaError};
use ulid::Ulid;

#[derive(Clone)]
pub struct GrammarService;

impl GrammarService {
    pub fn new() -> Self {
        Self {}
    }

    /// Получить грамматику по JLPT уровню
    pub async fn get_grammar_by_level(
        &self,
        level: JapaneseLevel,
        _user_id: Ulid,
    ) -> Result<Vec<GrammarCardData>, OrigaError> {
        // TODO: Интегрировать GrammarRuleInfoUseCase когда будет repository
        // let use_case = GrammarRuleInfoUseCase::new(repository);
        // let grammar_list = use_case.execute(user_id, &level).await?;

        // Временно возвращаем пустой список
        Ok(vec![])
    }

    fn calculate_difficulty(&self, level: &JapaneseLevel) -> u32 {
        match level {
            JapaneseLevel::N5 => 20,
            JapaneseLevel::N4 => 35,
            JapaneseLevel::N3 => 50,
            JapaneseLevel::N2 => 70,
            JapaneseLevel::N1 => 85,
        }
    }

    fn get_difficulty_text(&self, level: &JapaneseLevel) -> String {
        match level {
            JapaneseLevel::N5 => "Начальный".to_string(),
            JapaneseLevel::N4 => "Базовый".to_string(),
            JapaneseLevel::N3 => "Средний".to_string(),
            JapaneseLevel::N2 => "Продвинутый".to_string(),
            JapaneseLevel::N1 => "Эксперт".to_string(),
        }
    }
}
