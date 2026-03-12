use crate::domain::{
    get_rule_by_id,
    tokenizer::PartOfSpeech,
    value_objects::{Answer, NativeLanguage, Question},
    OrigaError,
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

    pub fn title(&self, lang: &NativeLanguage) -> Result<Question, OrigaError> {
        let rule = get_rule_by_id(&self.rule_id).ok_or(OrigaError::GrammarRuleNotFound {
            rule_id: self.rule_id,
        })?;

        let title = rule.content(lang).title();
        if title.is_empty() {
            return Err(OrigaError::GrammarContentNotFound {
                rule_id: self.rule_id,
                lang: *lang,
            });
        }

        Question::new(title.to_string()).map_err(|e| OrigaError::InvalidQuestion {
            reason: e.to_string(),
        })
    }

    pub fn description(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        let rule = get_rule_by_id(&self.rule_id).ok_or(OrigaError::GrammarRuleNotFound {
            rule_id: self.rule_id,
        })?;

        let desc = rule.content(lang).md_description();
        if desc.is_empty() {
            return Err(OrigaError::GrammarContentNotFound {
                rule_id: self.rule_id,
                lang: *lang,
            });
        }

        Answer::new(desc.to_string()).map_err(|e| OrigaError::InvalidAnswer {
            reason: e.to_string(),
        })
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
