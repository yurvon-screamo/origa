use std::collections::HashMap;
use leptos::prelude::*;
use ulid::Ulid;
use origa::domain::{Card, CardType, CardId, OrigaError, JapaneseLevel};
use origa::application::{
    KanjiListUseCase,
    KanjiInfoUseCase,
    CreateKanjiCardUseCase,
    DeleteCardUseCase,
    KnowledgeSetCardsUseCase,
};

#[derive(Clone)]
pub struct KanjiService {
    // In a real app, these would be actual use case instances
    pub kanji_list_use_case: KanjiListUseCase,
    pub kanji_info_use_case: KanjiInfoUseCase,
    pub create_kanji_card_use_case: CreateKanjiCardUseCase,
    pub delete_card_use_case: DeleteCardUseCase,
    pub knowledge_set_cards_use_case: KnowledgeSetCardsUseCase,
}

impl KanjiService {
    pub fn new(
        kanji_list_use_case: KanjiListUseCase,
        kanji_info_use_case: KanjiInfoUseCase,
        create_kanji_card_use_case: CreateKanjiCardUseCase,
        delete_card_use_case: DeleteCardUseCase,
        knowledge_set_cards_use_case: KnowledgeSetCardsUseCase,
    ) -> Self {
        Self {
            kanji_list_use_case,
            kanji_info_use_case,
            create_kanji_card_use_case,
            delete_card_use_case,
            knowledge_set_cards_use_case,
        }
    }
    
    pub async fn get_kanji_by_level(
        &self,
        level: JapaneseLevel,
        user_id: Ulid,
    ) -> Result<Vec<KanjiListData>, OrigaError> {
        // Get kanji list for the specified JLPT level
        let kanji_list = self.kanji_list_use_case.execute(level).await?;
        
        // Convert to UI data structure
        let kanji_data = kanji_list.into_iter()
            .enumerate()
            .map(|(index, kanji_info)| KanjiListData {
                id: format!("kanji_{}", index + 1),
                character: kanji_info.character.clone(),
                stroke_count: kanji_info.stroke_count,
                jlpt_level: level,
                meanings: kanji_info.meanings,
                onyomi: kanji_info.onyomi,
                kunyomi: kanji_info.kunyomi,
                radicals: kanji_info.radicals,
                is_in_knowledge_set: false, // Will be determined from user's cards
            })
            .collect();
            
        Ok(kanji_data)
    }
    
    pub async fn get_kanji_details(
        &self,
        kanji: String,
        user_id: Ulid,
    ) -> Result<KanjiDetailData, OrigaError> {
        // Get detailed information for a specific kanji
        let kanji_info = self.kanji_info_use_case.execute(kanji.clone()).await?;
        
        // Get user's existing cards to determine if this kanji is already added
        let user_cards = self.knowledge_set_cards_use_case.execute(user_id).await?;
        let is_in_knowledge_set = user_cards.values().any(|card| {
            matches!(card, Card::Kanji(kanji_card) if kanji_card.question.text == kanji)
        });
        
        // Convert to UI data structure with enhanced information
        let detail_data = KanjiDetailData {
            id: format!("kanji_detail_{}", kanji),
            character: kanji_info.character.clone(),
            stroke_count: kanji_info.stroke_count,
            grade_level: kanji_info.grade_level,
            jlpt_level: kanji_info.jlpt_level,
            meanings: kanji_info.meanings,
            onyomi: kanji_info.onyomi,
            kunyomi: kanji_info.kunyomi,
            radicals: kanji_info.radicals,
            examples: kanji_info.examples,
            status: self.determine_card_status(&kanji, &user_cards),
            difficulty: self.calculate_difficulty(&kanji_info),
            stability: self.calculate_stability(&kanji, &user_cards),
            next_review: self.calculate_next_review(&kanji, &user_cards),
            is_in_knowledge_set,
            mnemonic_hint: kanji_info.mnemonic_hint.unwrap_or_else(|| "Используйте мнемоники для запоминания".to_string()),
            stroke_order_hint: kanji_info.stroke_order_hint.unwrap_or_else(|| "Соблюдайте порядок чертения".to_string()),
            related_kanji: kanji_info.related_kanji.unwrap_or_default(),
        };
        
        Ok(detail_data)
    }
    
    pub async fn add_kanji_to_knowledge_set(
        &self,
        user_id: Ulid,
        kanji: String,
    ) -> Result<(), OrigaError> {
        // Create a new kanji card in the user's knowledge set
        let new_cards = self.create_kanji_card_use_case.execute(user_id, kanji.clone()).await?;
        
        // Return success if cards were created
        if !new_cards.is_empty() {
            Ok(())
        } else {
            Err(OrigaError::DuplicateCard { question: kanji })
        }
    }
    
