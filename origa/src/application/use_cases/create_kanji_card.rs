use crate::application::UserRepository;
use crate::domain::OrigaError;
use crate::domain::{Card, KanjiCard, StudyCard};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateKanjiCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateKanjiCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        kanjies: Vec<String>,
    ) -> Result<Vec<StudyCard>, OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let mut cards = vec![];
        for kanji in kanjies {
            let card = Card::Kanji(KanjiCard::new(kanji, user.native_language())?);
            let created = user.create_card(card)?;
            cards.push(created);
        }

        self.repository.save(&user).await?;
        Ok(cards)
    }
}
