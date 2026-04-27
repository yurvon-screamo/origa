use crate::api::{GrammarPromptInput, Language};
use origa::domain::OrigaError;

pub fn run_generate_grammar_prompt(
    title: String,
    level: String,
    language: String,
    rule_name_from_index: Option<String>,
) -> Result<(), OrigaError> {
    let lang = Language::from_code(&language).ok_or_else(|| OrigaError::TokenizerError {
        reason: format!(
            "Unsupported language '{}'. Use 'russian'/'ru' or 'english'/'en'",
            language
        ),
    })?;

    let input = GrammarPromptInput {
        title: &title,
        level: &level,
        rule_name_from_index: rule_name_from_index.as_deref(),
        language: lang,
    };

    let prompt = crate::api::get_grammar_prompt(&input);
    println!("{}", prompt);

    Ok(())
}
