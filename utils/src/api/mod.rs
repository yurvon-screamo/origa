mod client;
mod prompts;
mod types;

pub use client::{generate_grammar_description, translate_word, validate_translation};
pub use prompts::{GrammarPromptInput, Language, get_grammar_prompt};
pub use types::VocabularyEntry;
