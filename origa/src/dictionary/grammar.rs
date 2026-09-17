use std::{collections::HashMap, collections::HashSet, sync::OnceLock};

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

    let json = data
        .grammar_json
        .strip_prefix('\u{FEFF}')
        .unwrap_or(&data.grammar_json);

    let content: GrammarStoreValue =
        serde_json::from_str(json).map_err(|e| OrigaError::GrammarParseError {
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

pub fn get_all_rule_ids() -> HashSet<Ulid> {
    GRAMMAR_RULES
        .get()
        .map(|rules| rules.iter().map(|r| *r.rule_id()).collect())
        .unwrap_or_default()
}

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.get()?.iter().find(|x| x.rule_id() == rule_id)
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
    #[serde(default)]
    keywords: Vec<Vec<String>>,
}

/// A single wrong-usage / correct-usage pair from the structured
/// `nuances` section (schema v2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommonMistake {
    wrong: String,
    correct: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

impl CommonMistake {
    pub fn wrong(&self) -> &str {
        &self.wrong
    }

    pub fn correct(&self) -> &str {
        &self.correct
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    #[cfg(test)]
    pub fn for_test(wrong: &str, correct: &str) -> Self {
        Self {
            wrong: wrong.to_string(),
            correct: correct.to_string(),
            note: None,
        }
    }
}

/// Semantic class of a free-form nuance note (schema v2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NuanceTag {
    Register,
    Variation,
    Collocation,
    Formality,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NuanceNote {
    tag: NuanceTag,
    text: String,
}

impl NuanceNote {
    pub fn tag(&self) -> NuanceTag {
        self.tag
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    #[cfg(test)]
    pub fn for_test(tag: NuanceTag, text: &str) -> Self {
        Self {
            tag,
            text: text.to_string(),
        }
    }
}

/// Structured replacement for the legacy raw-markdown `nuances` blob.
/// Emoji markers are forbidden in the data — presentation is the UI's job.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Nuances {
    #[serde(default)]
    common_mistakes: Vec<CommonMistake>,
    #[serde(default)]
    notes: Vec<NuanceNote>,
}

impl Nuances {
    /// Terminal fallback for [`EMPTY_GRAMMAR_CONTENT`]: all sets empty.
    const EMPTY: Self = Self {
        common_mistakes: Vec::new(),
        notes: Vec::new(),
    };

    pub fn common_mistakes(&self) -> &[CommonMistake] {
        &self.common_mistakes
    }

    pub fn notes(&self) -> &[NuanceNote] {
        &self.notes
    }

    pub fn is_empty(&self) -> bool {
        self.common_mistakes.is_empty() && self.notes.is_empty()
    }
}

/// How a related grammar pattern relates to the current rule (schema v2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RelatedPatternRelation {
    Pair,
    Contrast,
    Derived,
}

/// Reference to another grammar rule, stored by stable `rule_id` so the
/// link survives title edits (schema v2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelatedPattern {
    rule_id: Ulid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation: Option<RelatedPatternRelation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

impl RelatedPattern {
    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn relation(&self) -> Option<RelatedPatternRelation> {
        self.relation
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRuleContent {
    /// Legacy fused form: `pattern（qualifier）`. Kept for already-released
    /// clients reading the grammar store; new code must use [`Self::pattern`].
    title: String,
    /// The bare Japanese pattern without the qualifier parentheses
    /// (`～も`, not `～も（тоже・даже）`). Migrated into the store by
    /// `scripts/add_grammar_pattern.py`; falls back to splitting `title`
    /// for stores predating the migration.
    #[serde(default)]
    pattern: String,
    short_description: String,
    explanation: String,
    how_to_form: String,
    examples: String,
    nuances: Nuances,
    pro_tip: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    related_patterns: Vec<RelatedPattern>,
}

impl GrammarRuleContent {
    /// The bare Japanese pattern (`～も`), with the legacy qualifier
    /// parentheses stripped. Prefers the migrated `pattern` field and
    /// falls back to splitting the legacy `title` for old stores.
    pub fn pattern(&self) -> &str {
        if self.pattern.is_empty() {
            split_legacy_title_pattern(&self.title)
        } else {
            &self.pattern
        }
    }
}

/// Runtime twin of the deploy validator's trailing-qualifier rule: strips
/// TRAILING parenthetical groups of either width from a legacy fused
/// title. Inner groups (`（よ）` optionality) stay — they are part of the
/// pattern, not glosses. Returns a prefix slice of `title`.
fn split_legacy_title_pattern(title: &str) -> &str {
    let mut pattern = title;
    loop {
        let Some(stripped) = strip_trailing_group(pattern.trim_end()) else {
            return pattern.trim_end();
        };
        if stripped.trim_end().is_empty() {
            return title; // a group ate everything: not a qualifier, bail out
        }
        pattern = stripped.trim_end();
    }
}

/// Prefix of `s` without its trailing parenthetical group, full-width or
/// ASCII. `None` when the string does not end with a closed group.
fn strip_trailing_group(s: &str) -> Option<&str> {
    let (closer, opener) = if s.ends_with('）') {
        ('）', '（')
    } else if s.ends_with(')') {
        (')', '(')
    } else {
        return None;
    };
    let body = &s[..s.len() - closer.len_utf8()];
    let open = body.rfind(opener)?;
    Some(&s[..open])
}

/// Terminal fallback for [`GrammarRule::content`]: a rule whose content map
/// is completely empty resolves here instead of panicking. All accessors
/// return empty strings/sets, which callers translate into
/// `GrammarContentNotFound` errors.
static EMPTY_GRAMMAR_CONTENT: GrammarRuleContent = GrammarRuleContent {
    title: String::new(),
    pattern: String::new(),
    short_description: String::new(),
    explanation: String::new(),
    how_to_form: String::new(),
    examples: String::new(),
    nuances: Nuances::EMPTY,
    pro_tip: String::new(),
    warnings: Vec::new(),
    related_patterns: Vec::new(),
};

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
    VerbToMizenkei {},
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

    VerbToNasai {},
    VerbToKudasai {},
    VerbToIrasshai {},

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
            | FormatAction::VerbToMizenkei {}
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
            | FormatAction::VerbToOShimasu {}
            | FormatAction::VerbToNasai {}
            | FormatAction::VerbToKudasai {}
            | FormatAction::VerbToIrasshai {} => FormatActionGroup::Verb,

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
            FormatAction::VerbToMizenkei {},
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
            keywords: vec![],
        }
    }

    #[cfg(test)]
    pub fn new_with_keywords(
        rule_id: Ulid,
        level: JapaneseLevel,
        content: HashMap<NativeLanguage, GrammarRuleContent>,
        format_map: Option<HashMap<PartOfSpeech, Vec<FormatAction>>>,
        keywords: Vec<Vec<String>>,
    ) -> Self {
        Self {
            rule_id,
            level,
            content,
            format_map,
            keywords,
        }
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }

    pub fn content(&self, lang: &NativeLanguage) -> &GrammarRuleContent {
        // The v3 corpus ships all four supported languages for every rule.
        // A missing locale is a data bug, not something to paper over with
        // a silent English substitution: resolve the requested language
        // directly and let the empty terminal surface as
        // `GrammarContentNotFound` to the caller.
        self.content.get(lang).unwrap_or(&EMPTY_GRAMMAR_CONTENT)
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

    pub fn keywords(&self) -> &[Vec<String>] {
        &self.keywords
    }
}

impl GrammarRuleContent {
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        short_description: String,
        explanation: String,
        how_to_form: String,
        examples: String,
        nuances: Nuances,
        pro_tip: String,
        related_patterns: Vec<RelatedPattern>,
    ) -> Self {
        Self {
            pattern: split_legacy_title_pattern(&title).to_string(),
            title,
            short_description,
            explanation,
            how_to_form,
            examples,
            nuances,
            pro_tip,
            warnings: Vec::new(),
            related_patterns,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn short_description(&self) -> &str {
        &self.short_description
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn how_to_form(&self) -> &str {
        &self.how_to_form
    }

    pub fn examples(&self) -> &str {
        &self.examples
    }

    pub fn nuances(&self) -> &Nuances {
        &self.nuances
    }

    pub fn pro_tip(&self) -> &str {
        &self.pro_tip
    }

    pub fn related_patterns(&self) -> &[RelatedPattern] {
        &self.related_patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_rules_should_not_be_loaded_before_init() {
        assert!(!is_grammar_loaded());
    }

    #[test]
    fn pattern_field_wins_over_legacy_title_split() {
        let content = GrammarRuleContent::new(
            "～も（тоже・даже）".to_string(),
            "тоже, также, даже".to_string(),
            String::new(),
            String::new(),
            String::new(),
            Nuances::EMPTY,
            String::new(),
            Vec::new(),
        );
        assert_eq!(content.pattern(), "～も");
    }

    #[test]
    fn pattern_falls_back_to_splitting_legacy_title() {
        let mut content = GrammarRuleContent::new(
            "～の（номинализация）".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            Nuances::EMPTY,
            String::new(),
            Vec::new(),
        );
        content.pattern = String::new(); // store predating the migration
        assert_eq!(content.pattern(), "～の");
    }

    #[test]
    fn pattern_without_qualifier_is_the_title_itself() {
        let content = GrammarRuleContent::new(
            "～は～です".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            Nuances::EMPTY,
            String::new(),
            Vec::new(),
        );
        assert_eq!(content.pattern(), "～は～です");
    }

    #[test]
    fn pattern_fallback_handles_ascii_parens_and_unclosed_groups() {
        let mut content = GrammarRuleContent::new(
            "～たら(if…then)".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            Nuances::EMPTY,
            String::new(),
            Vec::new(),
        );
        content.pattern = String::new();
        assert_eq!(content.pattern(), "～たら");

        let mut unclosed = content.clone();
        unclosed.title = "～（よ）うにも～ない".to_string();
        unclosed.pattern = String::new();
        // Trailing group must be last AND closed; here the paren is inner,
        // so nothing is stripped.
        assert_eq!(unclosed.pattern(), "～（よ）うにも～ない");
    }

    #[test]
    fn pattern_fallback_keeps_degenerate_paren_only_title_intact() {
        // Divergence from the Python validator twin is intentional: a
        // degenerate title that is ONLY a qualifier group yields an empty
        // pattern, which the deploy validator rejects at upload time
        // (fail-loud for data); the runtime fallback bails out and keeps
        // the full title so a legacy store still renders something.
        let mut content = GrammarRuleContent::new(
            "（тоже）".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            Nuances::EMPTY,
            String::new(),
            Vec::new(),
        );
        content.pattern = String::new();
        assert_eq!(content.pattern(), "（тоже）");
    }
}

#[cfg(test)]
mod tests_nuances_v2 {
    use super::*;

    #[test]
    fn nuances_deser_reads_schema_v2_object() {
        // Arrange
        let json = r#"{
            "common_mistakes": [
                {"wrong": "Using を", "correct": "Use が", "note": "subject marker"}
            ],
            "notes": [
                {"tag": "variation", "text": "In casual speech です becomes だ"}
            ]
        }"#;

        // Act
        let nuances: Nuances = serde_json::from_str(json).expect("valid v2 nuances");

        // Assert
        assert_eq!(nuances.common_mistakes().len(), 1);
        assert_eq!(nuances.common_mistakes()[0].wrong(), "Using を");
        assert_eq!(nuances.common_mistakes()[0].note(), Some("subject marker"));
        assert_eq!(nuances.notes().len(), 1);
        assert_eq!(nuances.notes()[0].tag(), NuanceTag::Variation);
        assert!(!nuances.is_empty());
    }

    #[test]
    fn nuances_deser_defaults_missing_lists_to_empty() {
        // Arrange
        let json = r#"{}"#;

        // Act
        let nuances: Nuances = serde_json::from_str(json).expect("empty v2 nuances");

        // Assert
        assert!(nuances.common_mistakes().is_empty());
        assert!(nuances.notes().is_empty());
        assert!(nuances.is_empty());
    }

    #[test]
    fn related_patterns_deser_reads_rule_id_reference() {
        // Arrange
        let json = r#"[{"rule_id": "01G000000000000000G0000000", "relation": "pair", "note": "directional pair"}]"#;

        // Act
        let related: Vec<RelatedPattern> =
            serde_json::from_str(json).expect("valid related_patterns");

        // Assert
        assert_eq!(related.len(), 1);
        assert_eq!(
            related[0].rule_id().to_string(),
            "01G000000000000000G0000000"
        );
        assert_eq!(related[0].relation(), Some(RelatedPatternRelation::Pair));
        assert_eq!(related[0].note(), Some("directional pair"));
    }

    #[test]
    fn grammar_rule_content_v2_serde_roundtrip_preserves_structure() {
        // Arrange
        let content = GrammarRuleContent::new(
            "～は～です".to_string(),
            "Basic topic-predicate pattern".to_string(),
            "Explanation".to_string(),
            "| table |".to_string(),
            "```\n私は学生です。\n```".to_string(),
            Nuances {
                common_mistakes: vec![CommonMistake {
                    wrong: "wrong".to_string(),
                    correct: "correct".to_string(),
                    note: None,
                }],
                notes: vec![NuanceNote {
                    tag: NuanceTag::Register,
                    text: "casual register note".to_string(),
                }],
            },
            "Pro tip".to_string(),
            vec![RelatedPattern {
                rule_id: Ulid::nil(),
                relation: None,
                note: None,
            }],
        );

        // Act
        let json = serde_json::to_string(&content).expect("serialize v2 content");
        let back: GrammarRuleContent = serde_json::from_str(&json).expect("deserialize v2 content");

        // Assert
        assert_eq!(back.title(), content.title());
        assert_eq!(back.nuances(), content.nuances());
        assert_eq!(back.related_patterns(), content.related_patterns());
        assert!(back.warnings().is_empty());
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

#[cfg(test)]
mod tests_language_fallback {
    use super::*;

    fn rule_with_content() -> GrammarRule {
        let json = r#"{
            "rule_id": "01J9TESTTESTTESTTESTTESTTE",
            "level": "N5",
            "content": {
                "English": {
                    "title": "title-en",
                    "short_description": "short-en",
                    "explanation": "explanation-en",
                    "how_to_form": "form-en",
                    "examples": "[]",
                    "nuances": {},
                    "pro_tip": ""
                },
                "Russian": {
                    "title": "заголовок",
                    "short_description": "кратко",
                    "explanation": "объяснение",
                    "how_to_form": "образование",
                    "examples": "[]",
                    "nuances": {},
                    "pro_tip": ""
                }
            }
        }"#;
        serde_json::from_str(json).expect("valid rule json")
    }

    #[test]
    fn locale_without_content_resolves_to_empty_terminal() {
        let rule = rule_with_content();
        // The v3 corpus ships all four locales; a missing one is a data
        // bug surfaced as empty content (GrammarContentNotFound upstream),
        // never a silent English substitution.
        assert_eq!(rule.content(&NativeLanguage::Korean).title(), "");
        assert_eq!(rule.content(&NativeLanguage::Vietnamese).title(), "");
        assert_eq!(rule.content(&NativeLanguage::Russian).title(), "заголовок");
    }

    #[test]
    fn korean_and_vietnamese_content_resolve_directly() {
        // The merged grammar_v2 corpus ships Korean/Vietnamese content maps;
        // the resolution chain must prefer them over the English fallback.
        let json = r#"{
            "rule_id": "01J9TESTTESTTESTTESTTESTTE",
            "level": "N5",
            "content": {
                "English": {
                    "title": "title-en",
                    "short_description": "short-en",
                    "explanation": "explanation-en",
                    "how_to_form": "form-en",
                    "examples": "[]",
                    "nuances": {},
                    "pro_tip": ""
                },
                "Korean": {
                    "title": "제목",
                    "short_description": "요약",
                    "explanation": "설명",
                    "how_to_form": "형성",
                    "examples": "[]",
                    "nuances": {},
                    "pro_tip": ""
                },
                "Vietnamese": {
                    "title": "tiêu đề",
                    "short_description": "tóm tắt",
                    "explanation": "giải thích",
                    "how_to_form": "cách tạo",
                    "examples": "[]",
                    "nuances": {},
                    "pro_tip": ""
                }
            }
        }"#;
        let rule: GrammarRule = serde_json::from_str(json).expect("valid rule json");
        assert_eq!(rule.content(&NativeLanguage::Korean).title(), "제목");
        assert_eq!(rule.content(&NativeLanguage::Vietnamese).title(), "tiêu đề");
        assert_eq!(rule.content(&NativeLanguage::English).title(), "title-en");
    }

    #[test]
    fn empty_content_map_resolves_to_empty_static() {
        let json = r#"{
            "rule_id": "01J9TESTTESTTESTTESTTESTT0",
            "level": "N5",
            "content": {}
        }"#;
        let rule: GrammarRule = serde_json::from_str(json).expect("valid rule json");
        // Terminal case: an empty content map must not panic; every accessor
        // yields empty text, which the get_content! macro surfaces as
        // GrammarContentNotFound.
        assert_eq!(rule.content(&NativeLanguage::English).title(), "");
        assert_eq!(rule.content(&NativeLanguage::Korean).explanation(), "");
    }
}
