use crate::domain::OrigaError;
use std::path::Path;

#[derive(Clone)]
pub struct Vocabulary {
    chars: Vec<char>,
}

impl Vocabulary {
    pub fn from_file(path: &Path) -> Result<Self, OrigaError> {
        let content = std::fs::read_to_string(path).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to read vocabulary file {:?}: {:?}", path, e),
        })?;

        let chars: Vec<char> = content
            .lines()
            .filter(|line| !line.is_empty())
            .flat_map(|line| line.chars().next())
            .collect();

        Ok(Self { chars })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OrigaError> {
        let content = std::str::from_utf8(bytes).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to parse vocabulary as UTF-8: {:?}", e),
        })?;

        let chars: Vec<char> = content
            .lines()
            .filter(|line| !line.is_empty())
            .flat_map(|line| line.chars().next())
            .collect();

        Ok(Self { chars })
    }

    pub fn decode(&self, indices: &[i64]) -> String {
        indices
            .iter()
            .filter(|&&idx| idx > 0)
            .filter_map(|&idx| {
                let char_idx = (idx - 1) as usize;
                self.chars.get(char_idx).copied()
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}
