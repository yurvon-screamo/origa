use ulid::Ulid;

use crate::{
    application::UserRepository,
    domain::{
        KeikakuError, PartOfSpeech, {GRAMMAR_RULES, GrammarRule}, {JapaneseLevel, NativeLanguage},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct GrammarRuleItem {
    rule_id: Ulid,
    level: JapaneseLevel,
    apply_to: Vec<PartOfSpeech>,

    title: String,
    short_description: String,
    md_description: String,
}

#[derive(Clone)]
pub struct GrammarRuleInfoUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> GrammarRuleInfoUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        level: &JapaneseLevel,
    ) -> Result<Vec<GrammarRuleItem>, KeikakuError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(KeikakuError::UserNotFound { user_id })?;

        let lang = user.native_language();

        Ok(GRAMMAR_RULES
            .iter()
            .filter_map(|x| filter_by_level(x.as_ref(), lang, level))
            .collect())
    }
}

fn filter_by_level(
    x: &dyn GrammarRule,
    lang: &NativeLanguage,
    level: &JapaneseLevel,
) -> Option<GrammarRuleItem> {
    let info = x.info();
    if info.level() == level {
        let content = info.content(lang);
        Some(GrammarRuleItem {
            rule_id: *info.rule_id(),
            level: *info.level(),
            apply_to: info.apply_to().to_vec(),
            title: content.title().to_string(),
            short_description: content.short_description().to_string(),
            md_description: content.md_description().to_string(),
        })
    } else {
        None
    }
}
