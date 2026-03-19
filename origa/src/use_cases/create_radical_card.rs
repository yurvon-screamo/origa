use crate::domain::{Card, OrigaError, RadicalCard, StudyCard};
use crate::traits::UserRepository;
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct CreateRadicalCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateRadicalCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, radicals: Vec<char>) -> Result<Vec<StudyCard>, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let mut created_cards = vec![];

        for radical in radicals {
            debug!(radical = %radical, "Creating radical card");
            let card = Card::Radical(RadicalCard::new(radical)?);
            match user.create_card(card) {
                Ok(study_card) => {
                    info!(radical = %radical, "Radical card created");
                    created_cards.push(study_card);
                }
                Err(OrigaError::DuplicateCard { question }) => {
                    warn!(radical = %radical, question = %question, "Radical card already exists, skipping");
                }
                Err(e) => return Err(e),
            }
        }

        self.repository.save(&user).await?;
        Ok(created_cards)
    }
}
