use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::domain::{
    OrigaError,
    value_objects::{JapaneseLevel, NativeLanguage},
};

pub static WELL_KNOWN_SETS: OnceLock<HashMap<WellKnownSets, WellKnownSet>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WellKnownSets {
    JlptN1,
    JlptN2,
    JlptN3,
    JlptN4,
    JlptN5,
    MigiiN5Lesson1,
    MigiiN5Lesson2,
    MigiiN5Lesson3,
    MigiiN5Lesson4,
    MigiiN5Lesson5,
    MigiiN5Lesson6,
    MigiiN5Lesson7,
    MigiiN5Lesson8,
    MigiiN5Lesson9,
    MigiiN5Lesson10,
    MigiiN5Lesson11,
    MigiiN5Lesson12,
    MigiiN5Lesson13,
    MigiiN5Lesson14,
    MigiiN5Lesson15,
    MigiiN5Lesson16,
    MigiiN5Lesson17,
    MigiiN5Lesson18,
    MigiiN5Lesson19,
    MigiiN5Lesson20,
    MigiiN4Lesson1,
    MigiiN4Lesson2,
    MigiiN4Lesson3,
    MigiiN4Lesson4,
    MigiiN4Lesson5,
    MigiiN4Lesson6,
    MigiiN4Lesson7,
    MigiiN4Lesson8,
    MigiiN4Lesson9,
    MigiiN4Lesson10,
    MigiiN4Lesson11,
    MigiiN3Lesson1,
    MigiiN3Lesson2,
    MigiiN3Lesson3,
    MigiiN3Lesson4,
    MigiiN3Lesson5,
    MigiiN3Lesson6,
    MigiiN3Lesson7,
    MigiiN3Lesson8,
    MigiiN3Lesson9,
    MigiiN3Lesson10,
    MigiiN3Lesson11,
    MigiiN3Lesson12,
    MigiiN3Lesson13,
    MigiiN3Lesson14,
    MigiiN3Lesson15,
    MigiiN3Lesson16,
    MigiiN3Lesson17,
    MigiiN3Lesson18,
    MigiiN3Lesson19,
    MigiiN3Lesson20,
    MigiiN3Lesson21,
    MigiiN3Lesson22,
    MigiiN3Lesson23,
    MigiiN3Lesson24,
    MigiiN3Lesson25,
    MigiiN3Lesson26,
    MigiiN3Lesson27,
    MigiiN3Lesson28,
    MigiiN3Lesson29,
    MigiiN3Lesson30,
    MigiiN3Lesson31,
    MigiiN2Lesson1,
    MigiiN2Lesson2,
    MigiiN2Lesson3,
    MigiiN2Lesson4,
    MigiiN2Lesson5,
    MigiiN2Lesson6,
    MigiiN2Lesson7,
    MigiiN2Lesson8,
    MigiiN2Lesson9,
    MigiiN2Lesson10,
    MigiiN2Lesson11,
    MigiiN2Lesson12,
    MigiiN2Lesson13,
    MigiiN2Lesson14,
    MigiiN2Lesson15,
    MigiiN2Lesson16,
    MigiiN2Lesson17,
    MigiiN2Lesson18,
    MigiiN2Lesson19,
    MigiiN2Lesson20,
    MigiiN2Lesson21,
    MigiiN2Lesson22,
    MigiiN2Lesson23,
    MigiiN2Lesson24,
    MigiiN2Lesson25,
    MigiiN2Lesson26,
    MigiiN2Lesson27,
    MigiiN2Lesson28,
    MigiiN2Lesson29,
    MigiiN2Lesson30,
    MigiiN2Lesson31,
    MigiiN1Lesson1,
    MigiiN1Lesson2,
    MigiiN1Lesson3,
    MigiiN1Lesson4,
    MigiiN1Lesson5,
    MigiiN1Lesson6,
    MigiiN1Lesson7,
    MigiiN1Lesson8,
    MigiiN1Lesson9,
    MigiiN1Lesson10,
    MigiiN1Lesson11,
    MigiiN1Lesson12,
    MigiiN1Lesson13,
    MigiiN1Lesson14,
    MigiiN1Lesson15,
    MigiiN1Lesson16,
    MigiiN1Lesson17,
    MigiiN1Lesson18,
    MigiiN1Lesson19,
    MigiiN1Lesson20,
    MigiiN1Lesson21,
    MigiiN1Lesson22,
    MigiiN1Lesson23,
    MigiiN1Lesson24,
    MigiiN1Lesson25,
    MigiiN1Lesson26,
    MigiiN1Lesson27,
    MigiiN1Lesson28,
    MigiiN1Lesson29,
    MigiiN1Lesson30,
    MigiiN1Lesson31,
    MigiiN1Lesson32,
    MigiiN1Lesson33,
    MigiiN1Lesson34,
    MigiiN1Lesson35,
    MigiiN1Lesson36,
    MigiiN1Lesson37,
    MigiiN1Lesson38,
    MigiiN1Lesson39,
    MigiiN1Lesson40,
    MigiiN1Lesson41,
    MigiiN1Lesson42,
    MigiiN1Lesson43,
    MigiiN1Lesson44,
    MigiiN1Lesson45,
    MigiiN1Lesson46,
    MigiiN1Lesson47,
    MigiiN1Lesson48,
    MigiiN1Lesson49,
    MigiiN1Lesson50,
    MigiiN1Lesson51,
    MigiiN1Lesson52,
    MigiiN1Lesson53,
    MigiiN1Lesson54,
    MigiiN1Lesson55,
    MigiiN1Lesson56,
}

