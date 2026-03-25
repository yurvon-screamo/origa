use crate::dictionary::load_dictionary;
use origa::domain::{OrigaError, tokenize_text};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Extracts vocabulary words from a text string
fn extract_vocab_words(text: &str) -> Result<HashSet<String>, OrigaError> {
    let mut vocab_words = HashSet::new();

    for line in text.lines() {
        let tokens = tokenize_text(line)?;
        for token in tokens {
            if token.part_of_speech().is_vocabulary_word() {
                vocab_words.insert(token.orthographic_base_form().to_string());
            }
        }
    }

    Ok(vocab_words)
}

/// Tokenizes Japanese text and extracts vocabulary words
pub fn run_tokenize(text: String, from_file: bool) -> Result<(), OrigaError> {
    load_dictionary()?;

    let vocab_words = if from_file || Path::new(&text).exists() {
        let bytes = fs::read(&text).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read file {}: {}", text, e),
        })?;
        let text_content = String::from_utf8_lossy(&bytes);
        extract_vocab_words(&text_content)?
    } else {
        extract_vocab_words(&text)?
    };

    let mut sorted_words: Vec<String> = vocab_words.into_iter().collect();
    sorted_words.sort();

    println!("{}", sorted_words.join(" "));
    Ok(())
}
