use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, PartOfSpeech};

pub static GRAMMAR_RULES: OnceLock<Vec<GrammarRule>> = OnceLock::new();

#[derive(Deserialize)]
struct GrammarStoreValue {
    grammar: Vec<GrammarRule>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GrammarData {
    pub grammar_json: String,
}

pub fn init_grammar(data: GrammarData) -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    let content: GrammarStoreValue =
        serde_json::from_str(&data.grammar_json).map_err(|e| OrigaError::GrammarParseError {
            reason: format!("Failed to parse grammar.json: {}", e),
        })?;

    GRAMMAR_RULES
        .set(content.grammar)
        .map_err(|_| OrigaError::GrammarParseError {
            reason: "Failed to set grammar rules".to_string(),
        })
}

pub fn is_grammar_loaded() -> bool {
    GRAMMAR_RULES.get().is_some()
}

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.get()?.iter().find(|x| x.rule_id() == rule_id)
}

pub fn get_rule_by_title(title: &str) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES
        .get()?
        .iter()
        .find(|x| x.content.values().any(|c| c.title() == title))
}

pub fn iter_grammar_rules() -> impl Iterator<Item = &'static GrammarRule> {
    GRAMMAR_RULES
        .get()
        .into_iter()
        .flat_map(|rules| rules.iter())
}

