/// Input context for building a grammar prompt
pub struct GrammarPromptInput<'a> {
    pub title: &'a str,
    pub level: &'a str,
    pub rule_name_from_index: Option<&'a str>,
}

const GRAMMAR: &str = include_str!("prompts/grammar.md");
const TRANSLATION: &str = include_str!("prompts/translation.md");

pub fn get_grammar_prompt(input: &GrammarPromptInput) -> String {
    let rule_name_from_index = match input.rule_name_from_index {
        Some(name) => format!("\n  <rule_name_from_index>{}</rule_name_from_index>", name),
        None => String::new(),
    };

    GRAMMAR
        .replace("{title}", input.title)
        .replace("{level}", input.level)
        .replace("{rule_name_from_index}", &rule_name_from_index)
}

pub fn get_translation_prompt(word: &str) -> String {
    TRANSLATION.replace("{word}", word)
}
