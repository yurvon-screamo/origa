use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    value_objects::{JapaneseLevel, NativeLanguage},
};

const JLPT_N1_RAW: &str = include_str!("./jltp_n1.json");
const JLPT_N2_RAW: &str = include_str!("./jltp_n2.json");
const JLPT_N3_RAW: &str = include_str!("./jltp_n3.json");
const JLPT_N4_RAW: &str = include_str!("./jltp_n4.json");
const JLPT_N5_RAW: &str = include_str!("./jltp_n5.json");

pub fn load_jlpt_n1() -> Result<WellKnownSet, KeikakuError> {
    load(JLPT_N1_RAW)
}

pub fn load_jlpt_n2() -> Result<WellKnownSet, KeikakuError> {
    load(JLPT_N2_RAW)
}

pub fn load_jlpt_n3() -> Result<WellKnownSet, KeikakuError> {
    load(JLPT_N3_RAW)
}

pub fn load_jlpt_n4() -> Result<WellKnownSet, KeikakuError> {
    load(JLPT_N4_RAW)
}

pub fn load_jlpt_n5() -> Result<WellKnownSet, KeikakuError> {
    load(JLPT_N5_RAW)
}

fn load(raw: &str) -> Result<WellKnownSet, KeikakuError> {
    serde_json::from_str(raw).map_err(|e| KeikakuError::WellKnownSetError {
        reason: format!("Error parse stored value: {e}"),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSet {
    level: JapaneseLevel,
    words: Vec<String>,
    content: HashMap<NativeLanguage, WellKnownSetContent>,
}

impl WellKnownSet {
    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn content(&self, lang: &NativeLanguage) -> &WellKnownSetContent {
        &self.content[lang]
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSetContent {
    title: String,
    description: String,
}

impl WellKnownSetContent {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
