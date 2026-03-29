use std::collections::HashSet;

use tracing::{debug, info, warn};

use crate::dictionary::grammar::get_rules_by_level;
use crate::dictionary::kanji::get_kanji_info;
use crate::dictionary::radical::get_radical_info;
use crate::domain::{
    Card, GrammarRuleCard, JapaneseLevel, KanjiCard, OrigaError, RadicalCard, StudyCard,
    VocabularyCard,
};
use crate::traits::{UserRepository, WellKnownSetLoader, id_to_set_type};

pub struct ImportOnboardingResult {
    pub imported_set_ids: Vec<String>,
    pub created_vocabulary: usize,
    pub created_kanji: usize,
    pub created_grammar: usize,
    pub created_radicals: usize,
    pub skipped_duplicates: usize,
    pub skipped_no_translation: usize,
}

#[derive(Clone)]
pub struct ImportOnboardingSetsUseCase<'a, R: UserRepository, L: WellKnownSetLoader> {
    repository: &'a R,
    loader: &'a L,
}

impl<'a, R: UserRepository, L: WellKnownSetLoader> ImportOnboardingSetsUseCase<'a, R, L> {
    pub fn new(repository: &'a R, loader: &'a L) -> Self {
        Self { repository, loader }
    }

    pub async fn execute(
        &self,
        user_id: ulid::Ulid,
        set_ids: Vec<String>,
    ) -> Result<ImportOnboardingResult, OrigaError> {
        debug!(user_id = %user_id, set_count = set_ids.len(), "Starting onboarding sets import");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        if user.id() != user_id {
            return Err(OrigaError::CurrentUserNotExist {});
        }

        let current_level = user.current_japanese_level();
        let native_language = *user.native_language();

        let sets = self.loader.load_sets(set_ids.clone()).await?;

        let mut result = ImportOnboardingResult {
            imported_set_ids: Vec::new(),
            created_vocabulary: 0,
            created_kanji: 0,
            created_grammar: 0,
            created_radicals: 0,
            skipped_duplicates: 0,
            skipped_no_translation: 0,
        };

        let mut created_kanji_chars: HashSet<String> = HashSet::new();
        let mut jlpt_imported_levels: HashSet<JapaneseLevel> = HashSet::new();

        for (set_id, set) in sets {
            debug!(set_id = %set_id, words_count = set.words().len(), "Processing set");

            let set_level = *set.level();
            if id_to_set_type(&set_id) == "Jlpt" {
                jlpt_imported_levels.insert(set_level);
            }
            let words_result = VocabularyCard::from_text(&set.words().join(" "), &native_language);

            result.skipped_no_translation += words_result.skipped_no_translation.len();

            for vocab_card in words_result.cards {
                if let Ok(study_card) = self.create_vocabulary_card(
                    &mut user,
                    vocab_card,
                    &mut result.skipped_duplicates,
                ) {
                    result.created_vocabulary += 1;

                    self.process_kanji_from_vocab(
                        &study_card,
                        set_level,
                        &current_level,
                        &mut user,
                        &mut created_kanji_chars,
                        &mut result,
                    );
                }
            }

            result.imported_set_ids.push(set_id);
        }

        debug!(levels = ?jlpt_imported_levels, "Importing grammar rules for onboarding levels");

        for level in &jlpt_imported_levels {
            let grammar_rules = get_rules_by_level(level);
            for rule in grammar_rules {
                if let Ok(grammar_card) = GrammarRuleCard::new(*rule.rule_id()) {
                    let card = Card::Grammar(grammar_card);
                    match user.create_card(card) {
                        Ok(_) => {
                            result.created_grammar += 1;
                        },
                        Err(OrigaError::DuplicateCard { .. }) => {
                            result.skipped_duplicates += 1;
                        },
                        Err(e) => {
                            warn!(error = ?e, "Failed to create grammar card");
                        },
                    }
                }
            }
        }

        user.mark_sets_as_imported(set_ids);
        self.repository.save_sync(&user).await?;

        info!(
            user_id = %user_id,
            vocabulary = result.created_vocabulary,
            kanji = result.created_kanji,
            grammar = result.created_grammar,
            radicals = result.created_radicals,
            duplicates = result.skipped_duplicates,
            "Onboarding sets import completed"
        );

        Ok(result)
    }

