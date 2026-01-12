use crate::domain::OrigaError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    text: String,
}

impl Question {
    pub fn new(text: String) -> Result<Self, OrigaError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(OrigaError::InvalidQuestion {
                reason: "Question text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    text: String,
}

impl Answer {
    pub fn new(text: String) -> Result<Self, OrigaError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(OrigaError::InvalidAnswer {
                reason: "Answer text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum JapaneseLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JapaneseLevel {
    pub fn as_number(&self) -> u8 {
        match self {
            JapaneseLevel::N5 => 5,
            JapaneseLevel::N4 => 4,
            JapaneseLevel::N3 => 3,
            JapaneseLevel::N2 => 2,
            JapaneseLevel::N1 => 1,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            JapaneseLevel::N5 => "N5",
            JapaneseLevel::N4 => "N4",
            JapaneseLevel::N3 => "N3",
            JapaneseLevel::N2 => "N2",
            JapaneseLevel::N1 => "N1",
        }
    }
}

impl fmt::Display for JapaneseLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_number())
    }
}

impl FromStr for JapaneseLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "N5" => Ok(JapaneseLevel::N5),
            "N4" => Ok(JapaneseLevel::N4),
            "N3" => Ok(JapaneseLevel::N3),
            "N2" => Ok(JapaneseLevel::N2),
            "N1" => Ok(JapaneseLevel::N1),
            other => Err(format!("Unknown Japanese level: {}", other)),
        }
    }
}

#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeLanguage {
    English,
    Russian,
}

impl NativeLanguage {
    pub fn as_str(&self) -> &str {
        match self {
            NativeLanguage::English => "English",
            NativeLanguage::Russian => "Russian",
        }
    }
}

impl fmt::Display for NativeLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