    pub async fn remove_kanji_from_knowledge_set(
        &self,
        user_id: Ulid,
        kanji: String,
    ) -> Result<(), OrigaError> {
        // Find the card ID for this kanji
        let user_cards = self.knowledge_set_cards_use_case.execute(user_id).await?;
        
        if let Some((card_id, _)) = user_cards.into_iter().find(|(_, card)| {
            matches!(card, Card::Kanji(kanji_card) if kanji_card.question.text == kanji)
        }) {
            // Delete the card from user's knowledge set
            self.delete_card_use_case.execute(user_id, card_id).await
        } else {
            Err(OrigaError::CardNotFound { card_id: Ulid::new() })
        }
    }
    
    pub async fn get_user_kanji_by_level(
        &self,
        user_id: Ulid,
        level: JapaneseLevel,
    ) -> Result<Vec<KanjiListData>, OrigaError> {
        // Get all kanji for the level
        let all_kanji = self.get_kanji_by_level(level, user_id).await?;
        
        // Get user's cards
        let user_cards = self.knowledge_set_cards_use_case.execute(user_id).await?;
        let user_kanji_set: HashMap<String, _> = user_cards.values()
            .filter_map(|card| {
                if let Card::Kanji(kanji_card) = card {
                    Some((kanji_card.question.text.clone(), kanji_card))
                } else {
                    None
                }
            })
            .collect();
        
        // Update status for each kanji based on whether it's in user's knowledge set
        let updated_kanji = all_kanji.into_iter()
            .map(|mut kanji| {
                kanji.is_in_knowledge_set = user_kanji_set.contains_key(&kanji.character);
                kanji
            })
            .collect();
            
        Ok(updated_kanji)
    }
    
    fn determine_card_status(&self, kanji: &str, user_cards: &HashMap<CardId, Card>) -> crate::components::cards::vocab_card::CardStatus {
        if let Some(card) = user_cards.values().find(|card| {
            matches!(card, Card::Kanji(kanji_card) if kanji_card.question.text == kanji)
        }) {
            // Map memory state to UI status
            // This would depend on the actual memory state from the card
            // For now, return a status based on some logic
            crate::components::cards::vocab_card::CardStatus::InProgress
        } else {
            crate::components::cards::vocab_card::CardStatus::New
        }
    }
    
    fn calculate_difficulty(&self, kanji_info: &origa::domain::KanjiInfo) -> u32 {
        // Calculate difficulty based on stroke count, JLPT level, etc.
        // This is a simplified calculation - in reality, this would be more sophisticated
        let stroke_difficulty = (kanji_info.stroke_count as u32 * 2).min(50);
        let jlpt_difficulty = match kanji_info.jlpt_level {
            JapaneseLevel::N5 => 10,
            JapaneseLevel::N4 => 20,
            JapaneseLevel::N3 => 30,
            JapaneseLevel::N2 => 40,
            JapaneseLevel::N1 => 50,
        };
        
        (stroke_difficulty + jlpt_difficulty).min(100)
    }
    
    fn calculate_stability(&self, kanji_info: &origa::domain::KanjiInfo, user_cards: &HashMap<CardId, Card>) -> u32 {
        // Calculate stability based on how many times the kanji has been reviewed
        // This is a simplified version
        if let Some(_) = user_cards.values().find(|card| {
            matches!(card, Card::Kanji(kanji_card) if kanji_card.question.text == kanji_info.character)
        }) {
            // If the user has this kanji, calculate stability based on review history
            // For now, return a moderate stability
            50
        } else {
            // If user doesn't have this kanji, stability is 0
            0
        }
    }
    
    fn calculate_next_review(&self, kanji: &str, user_cards: &HashMap<CardId, Card>) -> chrono::NaiveDate {
        // Calculate next review date based on memory state
        use chrono::Local;
        
        if let Some(card) = user_cards.values().find(|card| {
            matches!(card, Card::Kanji(kanji_card) if kanji_card.question.text == kanji)
        }) {
            // Calculate next review based on the card's memory state
            // This is a simplified calculation
            Local::now().date_naive() + chrono::Duration::days(3)
        } else {
            // If not in knowledge set, next review is today
            Local::now().date_naive()
        }
    }
}

// UI data structures
#[derive(Clone)]
pub struct KanjiListData {
    pub id: String,
    pub character: String,
    pub stroke_count: u8,
    pub jlpt_level: JapaneseLevel,
    pub meanings: Vec<String>,
    pub onyomi: Vec<String>,
    pub kunyomi: Vec<String>,
    pub radicals: Vec<RadicalInfo>,
    pub is_in_knowledge_set: bool,
}

#[derive(Clone)]
pub struct RadicalInfo {
    pub character: String,
    pub meaning: String,
    pub strokes: u8,
}