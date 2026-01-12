mod adjective_naru;
mod adjective_past;
mod nda;
mod verb_forms;
mod verb_hou_ga_ii;
mod verb_mada_te_inai;
mod verb_masenka;
mod verb_mashou;
mod verb_mashouka;
mod verb_naide_kudasai;
mod verb_ni_iku;
mod verb_sugiru;
mod verb_ta_koto_ga_aru;
mod verb_tai;
mod verb_tari;
mod verb_te_iru;
mod verb_te_kudasai;
mod verb_te_wa_ikemasen;
mod verb_tsumori;

use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    OrigaError,
    grammar::verb_mashou::VerbMashouRule,
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

pub static GRAMMAR_RULES: LazyLock<Vec<Box<dyn GrammarRule>>> = LazyLock::new(|| {
    vec![
        // // Существующие правила
        // Box::new(んだRule {}),
        // Box::new(AdjectivePastRule {}),
        // // Глагольные формы
        // Box::new(VerbMasenkaRule {}),
        Box::new(VerbMashouRule::new()),
        // Box::new(VerbMashoukaRule {}),
        // Box::new(VerbTeKudasaiRule {}),
        // Box::new(VerbTeWaIkemasenRule {}),
        // Box::new(VerbTeIruRule {}),
        // Box::new(VerbNiIkuRule {}),
        // Box::new(VerbNaideKudasaiRule {}),
        // Box::new(VerbMadaTeInaiRule {}),
        // Box::new(VerbTaiRule {}),
        // Box::new(VerbTariRule {}),
        // Box::new(VerbTaKotoGaAruRule {}),
        // Box::new(VerbSugiruRule {}),
        // Box::new(VerbHouGaIiRule {}),
        // // Конструкции с прилагательными
        // Box::new(AdjectiveNaruRule {}),
        // // Конструкции намерения
        // Box::new(ConstructionTsumoriRule {}),
    ]
});

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static dyn GrammarRule> {
    GRAMMAR_RULES
        .iter()
        .find(|x| x.info().rule_id() == rule_id)
        .map(|x| x.as_ref())
}

pub trait GrammarRule: Send + Sync {
    fn info(&self) -> &GrammarRuleInfo;
    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleInfo {
    rule_id: Ulid,
    level: JapaneseLevel,
    apply_to: Vec<PartOfSpeech>,
    content: HashMap<NativeLanguage, GrammarRuleContent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleContent {
    title: String,
    short_description: String,
    md_description: String,
}

impl GrammarRuleInfo {
    pub fn new(
        rule_id: Ulid,
        level: JapaneseLevel,
        apply_to: Vec<PartOfSpeech>,
        content: HashMap<NativeLanguage, GrammarRuleContent>,
    ) -> Self {
        Self {
            rule_id,
            level,
            apply_to,
            content,
        }
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }

    pub fn apply_to(&self) -> &[PartOfSpeech] {
        &self.apply_to
    }

    pub fn content(&self, lang: &NativeLanguage) -> &GrammarRuleContent {
        &self.content[lang]
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
