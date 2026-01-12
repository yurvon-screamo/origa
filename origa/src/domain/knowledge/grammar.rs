use crate::domain::{
    OrigaError,
    grammar::GrammarRuleInfo,
    tokenizer::PartOfSpeech,
    value_objects::{Answer, NativeLanguage, Question},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleCard {
    rule_id: Ulid,
    title: Question,
    description: Answer,
    apply_to: Vec<PartOfSpeech>,
}

impl GrammarRuleCard {
    pub fn new(rule_info: GrammarRuleInfo, lang: &NativeLanguage) -> Result<Self, OrigaError> {
        Ok(Self {
            rule_id: rule_info.rule_id().to_owned(),
            title: Question::new(rule_info.content(lang).title().to_string())?,
            description: Answer::new(rule_info.content(lang).md_description().to_string())?,
            apply_to: rule_info.apply_to().to_vec(),
        })
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn title(&self) -> &Question {
        &self.title
    }

    pub fn description(&self) -> &Answer {
        &self.description
    }

    pub fn apply_to(&self) -> &[PartOfSpeech] {
        &self.apply_to
    }
}
