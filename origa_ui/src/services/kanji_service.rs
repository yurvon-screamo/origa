use crate::components::cards::kanji_card::RadicalInfo;
use crate::components::cards::kanji_detail::{ExampleInfo, KanjiDetailData};
use leptos::prelude::*;
use origa::application::{KanjiInfoUseCase, KanjiListUseCase};
use origa::domain::{Card, JapaneseLevel, OrigaError};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Clone)]
pub struct KanjiService;

impl KanjiService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_kanji_by_level(
        &self,
        level: JapaneseLevel,
        _user_id: Ulid,
    ) -> Result<Vec<KanjiListData>, OrigaError> {
        // Get kanji list for the specified JLPT level
        let use_case = KanjiListUseCase::new();
        let kanji_list = use_case.execute(&level)?;

        // Get user's existing cards to determine if kanji is already added
        // TODO: Implement when KnowledgeSetCardsUseCase is available
        let empty_user_cards: HashMap<Ulid, Card> = HashMap::new();

        // Convert to UI data structure
        let kanji_data = kanji_list
            .into_iter()
            .enumerate()
            .map(|(index, kanji_info)| {
                let kanji_char = kanji_info.kanji().to_string();
                let is_in_knowledge_set = empty_user_cards.values().any(|card| {
                    matches!(card, Card::Kanji(kanji_card) if kanji_card.kanji().text() == kanji_char)
                });

                KanjiListData {
                    id: format!("kanji_{}", index + 1),
                    character: kanji_char.clone(),
                    stroke_count: 0, // Not available in KanjiInfo
                    jlpt_level: *kanji_info.jlpt(),
                    meanings: vec![kanji_info.description().to_string()],
                    onyomi: vec![], // Not available in KanjiInfo
                    kunyomi: vec![], // Not available in KanjiInfo
                    radicals: kanji_info
                        .radicals()
                        .into_iter()
                        .map(|r| RadicalInfo {
                            character: r.radical().to_string(),
                            meaning: r.name().to_string(),
                        })
                        .collect(),
                    status: self.determine_card_status(&kanji_char, &empty_user_cards),
                    difficulty: self.calculate_difficulty(&kanji_info),
                    stability: self.calculate_stability(&kanji_info, &empty_user_cards),
                    next_review: self.calculate_next_review(&kanji_char, &empty_user_cards),
                    is_in_knowledge_set,
                }
            })
            .collect();

        Ok(kanji_data)
    }

    pub async fn add_kanji_to_knowledge_set(&self, kanji: String) -> Result<(), OrigaError> {
        // Create a new kanji card in the user's knowledge set
        // TODO: Implement create_kanji_card_use_case
        // let new_cards = self.create_kanji_card_use_case.execute(user_id, kanji.clone()).await?;
        let new_cards: Vec<Card> = vec![];

        // Return success if cards were created
        if !new_cards.is_empty() {
            Ok(())
        } else {
            Err(OrigaError::DuplicateCard { question: kanji })
        }
    }

    pub async fn remove_kanji_from_knowledge_set(
        &self,
        _user_id: Ulid,
        _kanji: String,
    ) -> Result<(), OrigaError> {
        // TODO: Implement when KnowledgeSetCardsUseCase and DeleteCardUseCase are available
        Err(OrigaError::RepositoryError {
            reason: "Not implemented yet".to_string(),
        })
    }

    pub async fn get_user_kanji_by_level(
        &self,
        user_id: Ulid,
        level: JapaneseLevel,
    ) -> Result<Vec<KanjiListData>, OrigaError> {
        // Get all kanji for the level
        let all_kanji = self.get_kanji_by_level(level, user_id).await?;

        // TODO: Get user's cards when KnowledgeSetCardsUseCase is available
        // For now, return kanji without user-specific status
        Ok(all_kanji)
    }

    pub async fn get_kanji_detail(
        &self,
        kanji_char: String,
        _user_id: Ulid,
    ) -> Result<KanjiDetailData, OrigaError> {
        // Get kanji info from use case
        let use_case = KanjiInfoUseCase::new();
        let kanji_info = use_case.execute(&kanji_char)?;

        // Get user's existing cards to determine if kanji is already added
        // TODO: Implement when KnowledgeSetCardsUseCase is available
        let empty_user_cards: HashMap<Ulid, Card> = HashMap::new();

        // Convert to KanjiDetailData
        let radicals = kanji_info
            .radicals()
            .into_iter()
            .map(|r| crate::components::cards::kanji_detail::RadicalDetail {
                character: r.radical().to_string(),
                meaning: r.name().to_string(),
                stroke_count: 0,          // Not available in RadicalInfo
                position: "".to_string(), // Not available
            })
            .collect();

        // Convert popular words to examples
        let examples = kanji_info
            .popular_words()
            .iter()
            .take(5) // Limit to 5 examples
            .map(|word| ExampleInfo {
                kanji: word.clone(),
                reading: "".to_string(), // Not available
                meaning: "".to_string(), // Not available
                romaji: "".to_string(),  // Not available
            })
            .collect();

        // Split description into meanings (assuming comma-separated)
        let meanings: Vec<String> = kanji_info
            .description()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let is_in_knowledge_set = empty_user_cards.values().any(|card| {
            matches!(card, Card::Kanji(kanji_card) if kanji_card.kanji().text() == kanji_char)
        });

        Ok(KanjiDetailData {
            id: format!("kanji_{}", kanji_char),
            character: kanji_char.clone(),
            stroke_count: 0, // Not available in KanjiInfo
            grade_level: format!("JLPT {}", kanji_info.jlpt()),
            jlpt_level: *kanji_info.jlpt(),
            meanings: if meanings.is_empty() {
                vec![kanji_info.description().to_string()]
            } else {
                meanings
            },
            onyomi: vec![],  // Not available in KanjiInfo
            kunyomi: vec![], // Not available in KanjiInfo
            radicals,
            examples,
            status: self.determine_card_status(&kanji_char, &empty_user_cards),
            difficulty: self.calculate_difficulty(&kanji_info),
            stability: self.calculate_stability(&kanji_info, &empty_user_cards),
            next_review: self.calculate_next_review(&kanji_char, &empty_user_cards),
            is_in_knowledge_set,
            mnemonic_hint: format!(
                "Кандзи {} часто используется в словах: {}",
                kanji_char,
                kanji_info
                    .popular_words()
                    .iter()
                    .take(3)
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            stroke_order_hint: format!(
                "Порядок черт для {}: слева направо, сверху вниз",
                kanji_char
            ),
            related_kanji: vec![], // Not available
        })
    }

    fn determine_card_status(
        &self,
        kanji: &str,
        user_cards: &HashMap<Ulid, Card>,
    ) -> crate::components::cards::vocab_card::CardStatus {
        if let Some(_card) = user_cards.values().find(
            |card| matches!(card, Card::Kanji(kanji_card) if kanji_card.kanji().text() == kanji),
        ) {
            // Map memory state to UI status
            // This would depend on the actual memory state from the card
            // For now, return a status based on some logic
            crate::components::cards::vocab_card::CardStatus::InProgress
        } else {
            crate::components::cards::vocab_card::CardStatus::New
        }
    }

    fn calculate_difficulty(&self, kanji_info: &origa::domain::KanjiInfo) -> u32 {
        // Calculate difficulty based on JLPT level and usage frequency
        // Use used_in as a proxy for complexity (more common kanji = easier)
        // Higher used_in means more common, so lower difficulty
        let usage_factor = (100.0 / (kanji_info.used_in() as f32 + 1.0) * 10.0).min(50.0) as u32;
        let jlpt_difficulty = match kanji_info.jlpt() {
            JapaneseLevel::N5 => 10,
            JapaneseLevel::N4 => 20,
            JapaneseLevel::N3 => 30,
            JapaneseLevel::N2 => 40,
            JapaneseLevel::N1 => 50,
        };

        (usage_factor + jlpt_difficulty).min(100)
    }

    fn calculate_stability(
        &self,
        kanji_info: &origa::domain::KanjiInfo,
        user_cards: &HashMap<Ulid, Card>,
    ) -> u32 {
        // Calculate stability based on how many times the kanji has been reviewed
        // This is a simplified version
        let kanji_char = kanji_info.kanji().to_string();
        if user_cards.values().any(|card| {
            matches!(&card, Card::Kanji(kanji_card) if kanji_card.kanji().text() == kanji_char)
        }) {
            // If the user has this kanji, calculate stability based on review history
            // For now, return a moderate stability
            50
        } else {
            // If user doesn't have this kanji, stability is 0
            0
        }
    }

    fn calculate_next_review(
        &self,
        kanji: &str,
        user_cards: &HashMap<Ulid, Card>,
    ) -> chrono::NaiveDate {
        // Calculate next review date based on memory state
        use chrono::Local;

        if let Some(_card) = user_cards.values().find(
            |card| matches!(card, Card::Kanji(kanji_card) if kanji_card.kanji().text() == kanji),
        ) {
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
// KanjiListData is compatible with KanjiCardData from kanji_card.rs
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
    pub status: crate::components::cards::vocab_card::CardStatus,
    pub difficulty: u32,
    pub stability: u32,
    pub next_review: chrono::NaiveDate,
    pub is_in_knowledge_set: bool,
}

// Implement conversion to KanjiCardData
impl From<KanjiListData> for crate::components::cards::kanji_card::KanjiCardData {
    fn from(data: KanjiListData) -> Self {
        Self {
            id: data.id,
            character: data.character,
            stroke_count: data.stroke_count,
            jlpt_level: data.jlpt_level,
            meanings: data.meanings,
            onyomi: data.onyomi,
            kunyomi: data.kunyomi,
            radicals: data.radicals,
            status: data.status,
            difficulty: data.difficulty,
            stability: data.stability,
            next_review: data.next_review,
            is_in_knowledge_set: data.is_in_knowledge_set,
        }
    }
}
