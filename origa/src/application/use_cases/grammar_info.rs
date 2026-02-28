use std::collections::HashSet;
use ulid::Ulid;

use crate::{
    application::UserRepository,
    domain::{
        GrammarRule, JapaneseLevel, NativeLanguage, OrigaError, PartOfSpeech, iter_grammar_rules,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct GrammarRuleItem {
    pub rule_id: Ulid,
    pub level: JapaneseLevel,
    pub apply_to: Vec<PartOfSpeech>,

    pub title: String,
    pub short_description: String,
    pub md_description: String,
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
        existing_rule_ids: &HashSet<Ulid>,
    ) -> Result<Vec<GrammarRuleItem>, OrigaError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let lang = user.native_language();

        Ok(iter_grammar_rules()
            .filter_map(|rule| filter_by_level(rule, lang, level))
            .filter(|item| !existing_rule_ids.contains(&item.rule_id))
            .collect())
    }
}

fn filter_by_level(
    rule: &GrammarRule,
    lang: &NativeLanguage,
    level: &JapaneseLevel,
) -> Option<GrammarRuleItem> {
    if rule.level() == level {
        let content = rule.content(lang);
        Some(GrammarRuleItem {
            rule_id: *rule.rule_id(),
            level: *rule.level(),
            apply_to: rule.apply_to().to_vec(),
            title: content.title().to_string(),
            short_description: content.short_description().to_string(),
            md_description: content.md_description().to_string(),
        })
    } else {
        None
    }
}