    fn create_vocabulary_card(
        &self,
        user: &mut crate::domain::User,
        vocab_card: VocabularyCard,
        skipped_duplicates: &mut usize,
    ) -> Result<StudyCard, OrigaError> {
        let card = Card::Vocabulary(vocab_card);
        match user.create_card(card) {
            Ok(study_card) => {
                debug!(word = ?study_card.card().question(&crate::domain::NativeLanguage::Russian), "Vocabulary card created");
                Ok(study_card)
            },
            Err(OrigaError::DuplicateCard { question }) => {
                warn!(word = %question, "Duplicate vocabulary card, skipping");
                *skipped_duplicates += 1;
                Err(OrigaError::DuplicateCard { question })
            },
            Err(e) => Err(e),
        }
    }

    fn extract_kanji_chars(
        &self,
        study_card: &StudyCard,
        current_level: &JapaneseLevel,
    ) -> Vec<String> {
        if let Card::Vocabulary(vocab) = study_card.card() {
            vocab
                .get_kanji_cards(current_level)
                .into_iter()
                .map(|info| info.kanji().to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn process_kanji_from_vocab(
        &self,
        study_card: &StudyCard,
        set_level: JapaneseLevel,
        current_level: &JapaneseLevel,
        user: &mut crate::domain::User,
        created_kanji_chars: &mut HashSet<String>,
        result: &mut ImportOnboardingResult,
    ) {
        let kanji_chars = self.extract_kanji_chars(study_card, current_level);

        for kanji_char in kanji_chars {
            if created_kanji_chars.contains(&kanji_char) {
                continue;
            }

            if !self.should_create_kanji_card(&kanji_char, set_level, current_level) {
                continue;
            }

            if self.create_kanji_card(user, &kanji_char, result).is_ok() {
                result.created_kanji += 1;
                created_kanji_chars.insert(kanji_char);
            }
        }
    }

    fn should_create_kanji_card(
        &self,
        kanji_char: &str,
        set_level: JapaneseLevel,
        current_level: &JapaneseLevel,
    ) -> bool {
        set_level <= *current_level
            || crate::dictionary::kanji::get_kanji_info(kanji_char)
                .map(|info| info.jlpt() <= current_level)
                .unwrap_or(false)
    }

    fn create_kanji_card(
        &self,
        user: &mut crate::domain::User,
        kanji_char: &str,
        result: &mut ImportOnboardingResult,
    ) -> Result<StudyCard, OrigaError> {
        match KanjiCard::new(kanji_char.to_string()) {
            Ok(kanji_card) => {
                let card = Card::Kanji(kanji_card);
                match user.create_card(card) {
                    Ok(study_card) => {
                        debug!(kanji = %kanji_char, "Kanji card created");
                        if let Ok(kanji_info) = get_kanji_info(kanji_char) {
                            for radical_char in kanji_info.radicals_chars() {
                                if get_radical_info(*radical_char).is_ok() {
                                    match RadicalCard::new(*radical_char) {
                                        Ok(radical_card) => {
                                            match user.create_card(Card::Radical(radical_card)) {
                                                Ok(_) => {
                                                    debug!(radical = %radical_char, "Auto-created radical card for kanji {}", kanji_char);
                                                    result.created_radicals += 1;
                                                },
                                                Err(OrigaError::DuplicateCard { .. }) => {
                                                    result.skipped_duplicates += 1;
                                                },
                                                Err(_) => {},
                                            }
                                        },
                                        Err(e) => {
                                            warn!(radical = %radical_char, error = ?e, "Failed to create radical card");
                                        },
                                    }
                                }
                            }
                        }
                        Ok(study_card)
                    },
                    Err(OrigaError::DuplicateCard { question }) => {
                        warn!(kanji = %question, "Duplicate kanji card, skipping");
                        result.skipped_duplicates += 1;
                        Err(OrigaError::DuplicateCard { question })
                    },
                    Err(e) => Err(e),
                }
            },
            Err(e) => {
                warn!(kanji = %kanji_char, error = ?e, "Failed to create kanji card");
                Err(e)
            },
        }
    }
}
