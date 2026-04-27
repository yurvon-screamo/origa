use crate::api::GrammarPromptInput;
use origa::domain::OrigaError;

pub fn run_generate_grammar_prompt(
    title: String,
    level: String,
    rule_name_from_index: Option<String>,
) -> Result<(), OrigaError> {
    let input = GrammarPromptInput {
        title: &title,
        level: &level,
        rule_name_from_index: rule_name_from_index.as_deref(),
    };

    let prompt = crate::api::get_grammar_prompt(&input);
    println!("{}", prompt);

    Ok(())
}
