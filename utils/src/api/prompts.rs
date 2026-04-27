/// Supported languages for prompt generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Russian,
    English,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::Russian => "russian",
            Language::English => "english",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "russian" | "ru" => Some(Language::Russian),
            "english" | "en" => Some(Language::English),
            _ => None,
        }
    }
}

/// Input context for building a grammar prompt
pub struct GrammarPromptInput<'a> {
    pub title: &'a str,
    pub level: &'a str,
    pub rule_name_from_index: Option<&'a str>,
    pub language: Language,
}

const GRAMMAR_RU: &str = include_str!("prompts/grammar_ru.md");
const GRAMMAR_EN: &str = include_str!("prompts/grammar_en.md");
const TRANSLATION_RU: &str = include_str!("prompts/translation_ru.md");
const TRANSLATION_EN: &str = include_str!("prompts/translation_en.md");

pub fn get_grammar_prompt(input: &GrammarPromptInput) -> String {
    let template = match input.language {
        Language::Russian => GRAMMAR_RU,
        Language::English => GRAMMAR_EN,
    };

    let rule_name_from_index = match input.rule_name_from_index {
        Some(name) => format!("\n  <rule_name_from_index>{}</rule_name_from_index>", name),
        None => String::new(),
    };

    template
        .replace("{title}", input.title)
        .replace("{level}", input.level)
        .replace("{rule_name_from_index}", &rule_name_from_index)
}

pub fn get_russian_translation_prompt(word: &str) -> String {
    TRANSLATION_RU.replace("{word}", word)
}

pub fn get_english_translation_prompt(word: &str) -> String {
    TRANSLATION_EN.replace("{word}", word)
}

pub fn get_grammar_russian_prompt(
    title: &str,
    level: &str,
    rule_name_from_index: Option<&str>,
) -> String {
    get_grammar_prompt(&GrammarPromptInput {
        title,
        level,
        rule_name_from_index,
        language: Language::Russian,
    })
}

pub fn get_grammar_english_prompt(
    title: &str,
    level: &str,
    rule_name_from_index: Option<&str>,
) -> String {
    get_grammar_prompt(&GrammarPromptInput {
        title,
        level,
        rule_name_from_index,
        language: Language::English,
    })
}
