use crate::domain::OrigaError;
use crate::domain::{Card, KanjiCard, StudyCard};
use crate::traits::UserRepository;
use tracing::info;

#[derive(Clone)]
pub struct CreateKanjiCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateKanjiCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, kanjies: Vec<String>) -> Result<Vec<StudyCard>, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let mut cards = vec![];
        for kanji in kanjies {
            info!(kanji = %kanji, "Creating kanji card");
            let card = Card::Kanji(KanjiCard::new(kanji.clone())?);
            let created = user.create_card(card)?;
            info!(kanji = %kanji, "Kanji card created");
            cards.push(created);
        }

        self.repository.save(&user).await?;
        Ok(cards)
    }
}
