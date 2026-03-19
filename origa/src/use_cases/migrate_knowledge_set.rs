use crate::domain::{Card, MemoryHistory, OrigaError};
use crate::traits::UserRepository;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use ulid::Ulid;

struct OldCardData {
    card: Card,
    memory_history: MemoryHistory,
    is_favorite: bool,
    perfect_streak: u8,
}

pub struct MigrateKnowledgeSetUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> MigrateKnowledgeSetUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<bool, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        info!("Starting migration");

        let old_data = self.collect_old_card_data(user.knowledge_set());

        if old_data.is_empty() {
            info!("Migration skipped: no cards to migrate");
            return Ok(false);
        }

        debug!(cards_count = old_data.len(), "Starting migration");

        let old_lesson_history = user.knowledge_set().lesson_history().to_vec();
        user.knowledge_set_mut().clear_study_cards();

        let mut new_card_ids: HashMap<String, Ulid> = HashMap::new();
        for (content_key, old) in &old_data {
            match user.create_card(old.card.clone()) {
                Ok(new_study_card) => {
                    new_card_ids.insert(content_key.clone(), *new_study_card.card_id());
                }
                Err(OrigaError::DuplicateCard { .. }) => {
                    warn!(content_key = %content_key, "Unexpected duplicate during migration");
                }
                Err(e) => return Err(e),
            }
        }

        for (content_key, old) in old_data {
            if let Some(&new_card_id) = new_card_ids.get(&content_key)
                && let Some(new_card) = user.knowledge_set_mut().get_card_mut(&new_card_id)
            {
                new_card.set_memory_history(old.memory_history);
                new_card.set_is_favorite(old.is_favorite);
                new_card.set_perfect_streak(old.perfect_streak);
            }
        }

        user.knowledge_set_mut()
            .set_lesson_history(old_lesson_history);
        self.repository.save_sync(&user).await?;

        info!(
            migrated_count = new_card_ids.len(),
            "Migration completed successfully"
        );
        Ok(true)
    }

    fn collect_old_card_data(
        &self,
        knowledge_set: &crate::domain::KnowledgeSet,
    ) -> HashMap<String, OldCardData> {
        let mut result: HashMap<String, OldCardData> = HashMap::new();

        for study_card in knowledge_set.study_cards().values() {
            let content_key = study_card.card().content_key();
            let memory = study_card.memory().clone();
            let stability = memory.stability().map(|s| s.value()).unwrap_or(0.0);

            let new_data = OldCardData {
                card: study_card.card().clone(),
                memory_history: memory,
                is_favorite: study_card.is_favorite(),
                perfect_streak: study_card.perfect_streak_since_known(),
            };

            result
                .entry(content_key)
                .and_modify(|existing| {
                    let existing_stability = existing
                        .memory_history
                        .stability()
                        .map(|s| s.value())
                        .unwrap_or(0.0);
                    if stability > existing_stability {
                        *existing = OldCardData {
                            card: new_data.card.clone(),
                            memory_history: new_data.memory_history.clone(),
                            is_favorite: new_data.is_favorite,
                            perfect_streak: new_data.perfect_streak,
                        };
                    }
                })
                .or_insert(new_data);
        }

        result
    }
}