pub fn get_rules_by_level(level: &JapaneseLevel) -> Vec<&'static GrammarRule> {
    GRAMMAR_RULES
        .get()
        .map(|rules| rules.iter().filter(|r| r.level() == level).collect())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    rule_id: Ulid,
    level: JapaneseLevel,
    content: HashMap<NativeLanguage, GrammarRuleContent>,
    format_map: Option<HashMap<PartOfSpeech, Vec<FormatAction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRuleContent {
    title: String,
    short_description: String,
    md_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatAction {
    AdjectiveRemovePostfix {},
    AdjectiveToKunai {},
    AdjectiveToKatta {},
    AdjectiveToKunakatta {},
    AdjectiveToKute {},
    AdjectiveToKu {},
    AdjectiveToKereba {},
    AdjectiveToSou {},
    AdjectiveToSugiru {},
    AdjectiveToNa {},
    AdjectiveToDe {},
    AdjectiveToNara {},
    AdjectiveToSouNa {},
    AdjectiveToNasasou {},
    AdjectiveToGaru {},

    VerbToTeForm {},
    VerbToMainView {},
    VerbToMasu {},
    VerbToMasen {},
    VerbToMashita {},
    VerbToMasenDeshita {},
    VerbToMashou {},
    VerbToStem {},
    VerbToTa {},
    VerbToNai {},
    VerbToTara {},
    VerbToBa {},
    VerbToPotential {},
    VerbToPassive {},
    VerbToCausative {},
    VerbToCausativePassive {},
    VerbToImperative {},
    VerbToVolitional {},
    VerbToSou {},
    VerbToZu {},
    VerbToTai {},
    VerbToYasui {},
    VerbToNikui {},
    VerbToSugiru {},
    VerbToChau {},
    VerbToToku {},
    VerbToTeru {},
    VerbToONinarimasu {},
    VerbToOKudasai {},
    VerbToOShimasu {},

    ReplacePostfix {
        old_postfix: String,
        new_postfix: String,
    },
    AddPostfix {
        postfix: String,
    },
    RemovePostfix {
        postfix: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatActionGroup {
    Verb,
    IAdjective,
    NaAdjective,
    Universal,
}

impl FormatAction {
    pub fn group(&self) -> FormatActionGroup {
        match self {
            FormatAction::AdjectiveRemovePostfix {}
            | FormatAction::AdjectiveToKunai {}
            | FormatAction::AdjectiveToKatta {}
            | FormatAction::AdjectiveToKunakatta {}
            | FormatAction::AdjectiveToKute {}
            | FormatAction::AdjectiveToKu {}
            | FormatAction::AdjectiveToKereba {}
            | FormatAction::AdjectiveToSou {}
            | FormatAction::AdjectiveToSugiru {} => FormatActionGroup::IAdjective,

            FormatAction::AdjectiveToNa {}
            | FormatAction::AdjectiveToDe {}
            | FormatAction::AdjectiveToNara {}
            | FormatAction::AdjectiveToSouNa {}
            | FormatAction::AdjectiveToNasasou {}
            | FormatAction::AdjectiveToGaru {} => FormatActionGroup::NaAdjective,

            FormatAction::VerbToTeForm {}
            | FormatAction::VerbToMainView {}
            | FormatAction::VerbToMasu {}
            | FormatAction::VerbToMasen {}
            | FormatAction::VerbToMashita {}
            | FormatAction::VerbToMasenDeshita {}
            | FormatAction::VerbToMashou {}
            | FormatAction::VerbToStem {}
            | FormatAction::VerbToTa {}
            | FormatAction::VerbToNai {}
            | FormatAction::VerbToTara {}
            | FormatAction::VerbToBa {}
            | FormatAction::VerbToPotential {}
            | FormatAction::VerbToPassive {}
            | FormatAction::VerbToCausative {}
            | FormatAction::VerbToCausativePassive {}
            | FormatAction::VerbToImperative {}
            | FormatAction::VerbToVolitional {}
            | FormatAction::VerbToSou {}
            | FormatAction::VerbToZu {}
            | FormatAction::VerbToTai {}
            | FormatAction::VerbToYasui {}
            | FormatAction::VerbToNikui {}
            | FormatAction::VerbToSugiru {}
            | FormatAction::VerbToChau {}
            | FormatAction::VerbToToku {}
            | FormatAction::VerbToTeru {}
            | FormatAction::VerbToONinarimasu {}
            | FormatAction::VerbToOKudasai {}
            | FormatAction::VerbToOShimasu {} => FormatActionGroup::Verb,

            FormatAction::ReplacePostfix { .. }
            | FormatAction::AddPostfix { .. }
            | FormatAction::RemovePostfix { .. } => FormatActionGroup::Universal,
        }
    }

    pub fn all_verb_actions() -> &'static [FormatAction] {
        &[
            FormatAction::VerbToTeForm {},
            FormatAction::VerbToMainView {},
            FormatAction::VerbToMasu {},
            FormatAction::VerbToMasen {},
            FormatAction::VerbToMashita {},
            FormatAction::VerbToMasenDeshita {},
            FormatAction::VerbToMashou {},
            FormatAction::VerbToStem {},
            FormatAction::VerbToTa {},
            FormatAction::VerbToNai {},
            FormatAction::VerbToTara {},
            FormatAction::VerbToBa {},
            FormatAction::VerbToPotential {},
            FormatAction::VerbToPassive {},
            FormatAction::VerbToCausative {},
            FormatAction::VerbToCausativePassive {},
            FormatAction::VerbToImperative {},
            FormatAction::VerbToVolitional {},
            FormatAction::VerbToSou {},
            FormatAction::VerbToZu {},
            FormatAction::VerbToTai {},
            FormatAction::VerbToYasui {},
            FormatAction::VerbToNikui {},
            FormatAction::VerbToSugiru {},
            FormatAction::VerbToChau {},
            FormatAction::VerbToToku {},
            FormatAction::VerbToTeru {},
            FormatAction::VerbToONinarimasu {},
            FormatAction::VerbToOKudasai {},
            FormatAction::VerbToOShimasu {},
        ]
    }

    pub fn all_i_adjective_actions() -> &'static [FormatAction] {
        &[
            FormatAction::AdjectiveRemovePostfix {},
            FormatAction::AdjectiveToKunai {},
            FormatAction::AdjectiveToKatta {},
            FormatAction::AdjectiveToKunakatta {},
            FormatAction::AdjectiveToKute {},
            FormatAction::AdjectiveToKu {},
            FormatAction::AdjectiveToKereba {},
            FormatAction::AdjectiveToSou {},
            FormatAction::AdjectiveToSugiru {},
        ]
    }

    pub fn all_na_adjective_actions() -> &'static [FormatAction] {
        &[
            FormatAction::AdjectiveToNa {},
            FormatAction::AdjectiveToDe {},
            FormatAction::AdjectiveToNara {},
            FormatAction::AdjectiveToSouNa {},
            FormatAction::AdjectiveToNasasou {},
            FormatAction::AdjectiveToGaru {},
        ]
    }

    /// Returns all FormatActions from the same group, excluding self.
    /// Returns empty Vec for Universal group.
    pub fn mutation_alternatives(&self) -> Vec<&'static FormatAction> {
        let all: &[FormatAction] = match self.group() {
            FormatActionGroup::Verb => Self::all_verb_actions(),
            FormatActionGroup::IAdjective => Self::all_i_adjective_actions(),
            FormatActionGroup::NaAdjective => Self::all_na_adjective_actions(),
            FormatActionGroup::Universal => return Vec::new(),
        };
        all.iter().filter(|a| !std::ptr::eq(*a, self)).collect()
    }
}