pub struct WellKnownSetData {
    pub jlpt_n1: String,
    pub jlpt_n2: String,
    pub jlpt_n3: String,
    pub jlpt_n4: String,
    pub jlpt_n5: String,
    pub migii_n5: Vec<String>,
    pub migii_n4: Vec<String>,
    pub migii_n3: Vec<String>,
    pub migii_n2: Vec<String>,
    pub migii_n1: Vec<String>,
}

pub fn init_well_known_sets(data: WellKnownSetData) -> Result<(), OrigaError> {
    let mut sets = HashMap::new();

    let parse_set = |json: &str, set_type: WellKnownSets| -> Result<WellKnownSet, OrigaError> {
        let set: WellKnownSet =
            serde_json::from_str(json).map_err(|e| OrigaError::WellKnownSetParseError {
                reason: format!("Error parsing {:?}: {}", set_type, e),
            })?;
        Ok(set)
    };

    sets.insert(
        WellKnownSets::JlptN1,
        parse_set(&data.jlpt_n1, WellKnownSets::JlptN1)?,
    );
    sets.insert(
        WellKnownSets::JlptN2,
        parse_set(&data.jlpt_n2, WellKnownSets::JlptN2)?,
    );
    sets.insert(
        WellKnownSets::JlptN3,
        parse_set(&data.jlpt_n3, WellKnownSets::JlptN3)?,
    );
    sets.insert(
        WellKnownSets::JlptN4,
        parse_set(&data.jlpt_n4, WellKnownSets::JlptN4)?,
    );
    sets.insert(
        WellKnownSets::JlptN5,
        parse_set(&data.jlpt_n5, WellKnownSets::JlptN5)?,
    );

    for (i, json) in data.migii_n5.iter().enumerate() {
        let set_type = match i {
            0 => WellKnownSets::MigiiN5Lesson1,
            1 => WellKnownSets::MigiiN5Lesson2,
            2 => WellKnownSets::MigiiN5Lesson3,
            3 => WellKnownSets::MigiiN5Lesson4,
            4 => WellKnownSets::MigiiN5Lesson5,
            5 => WellKnownSets::MigiiN5Lesson6,
            6 => WellKnownSets::MigiiN5Lesson7,
            7 => WellKnownSets::MigiiN5Lesson8,
            8 => WellKnownSets::MigiiN5Lesson9,
            9 => WellKnownSets::MigiiN5Lesson10,
            10 => WellKnownSets::MigiiN5Lesson11,
            11 => WellKnownSets::MigiiN5Lesson12,
            12 => WellKnownSets::MigiiN5Lesson13,
            13 => WellKnownSets::MigiiN5Lesson14,
            14 => WellKnownSets::MigiiN5Lesson15,
            15 => WellKnownSets::MigiiN5Lesson16,
            16 => WellKnownSets::MigiiN5Lesson17,
            17 => WellKnownSets::MigiiN5Lesson18,
            18 => WellKnownSets::MigiiN5Lesson19,
            19 => WellKnownSets::MigiiN5Lesson20,
            _ => continue,
        };
        sets.insert(set_type, parse_set(json, set_type)?);
    }

    for (i, json) in data.migii_n4.iter().enumerate() {
        let set_type = match i {
            0 => WellKnownSets::MigiiN4Lesson1,
            1 => WellKnownSets::MigiiN4Lesson2,
            2 => WellKnownSets::MigiiN4Lesson3,
            3 => WellKnownSets::MigiiN4Lesson4,
            4 => WellKnownSets::MigiiN4Lesson5,
            5 => WellKnownSets::MigiiN4Lesson6,
            6 => WellKnownSets::MigiiN4Lesson7,
            7 => WellKnownSets::MigiiN4Lesson8,
            8 => WellKnownSets::MigiiN4Lesson9,
            9 => WellKnownSets::MigiiN4Lesson10,
            10 => WellKnownSets::MigiiN4Lesson11,
            _ => continue,
        };
        sets.insert(set_type, parse_set(json, set_type)?);
    }

    for (i, json) in data.migii_n3.iter().enumerate() {
        let set_type = match i {
            0 => WellKnownSets::MigiiN3Lesson1,
            1 => WellKnownSets::MigiiN3Lesson2,
            2 => WellKnownSets::MigiiN3Lesson3,
            3 => WellKnownSets::MigiiN3Lesson4,
            4 => WellKnownSets::MigiiN3Lesson5,
            5 => WellKnownSets::MigiiN3Lesson6,
            6 => WellKnownSets::MigiiN3Lesson7,
            7 => WellKnownSets::MigiiN3Lesson8,
            8 => WellKnownSets::MigiiN3Lesson9,
            9 => WellKnownSets::MigiiN3Lesson10,
            10 => WellKnownSets::MigiiN3Lesson11,
            11 => WellKnownSets::MigiiN3Lesson12,
            12 => WellKnownSets::MigiiN3Lesson13,
            13 => WellKnownSets::MigiiN3Lesson14,
            14 => WellKnownSets::MigiiN3Lesson15,
            15 => WellKnownSets::MigiiN3Lesson16,
            16 => WellKnownSets::MigiiN3Lesson17,
            17 => WellKnownSets::MigiiN3Lesson18,
            18 => WellKnownSets::MigiiN3Lesson19,
            19 => WellKnownSets::MigiiN3Lesson20,
            20 => WellKnownSets::MigiiN3Lesson21,
            21 => WellKnownSets::MigiiN3Lesson22,
            22 => WellKnownSets::MigiiN3Lesson23,
            23 => WellKnownSets::MigiiN3Lesson24,
            24 => WellKnownSets::MigiiN3Lesson25,
            25 => WellKnownSets::MigiiN3Lesson26,
            26 => WellKnownSets::MigiiN3Lesson27,
            27 => WellKnownSets::MigiiN3Lesson28,
            28 => WellKnownSets::MigiiN3Lesson29,
            29 => WellKnownSets::MigiiN3Lesson30,
            30 => WellKnownSets::MigiiN3Lesson31,
            _ => continue,
        };
        sets.insert(set_type, parse_set(json, set_type)?);
    }

    for (i, json) in data.migii_n2.iter().enumerate() {
        let set_type = match i {
            0 => WellKnownSets::MigiiN2Lesson1,
            1 => WellKnownSets::MigiiN2Lesson2,
            2 => WellKnownSets::MigiiN2Lesson3,
            3 => WellKnownSets::MigiiN2Lesson4,
            4 => WellKnownSets::MigiiN2Lesson5,
            5 => WellKnownSets::MigiiN2Lesson6,
            6 => WellKnownSets::MigiiN2Lesson7,
            7 => WellKnownSets::MigiiN2Lesson8,
            8 => WellKnownSets::MigiiN2Lesson9,
            9 => WellKnownSets::MigiiN2Lesson10,
            10 => WellKnownSets::MigiiN2Lesson11,
            11 => WellKnownSets::MigiiN2Lesson12,
            12 => WellKnownSets::MigiiN2Lesson13,
            13 => WellKnownSets::MigiiN2Lesson14,
            14 => WellKnownSets::MigiiN2Lesson15,
            15 => WellKnownSets::MigiiN2Lesson16,
            16 => WellKnownSets::MigiiN2Lesson17,
            17 => WellKnownSets::MigiiN2Lesson18,
            18 => WellKnownSets::MigiiN2Lesson19,
            19 => WellKnownSets::MigiiN2Lesson20,
            20 => WellKnownSets::MigiiN2Lesson21,
            21 => WellKnownSets::MigiiN2Lesson22,
            22 => WellKnownSets::MigiiN2Lesson23,
            23 => WellKnownSets::MigiiN2Lesson24,
            24 => WellKnownSets::MigiiN2Lesson25,
            25 => WellKnownSets::MigiiN2Lesson26,
            26 => WellKnownSets::MigiiN2Lesson27,
            27 => WellKnownSets::MigiiN2Lesson28,
            28 => WellKnownSets::MigiiN2Lesson29,
            29 => WellKnownSets::MigiiN2Lesson30,
            30 => WellKnownSets::MigiiN2Lesson31,
            _ => continue,
        };
        sets.insert(set_type, parse_set(json, set_type)?);
    }

    for (i, json) in data.migii_n1.iter().enumerate() {
        let set_type = match i {
            0 => WellKnownSets::MigiiN1Lesson1,
            1 => WellKnownSets::MigiiN1Lesson2,
            2 => WellKnownSets::MigiiN1Lesson3,
            3 => WellKnownSets::MigiiN1Lesson4,
            4 => WellKnownSets::MigiiN1Lesson5,
            5 => WellKnownSets::MigiiN1Lesson6,
            6 => WellKnownSets::MigiiN1Lesson7,
            7 => WellKnownSets::MigiiN1Lesson8,
            8 => WellKnownSets::MigiiN1Lesson9,
            9 => WellKnownSets::MigiiN1Lesson10,
            10 => WellKnownSets::MigiiN1Lesson11,
            11 => WellKnownSets::MigiiN1Lesson12,
            12 => WellKnownSets::MigiiN1Lesson13,
            13 => WellKnownSets::MigiiN1Lesson14,
            14 => WellKnownSets::MigiiN1Lesson15,
            15 => WellKnownSets::MigiiN1Lesson16,
            16 => WellKnownSets::MigiiN1Lesson17,
            17 => WellKnownSets::MigiiN1Lesson18,
            18 => WellKnownSets::MigiiN1Lesson19,
            19 => WellKnownSets::MigiiN1Lesson20,
            20 => WellKnownSets::MigiiN1Lesson21,
            21 => WellKnownSets::MigiiN1Lesson22,
            22 => WellKnownSets::MigiiN1Lesson23,
            23 => WellKnownSets::MigiiN1Lesson24,
            24 => WellKnownSets::MigiiN1Lesson25,
            25 => WellKnownSets::MigiiN1Lesson26,
            26 => WellKnownSets::MigiiN1Lesson27,
            27 => WellKnownSets::MigiiN1Lesson28,
            28 => WellKnownSets::MigiiN1Lesson29,
            29 => WellKnownSets::MigiiN1Lesson30,
            30 => WellKnownSets::MigiiN1Lesson31,
            31 => WellKnownSets::MigiiN1Lesson32,
            32 => WellKnownSets::MigiiN1Lesson33,
            33 => WellKnownSets::MigiiN1Lesson34,
            34 => WellKnownSets::MigiiN1Lesson35,
            35 => WellKnownSets::MigiiN1Lesson36,
            36 => WellKnownSets::MigiiN1Lesson37,
            37 => WellKnownSets::MigiiN1Lesson38,
            38 => WellKnownSets::MigiiN1Lesson39,
            39 => WellKnownSets::MigiiN1Lesson40,
            40 => WellKnownSets::MigiiN1Lesson41,
            41 => WellKnownSets::MigiiN1Lesson42,
            42 => WellKnownSets::MigiiN1Lesson43,
            43 => WellKnownSets::MigiiN1Lesson44,
            44 => WellKnownSets::MigiiN1Lesson45,
            45 => WellKnownSets::MigiiN1Lesson46,
            46 => WellKnownSets::MigiiN1Lesson47,
            47 => WellKnownSets::MigiiN1Lesson48,
            48 => WellKnownSets::MigiiN1Lesson49,
            49 => WellKnownSets::MigiiN1Lesson50,
            50 => WellKnownSets::MigiiN1Lesson51,
            51 => WellKnownSets::MigiiN1Lesson52,
            52 => WellKnownSets::MigiiN1Lesson53,
            53 => WellKnownSets::MigiiN1Lesson54,
            54 => WellKnownSets::MigiiN1Lesson55,
            55 => WellKnownSets::MigiiN1Lesson56,
            _ => continue,
        };
        sets.insert(set_type, parse_set(json, set_type)?);
    }

    let _ = WELL_KNOWN_SETS.set(sets);
    Ok(())
}

pub fn is_well_known_sets_loaded() -> bool {
    WELL_KNOWN_SETS.get().is_some()
}

pub fn load_well_known_set(set: &WellKnownSets) -> Result<WellKnownSet, OrigaError> {
    WELL_KNOWN_SETS
        .get()
        .and_then(|sets| sets.get(set).cloned())
        .ok_or(OrigaError::WellKnownSetParseError {
            reason: format!("Well known set {:?} not loaded", set),
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
