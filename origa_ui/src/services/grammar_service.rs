use crate::components::cards::grammar_card::GrammarCardData;
use crate::components::cards::vocab_card::CardStatus;
use origa::application::{
    CreateGrammarCardUseCase, DeleteCardUseCase, GrammarRuleInfoUseCase, KnowledgeSetCardsUseCase,
};
use origa::domain::grammar::get_rule_by_id;
use origa::domain::{Card, JapaneseLevel, OrigaError, StudyCard};
use origa::settings::ApplicationEnvironment;
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
        user_id: Ulid,
    ) -> Result<Vec<GrammarCardData>, OrigaError> {
        let repository = ApplicationEnvironment::get().get_repository().await?;

        // Получить доступные правила грамматики для уровня
        let grammar_use_case = GrammarRuleInfoUseCase::new(repository);
        let grammar_rules = grammar_use_case.execute(user_id, &level).await?;

        // Получить карточки пользователя
        let knowledge_use_case = KnowledgeSetCardsUseCase::new(repository);
        let user_study_cards = knowledge_use_case.execute(user_id).await?;

        // Создать HashMap для быстрого поиска карточек по rule_id
        let user_grammar_cards: std::collections::HashMap<Ulid, &StudyCard> = user_study_cards
            .iter()
            .filter_map(|sc| {
                if let Card::Grammar(grammar_card) = sc.card() {
                    Some((*grammar_card.rule_id(), sc))
                } else {
                    None
                }
            })
            .collect();

        // Конвертировать правила в GrammarCardData
        let grammar_data: Vec<GrammarCardData> = grammar_rules
            .into_iter()
            .map(|rule_item| {
                let rule_id = rule_item.rule_id;
                let _is_in_knowledge_set = user_grammar_cards.contains_key(&rule_id);

                if let Some(study_card) = user_grammar_cards.get(&rule_id) {
                    // Карточка уже в knowledge set - использовать данные из StudyCard
                    self.convert_study_card_to_grammar_data(study_card, &level)
                } else {
                    // Карточка не в knowledge set - создать из GrammarRuleItem
                    self.convert_rule_item_to_grammar_data(&rule_item, &level)
                }
            })
            .collect();

        Ok(grammar_data)
    }

    /// Конвертировать StudyCard с Grammar в GrammarCardData
    fn convert_study_card_to_grammar_data(
        &self,
        study_card: &StudyCard,
        level: &JapaneseLevel,
    ) -> GrammarCardData {
        let card_id = study_card.card_id();
        let memory = study_card.memory();

        // Определить статус
        let status = self.map_memory_to_status(memory);

        // Получить difficulty и stability
        let difficulty = memory
            .difficulty()
            .map(|d| (d.value() * 100.0) as u32)
            .unwrap_or_else(|| self.calculate_difficulty(level));
        let stability = memory
            .stability()
            .map(|s| (s.value() * 100.0) as u32)
            .unwrap_or(0);

        // Получить дату следующего повторения
        let next_review = memory
            .next_review_date()
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        if let Card::Grammar(grammar) = study_card.card() {
            GrammarCardData {
                id: card_id.to_string(),
                pattern: grammar.title().text().to_string(),
                meaning: grammar.description().text().to_string(),
                attachment_rules: grammar
                    .apply_to()
                    .iter()
                    .map(|pos| format!("{:?}", pos))
                    .collect::<Vec<_>>()
                    .join(", "),
                difficulty,
                difficulty_text: self.get_difficulty_text(level),
                stability,
                jlpt_level: *level,
                examples: vec![], // Examples не хранятся в GrammarRuleCard
                status,
                next_review,
                is_in_knowledge_set: true,
            }
        } else {
            // Fallback (не должно произойти)
            GrammarCardData {
                id: card_id.to_string(),
                pattern: String::new(),
                meaning: String::new(),
                attachment_rules: String::new(),
                difficulty,
                difficulty_text: self.get_difficulty_text(level),
                stability,
                jlpt_level: *level,
                examples: vec![],
                status,
                next_review,
                is_in_knowledge_set: true,
            }
        }
    }

    /// Конвертировать GrammarRuleItem в GrammarCardData
    fn convert_rule_item_to_grammar_data(
        &self,
        rule_item: &origa::application::GrammarRuleItem,
        level: &JapaneseLevel,
    ) -> GrammarCardData {
        GrammarCardData {
            id: rule_item.rule_id.to_string(),
            pattern: rule_item.title.clone(),
            meaning: rule_item.short_description.clone(),
            attachment_rules: rule_item
                .apply_to
                .iter()
                .map(|pos| format!("{:?}", pos))
                .collect::<Vec<_>>()
                .join(", "),
            difficulty: self.calculate_difficulty(level),
            difficulty_text: self.get_difficulty_text(level),
            stability: 0, // Новые карточки имеют stability 0
            jlpt_level: rule_item.level,
            examples: vec![], // Examples не доступны в GrammarRuleItem
            status: CardStatus::New,
            next_review: chrono::Utc::now().naive_utc(),
            is_in_knowledge_set: false,
        }
    }

    /// Маппинг MemoryHistory -> CardStatus
    fn map_memory_to_status(&self, memory: &origa::domain::MemoryHistory) -> CardStatus {
        if memory.is_new() {
            CardStatus::New
        } else if memory.is_high_difficulty() {
            CardStatus::Difficult
        } else if memory.is_known_card() {
            CardStatus::Mastered
        } else if memory.is_in_progress() {
            CardStatus::InProgress
        } else {
            CardStatus::New
        }
    }

    /// Добавить грамматику в knowledge set
    pub async fn add_grammar_to_knowledge_set(
        &self,
        user_id: Ulid,
        rule_id: Ulid,
    ) -> Result<(), OrigaError> {
        let repository = ApplicationEnvironment::get().get_repository().await?;

        // Получить GrammarRuleInfo из rule_id
        let rule = get_rule_by_id(&rule_id).ok_or_else(|| OrigaError::RepositoryError {
            reason: format!("Grammar rule {} not found", rule_id),
        })?;

        let rule_info = rule.info().clone();
        let use_case = CreateGrammarCardUseCase::new(repository);
        use_case.execute(user_id, vec![rule_info]).await?;

        Ok(())
    }

    /// Удалить грамматику из knowledge set
    pub async fn remove_grammar_from_knowledge_set(
        &self,
        user_id: Ulid,
        rule_id: Ulid,
    ) -> Result<(), OrigaError> {
        let repository = ApplicationEnvironment::get().get_repository().await?;

        // Найти card_id по rule_id
        let knowledge_use_case = KnowledgeSetCardsUseCase::new(repository);
        let user_study_cards = knowledge_use_case.execute(user_id).await?;

        if let Some(study_card) = user_study_cards.iter().find(|sc| {
            if let Card::Grammar(grammar_card) = sc.card() {
                grammar_card.rule_id() == &rule_id
            } else {
                false
            }
        }) {
            let card_id = *study_card.card_id();
            let delete_use_case = DeleteCardUseCase::new(repository);
            delete_use_case.execute(user_id, card_id).await
        } else {
            Err(OrigaError::RepositoryError {
                reason: format!("Grammar rule {} not found in knowledge set", rule_id),
            })
        }
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
