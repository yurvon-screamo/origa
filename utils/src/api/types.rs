use serde::{Deserialize, Serialize};

/// Message in a chat conversation
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Reasoning effort levels for OpenRouter API
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    High,
}

/// Reasoning configuration for OpenRouter API
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl ReasoningConfig {
    pub fn high() -> Self {
        Self {
            effort: Some(ReasoningEffort::High),
            enabled: None,
        }
    }

    pub fn disabled() -> Self {
        Self {
            effort: None,
            enabled: Some(false),
        }
    }
}

/// Request to chat completions API
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub presence_penalty: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_template_kwargs: Option<serde_json::Value>,
}

/// Response from chat completions API
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

/// Choice in a chat response
#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessageContent,
}

/// Content of a chat message
#[derive(Debug, Deserialize)]
pub struct ChatMessageContent {
    #[serde(default)]
    pub content: Option<String>,
}

/// Entry in the vocabulary dictionary with translations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub russian_translation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub english_translation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found_in_sets: Option<Vec<String>>,
}

/// Grammar rule content for a single language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarContent {
    pub title: String,
    pub short_description: String,
    pub explanation: String,
    pub how_to_form: String,
    pub examples: String,
    pub nuances: String,
    pub pro_tip: String,
    #[serde(default)]
    pub related_patterns: Option<String>,
}

/// Bilingual grammar content (EN + RU)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BilingualGrammarContent {
    pub en: GrammarContent,
    pub ru: GrammarContent,
}

/// Bilingual translation (EN + RU)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BilingualTranslation {
    pub en: String,
    pub ru: String,
}
