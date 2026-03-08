use crate::domain::{
    get_rule_by_id,
    tokenizer::PartOfSpeech,
    value_objects::{Answer, NativeLanguage, Question},
    OrigaError, FALLBACK_ANSWER,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleCard {
    rule_id: Ulid,
}

impl GrammarRuleCard {
    pub fn new(rule_id: Ulid) -> Result<Self, OrigaError> {
        get_rule_by_id(&rule_id).ok_or_else(|| OrigaError::RepositoryError {
            reason: format!("Grammar rule {} not found", rule_id),
        })?;
        Ok(Self { rule_id })
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn title(&self, lang: &NativeLanguage) -> Question {
        get_rule_by_id(&self.rule_id)
            .map(|rule| {
                Question::new(rule.content(lang).title().to_string())
                    .unwrap_or_else(|_| Question::new(FALLBACK_ANSWER.to_string()).unwrap())
            })
            .unwrap_or_else(|| Question::new(FALLBACK_ANSWER.to_string()).unwrap())
    }

    pub fn description(&self, lang: &NativeLanguage) -> Answer {
        get_rule_by_id(&self.rule_id)
            .map(|rule| {
                Answer::new(rule.content(lang).md_description().to_string())
                    .unwrap_or_else(|_| Answer::new(FALLBACK_ANSWER.to_string()).unwrap())
            })
            .unwrap_or_else(|| Answer::new(FALLBACK_ANSWER.to_string()).unwrap())
    }

    pub fn apply_to(&self) -> Vec<PartOfSpeech> {
        get_rule_by_id(&self.rule_id)
            .map(|rule| rule.apply_to())
            .unwrap_or_default()
    }
}

impl GrammarRuleCard {
    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            rule_id: Ulid::new(),
        }
    }
}
