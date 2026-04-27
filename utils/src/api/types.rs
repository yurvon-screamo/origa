use serde::{Deserialize, Serialize};

/// Message in a chat conversation
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Reasoning configuration for OpenRouter API
#[derive(Debug, Serialize)]
pub struct ReasoningConfig {
    pub enabled: bool,
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
    pub content: String,
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
    pub md_description: String,
}
