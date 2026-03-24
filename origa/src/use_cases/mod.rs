mod analyze_text_for_cards;
mod create_cards_from_analysis;
mod create_grammar_card;
mod create_kanji_card;
mod create_vocabulary_card;
mod delete_card;
mod extract_text_from_image;
mod import_onboarding_sets;
mod rate_card;
mod select_cards_to_fixation;
mod select_cards_to_lesson;
mod toggle_favorite;
mod update_user_profile;

// TODO: Implement import_anki_pack
// mod import_anki_pack;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::fixtures::init_real_dictionaries;

pub use analyze_text_for_cards::{AnalyzeTextForCardsUseCase, AnalyzeTextResult, AnalyzedWord};
pub use create_cards_from_analysis::{
    CreateCardsFromAnalysisResult, CreateCardsFromAnalysisUseCase, WordToCreate,
};
pub use create_grammar_card::CreateGrammarCardUseCase;
pub use create_kanji_card::CreateKanjiCardUseCase;
pub use create_vocabulary_card::CreateVocabularyCardUseCase;
pub use delete_card::DeleteCardUseCase;
pub use extract_text_from_image::ExtractTextFromImageUseCase;
pub use import_onboarding_sets::{
    ImportOnboardingResult, ImportOnboardingSetsUseCase,
};
pub use rate_card::RateCardUseCase;
pub use select_cards_to_fixation::SelectCardsToFixationUseCase;
pub use select_cards_to_lesson::SelectCardsToLessonUseCase;
pub use toggle_favorite::ToggleFavoriteUseCase;
pub use update_user_profile::UpdateUserProfileUseCase;
