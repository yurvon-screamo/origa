use crate::application::UserRepository;
use crate::domain::{OrigaError, WellKnownSets, load_well_known_set};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct WellKnownSetInfo {
    pub set: WellKnownSets,
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
pub struct ListWellKnownSetsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ListWellKnownSetsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<Vec<WellKnownSetInfo>, OrigaError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let native_lang = user.native_language();
        let all_sets = Self::get_all_sets();

        let mut result = Vec::new();
        for set in all_sets {
            let well_known_set = load_well_known_set(&set)?;
            let content = well_known_set.content(native_lang);
            result.push(WellKnownSetInfo {
                set,
                title: content.title().to_string(),
                description: content.description().to_string(),
            });
        }

        Ok(result)
    }

    fn get_all_sets() -> Vec<WellKnownSets> {
        vec![
            WellKnownSets::JlptN5,
            WellKnownSets::JlptN4,
            WellKnownSets::JlptN3,
            WellKnownSets::JlptN2,
            WellKnownSets::JlptN1,
            WellKnownSets::MigiiN5Lesson1,
            WellKnownSets::MigiiN5Lesson2,
            WellKnownSets::MigiiN5Lesson3,
            WellKnownSets::MigiiN5Lesson4,
            WellKnownSets::MigiiN5Lesson5,
            WellKnownSets::MigiiN5Lesson6,
            WellKnownSets::MigiiN5Lesson7,
            WellKnownSets::MigiiN5Lesson8,
            WellKnownSets::MigiiN5Lesson9,
            WellKnownSets::MigiiN5Lesson10,
            WellKnownSets::MigiiN5Lesson11,
            WellKnownSets::MigiiN5Lesson12,
            WellKnownSets::MigiiN5Lesson13,
            WellKnownSets::MigiiN5Lesson14,
            WellKnownSets::MigiiN5Lesson15,
            WellKnownSets::MigiiN5Lesson16,
            WellKnownSets::MigiiN5Lesson17,
            WellKnownSets::MigiiN5Lesson18,
            WellKnownSets::MigiiN5Lesson19,
            WellKnownSets::MigiiN5Lesson20,
            WellKnownSets::MigiiN4Lesson1,
            WellKnownSets::MigiiN4Lesson2,
            WellKnownSets::MigiiN4Lesson3,
            WellKnownSets::MigiiN4Lesson4,
            WellKnownSets::MigiiN4Lesson5,
            WellKnownSets::MigiiN4Lesson6,
            WellKnownSets::MigiiN4Lesson7,
            WellKnownSets::MigiiN4Lesson8,
            WellKnownSets::MigiiN4Lesson9,
            WellKnownSets::MigiiN4Lesson10,
            WellKnownSets::MigiiN4Lesson11,
            WellKnownSets::MigiiN3Lesson1,
            WellKnownSets::MigiiN3Lesson2,
            WellKnownSets::MigiiN3Lesson3,
            WellKnownSets::MigiiN3Lesson4,
            WellKnownSets::MigiiN3Lesson5,
            WellKnownSets::MigiiN3Lesson6,
            WellKnownSets::MigiiN3Lesson7,
            WellKnownSets::MigiiN3Lesson8,
            WellKnownSets::MigiiN3Lesson9,
            WellKnownSets::MigiiN3Lesson10,
            WellKnownSets::MigiiN3Lesson11,
            WellKnownSets::MigiiN3Lesson12,
            WellKnownSets::MigiiN3Lesson13,
            WellKnownSets::MigiiN3Lesson14,
            WellKnownSets::MigiiN3Lesson15,
            WellKnownSets::MigiiN3Lesson16,
            WellKnownSets::MigiiN3Lesson17,
            WellKnownSets::MigiiN3Lesson18,
            WellKnownSets::MigiiN3Lesson19,
            WellKnownSets::MigiiN3Lesson20,
            WellKnownSets::MigiiN3Lesson21,
            WellKnownSets::MigiiN3Lesson22,
            WellKnownSets::MigiiN3Lesson23,
            WellKnownSets::MigiiN3Lesson24,
            WellKnownSets::MigiiN3Lesson25,
            WellKnownSets::MigiiN3Lesson26,
            WellKnownSets::MigiiN3Lesson27,
            WellKnownSets::MigiiN3Lesson28,
            WellKnownSets::MigiiN3Lesson29,
            WellKnownSets::MigiiN3Lesson30,
            WellKnownSets::MigiiN3Lesson31,
            WellKnownSets::MigiiN2Lesson1,
            WellKnownSets::MigiiN2Lesson2,
            WellKnownSets::MigiiN2Lesson3,
            WellKnownSets::MigiiN2Lesson4,
            WellKnownSets::MigiiN2Lesson5,
            WellKnownSets::MigiiN2Lesson6,
            WellKnownSets::MigiiN2Lesson7,
            WellKnownSets::MigiiN2Lesson8,
            WellKnownSets::MigiiN2Lesson9,
            WellKnownSets::MigiiN2Lesson10,
            WellKnownSets::MigiiN2Lesson11,
            WellKnownSets::MigiiN2Lesson12,
            WellKnownSets::MigiiN2Lesson13,
            WellKnownSets::MigiiN2Lesson14,
            WellKnownSets::MigiiN2Lesson15,
            WellKnownSets::MigiiN2Lesson16,
            WellKnownSets::MigiiN2Lesson17,
            WellKnownSets::MigiiN2Lesson18,
            WellKnownSets::MigiiN2Lesson19,
            WellKnownSets::MigiiN2Lesson20,
            WellKnownSets::MigiiN2Lesson21,
            WellKnownSets::MigiiN2Lesson22,
            WellKnownSets::MigiiN2Lesson23,
            WellKnownSets::MigiiN2Lesson24,
            WellKnownSets::MigiiN2Lesson25,
            WellKnownSets::MigiiN2Lesson26,
            WellKnownSets::MigiiN2Lesson27,
            WellKnownSets::MigiiN2Lesson28,
            WellKnownSets::MigiiN2Lesson29,
            WellKnownSets::MigiiN2Lesson30,
            WellKnownSets::MigiiN1Lesson1,
            WellKnownSets::MigiiN1Lesson2,
            WellKnownSets::MigiiN1Lesson3,
            WellKnownSets::MigiiN1Lesson4,
            WellKnownSets::MigiiN1Lesson5,
            WellKnownSets::MigiiN1Lesson6,
            WellKnownSets::MigiiN1Lesson7,
            WellKnownSets::MigiiN1Lesson8,
            WellKnownSets::MigiiN1Lesson9,
            WellKnownSets::MigiiN1Lesson10,
            WellKnownSets::MigiiN1Lesson11,
            WellKnownSets::MigiiN1Lesson12,
            WellKnownSets::MigiiN1Lesson13,
            WellKnownSets::MigiiN1Lesson14,
            WellKnownSets::MigiiN1Lesson15,
            WellKnownSets::MigiiN1Lesson16,
            WellKnownSets::MigiiN1Lesson17,
            WellKnownSets::MigiiN1Lesson18,
            WellKnownSets::MigiiN1Lesson19,
            WellKnownSets::MigiiN1Lesson20,
            WellKnownSets::MigiiN1Lesson21,
            WellKnownSets::MigiiN1Lesson22,
            WellKnownSets::MigiiN1Lesson23,
            WellKnownSets::MigiiN1Lesson24,
            WellKnownSets::MigiiN1Lesson25,
            WellKnownSets::MigiiN1Lesson26,
            WellKnownSets::MigiiN1Lesson27,
            WellKnownSets::MigiiN1Lesson28,
            WellKnownSets::MigiiN1Lesson29,
            WellKnownSets::MigiiN1Lesson30,
            WellKnownSets::MigiiN1Lesson31,
            WellKnownSets::MigiiN1Lesson32,
            WellKnownSets::MigiiN1Lesson33,
            WellKnownSets::MigiiN1Lesson34,
            WellKnownSets::MigiiN1Lesson35,
            WellKnownSets::MigiiN1Lesson36,
            WellKnownSets::MigiiN1Lesson37,
            WellKnownSets::MigiiN1Lesson38,
            WellKnownSets::MigiiN1Lesson39,
            WellKnownSets::MigiiN1Lesson40,
            WellKnownSets::MigiiN1Lesson41,
            WellKnownSets::MigiiN1Lesson42,
            WellKnownSets::MigiiN1Lesson43,
            WellKnownSets::MigiiN1Lesson44,
            WellKnownSets::MigiiN1Lesson45,
            WellKnownSets::MigiiN1Lesson46,
            WellKnownSets::MigiiN1Lesson47,
            WellKnownSets::MigiiN1Lesson48,
            WellKnownSets::MigiiN1Lesson49,
            WellKnownSets::MigiiN1Lesson50,
            WellKnownSets::MigiiN1Lesson51,
            WellKnownSets::MigiiN1Lesson52,
            WellKnownSets::MigiiN1Lesson53,
            WellKnownSets::MigiiN1Lesson54,
            WellKnownSets::MigiiN1Lesson55,
            WellKnownSets::MigiiN1Lesson56,
        ]
    }
}
