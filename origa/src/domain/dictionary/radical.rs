use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::domain::{OrigaError, value_objects::JapaneseLevel};

pub static RADICAL_DICTIONARY: OnceLock<RadicalDatabase> = OnceLock::new();

pub struct RadicalData {
    pub radicals_json: String,
}

pub fn init_radical_dictionary(data: RadicalData) -> Result<(), OrigaError> {
    let db = RadicalDatabase::from_json(&data.radicals_json)?;
    let _ = RADICAL_DICTIONARY.set(db);
    Ok(())
}

pub fn is_radical_loaded() -> bool {
    RADICAL_DICTIONARY.get().is_some()
}

pub fn get_radical_info(radical: char) -> Result<&'static RadicalInfo, OrigaError> {
    RADICAL_DICTIONARY
        .get()
        .ok_or(OrigaError::KradfileError {
            reason: "Radical dictionary not loaded".to_string(),
        })?
        .get_radical_info(&radical)
}

pub struct RadicalDatabase {
    known_radicals: Vec<char>,
    radical_map: HashMap<char, RadicalInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadicalInfo {
    radical: char,
    stroke_count: u32,
    name: String,
    description: String,
    jlpt: JapaneseLevel,
    kanji: Vec<char>,
}

impl RadicalInfo {
    pub fn radical(&self) -> char {
        self.radical
    }

    pub fn stroke_count(&self) -> u32 {
        self.stroke_count
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn jlpt(&self) -> &JapaneseLevel {
        &self.jlpt
    }

    pub fn kanji(&self) -> &[char] {
        &self.kanji
    }
}

impl RadicalDatabase {
    fn from_json(json: &str) -> Result<Self, OrigaError> {
        let radkfile: RadkfilStoredType =
            serde_json::from_str(json).map_err(|e| OrigaError::KradfileError {
                reason: format!("Failed to parse radicals.json: {}", e),
            })?;
        let radical_map = radkfile
            .radicals
            .into_iter()
            .map(|(k, v)| {
                let radical_char = k.chars().next().unwrap();
                let jlpt = JapaneseLevel::from_str_or_default(&v.jlpt);
                let kanji = v
                    .kanji
                    .into_iter()
                    .flat_map(|c| c.chars().collect::<Vec<_>>())
                    .collect::<Vec<char>>();

                (
                    radical_char,
                    RadicalInfo {
                        radical: radical_char,
                        stroke_count: v.stroke_count,
                        name: v.name,
                        description: v.description,
                        jlpt,
                        kanji,
                    },
                )
            })
            .collect::<HashMap<char, RadicalInfo>>();

        let known_radicals: Vec<_> = radical_map.keys().copied().collect();

        Ok(Self {
            known_radicals,
            radical_map,
        })
    }

    pub fn get_radical_info(&self, radical: &char) -> Result<&RadicalInfo, OrigaError> {
        self.radical_map
            .get(radical)
            .ok_or(OrigaError::KradfileError {
                reason: format!("Radical {} not found in radkfile", radical),
            })
    }

    pub fn known_radicals(&self) -> &[char] {
        &self.known_radicals
    }
}

#[derive(Serialize, Deserialize)]
struct RadicalStoredType {
    #[serde(rename = "strokeCount")]
    stroke_count: u32,
    kanji: Vec<String>,
    name: String,
    description: String,
    jlpt: String,
}

#[derive(Serialize, Deserialize)]
struct RadkfilStoredType {
    radicals: HashMap<String, RadicalStoredType>,
}
