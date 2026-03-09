use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct Vocabulary {
    chars: Vec<char>,
    #[allow(dead_code)]
    char_to_idx: HashMap<char, usize>,
}

impl Vocabulary {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read vocabulary file: {:?}", path))?;

        let chars: Vec<char> = content
            .lines()
            .filter(|line| !line.is_empty())
            .flat_map(|line| line.chars().next())
            .collect();

        let char_to_idx: HashMap<char, usize> =
            chars.iter().enumerate().map(|(i, &c)| (c, i + 1)).collect();

        Ok(Self { chars, char_to_idx })
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
