use crate::dictionary::kanji::get_kanji_info;
use crate::dictionary::radical::get_radical_info;
use crate::domain::OrigaError;
use crate::domain::{Card, KanjiCard, RadicalCard, StudyCard};
use crate::traits::UserRepository;
use tracing::{debug, info};

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
            debug!(kanji = %kanji, "Creating kanji card");
            let card = Card::Kanji(KanjiCard::new(kanji.clone())?);
            let created = user.create_card(card)?;
            info!(kanji = %kanji, "Kanji card created");
            cards.push(created);

            // Автосоздание радикалов для кандзи
            if let Ok(kanji_info) = get_kanji_info(&kanji) {
                for radical_char in kanji_info.radicals_chars() {
                    if get_radical_info(*radical_char).is_ok()
                        && user
                            .create_card(Card::Radical(RadicalCard::new(*radical_char)?))
                            .is_ok()
                    {
                        debug!(
                            radical = %radical_char,
                            "Auto-created radical card for kanji {}",
                            kanji
                        );
                    }
                }
            }
        }

        self.repository.save(&user).await?;
        Ok(cards)
    }
}