impl GrammarRule {
    #[cfg(test)]
    pub fn new(
        rule_id: Ulid,
        level: JapaneseLevel,
        content: HashMap<NativeLanguage, GrammarRuleContent>,
        format_map: Option<HashMap<PartOfSpeech, Vec<FormatAction>>>,
    ) -> Self {
        Self {
            rule_id,
            level,
            content,
            format_map,
        }
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }

    pub fn content(&self, lang: &NativeLanguage) -> &GrammarRuleContent {
        &self.content[lang]
    }

    pub fn apply_to(&self) -> Vec<PartOfSpeech> {
        match &self.format_map {
            Some(map) => map.keys().cloned().collect(),
            None => vec![],
        }
    }

    pub(crate) fn format_map(&self) -> Option<&HashMap<PartOfSpeech, Vec<FormatAction>>> {
        self.format_map.as_ref()
    }

    pub fn format_actions_for_pos(&self, pos: &PartOfSpeech) -> Option<&Vec<FormatAction>> {
        self.format_map.as_ref()?.get(pos)
    }

    pub fn has_format_map(&self) -> bool {
        self.format_map.is_some()
    }
}

impl GrammarRuleContent {
    pub fn new(title: String, short_description: String, md_description: String) -> Self {
        Self {
            title,
            short_description,
            md_description,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn short_description(&self) -> &str {
        &self.short_description
    }

    pub fn md_description(&self) -> &str {
        &self.md_description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_rules_should_not_be_loaded_before_init() {
        assert!(!is_grammar_loaded());
    }
}

#[cfg(test)]
mod tests_format_action_group {
    use super::*;

    #[test]
    fn verb_actions_are_classified_correctly() {
        assert_eq!(FormatAction::VerbToMasu {}.group(), FormatActionGroup::Verb);
        assert_eq!(
            FormatAction::VerbToTeForm {}.group(),
            FormatActionGroup::Verb
        );
        assert_eq!(FormatAction::VerbToNai {}.group(), FormatActionGroup::Verb);
    }

    #[test]
    fn i_adjective_actions_are_classified_correctly() {
        assert_eq!(
            FormatAction::AdjectiveToKunai {}.group(),
            FormatActionGroup::IAdjective
        );
        assert_eq!(
            FormatAction::AdjectiveToKatta {}.group(),
            FormatActionGroup::IAdjective
        );
    }

    #[test]
    fn na_adjective_actions_are_classified_correctly() {
        assert_eq!(
            FormatAction::AdjectiveToNa {}.group(),
            FormatActionGroup::NaAdjective
        );
        assert_eq!(
            FormatAction::AdjectiveToDe {}.group(),
            FormatActionGroup::NaAdjective
        );
    }

    #[test]
    fn universal_actions_are_classified_correctly() {
        assert_eq!(
            FormatAction::ReplacePostfix {
                old_postfix: "a".into(),
                new_postfix: "b".into()
            }
            .group(),
            FormatActionGroup::Universal
        );
    }

    #[test]
    fn mutation_alternatives_excludes_self() {
        let action = FormatAction::VerbToMasu {};
        let alternatives = action.mutation_alternatives();
        assert!(!alternatives.iter().any(|a| std::ptr::eq(*a, &action)));
        assert!(!alternatives.is_empty());
    }

    #[test]
    fn universal_has_no_mutation_alternatives() {
        let action = FormatAction::AddPostfix {
            postfix: "test".into(),
        };
        assert!(action.mutation_alternatives().is_empty());
    }

    #[test]
    fn all_verb_actions_count() {
        assert!(FormatAction::all_verb_actions().len() >= 25);
    }

    #[test]
    fn group_matches_all_lists_exhaustively() {
        for action in FormatAction::all_verb_actions() {
            assert_eq!(action.group(), FormatActionGroup::Verb);
        }
        for action in FormatAction::all_i_adjective_actions() {
            assert_eq!(action.group(), FormatActionGroup::IAdjective);
        }
        for action in FormatAction::all_na_adjective_actions() {
            assert_eq!(action.group(), FormatActionGroup::NaAdjective);
        }
    }
}
