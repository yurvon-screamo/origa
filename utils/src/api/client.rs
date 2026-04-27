use std::time::Duration;

use crate::api::prompts::{get_grammar_prompt, get_translation_prompt};
use crate::api::types::{
    BilingualGrammarContent, BilingualTranslation, ChatMessage, ChatRequest, ChatResponse,
    ReasoningConfig, VocabularyEntry,
};
use origa::domain::OrigaError;

fn create_chat_request(prompt: String) -> ChatRequest {
    create_chat_request_with_model(prompt, "llm".to_string())
}

fn create_chat_request_with_model(prompt: String, model: String) -> ChatRequest {
    ChatRequest {
        model,
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens: 500,
        temperature: 0.3,
        top_p: 0.9,
        presence_penalty: 0.0,
        reasoning: None,
    }
}

/// Sends a chat request to the API and returns the response
async fn send_chat_request(
    client: &reqwest::Client,
    api_base: &str,
    api_key: &str,
    request: &ChatRequest,
) -> Result<ChatResponse, OrigaError> {
    let response = client
        .post(format!("{}/chat/completions", api_base))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(request)
        .send()
        .await
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("API request failed: {}", e),
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::error!("API returned error status {}: {}", status, body);
        return Err(OrigaError::TokenizerError {
            reason: format!("API returned error status {}: {}", status, body),
        });
    }

    response
        .json()
        .await
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to parse API response: {}", e),
        })
}

fn strip_json_fences(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with("```json") && trimmed.ends_with("```") {
        &trimmed[7..trimmed.len() - 3]
    } else if trimmed.starts_with("```") && trimmed.ends_with("```") {
        &trimmed[3..trimmed.len() - 3]
    } else {
        trimmed
    }
    .trim()
}

async fn send_chat_request_with_retry(
    client: &reqwest::Client,
    api_base: &str,
    api_key: &str,
    request: &ChatRequest,
    max_retries: u32,
) -> Result<ChatResponse, OrigaError> {
    let mut last_error = None;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = Duration::from_millis(500 * 2u64.pow(attempt - 1));
            tracing::warn!(
                "Retry attempt {}/{} after {}ms",
                attempt,
                max_retries,
                delay.as_millis()
            );
            tokio::time::sleep(delay).await;
        }
        match send_chat_request(client, api_base, api_key, request).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                let should_retry = match &e {
                    OrigaError::TokenizerError { reason } => {
                        reason.contains("429")
                            || reason.contains("500")
                            || reason.contains("502")
                            || reason.contains("503")
                    },
                    _ => false,
                };
                if should_retry && attempt < max_retries {
                    last_error = Some(e);
                    continue;
                }
                return Err(e);
            },
        }
    }
    Err(match last_error {
        Some(e) => e,
        None => OrigaError::TokenizerError {
            reason: "All retry attempts exhausted".to_string(),
        },
    })
}

fn parse_bilingual_translation(raw: &str) -> Result<BilingualTranslation, OrigaError> {
    let cleaned = strip_json_fences(raw);

    serde_json::from_str(cleaned).map_err(|e| {
        tracing::error!(
            "Failed to parse bilingual translation JSON: {}. Raw: {}...",
            e,
            &raw[..raw.len().min(200)]
        );
        OrigaError::LlmError {
            reason: format!("Failed to parse bilingual translation JSON response: {}", e),
        }
    })
}

fn parse_bilingual_grammar_response(raw: &str) -> Result<BilingualGrammarContent, OrigaError> {
    let cleaned = strip_json_fences(raw);

    serde_json::from_str(cleaned).map_err(|e| {
        tracing::error!(
            "Failed to parse bilingual grammar JSON: {}. Raw: {}...",
            e,
            &raw[..raw.len().min(200)]
        );
        OrigaError::LlmError {
            reason: format!("Failed to parse bilingual grammar JSON response: {}", e),
        }
    })
}

/// Translates a word to both Russian and English using a single API call
pub async fn translate_word(
    word: &str,
    api_base: &str,
    api_key: &str,
    to_russian: bool,
    to_english: bool,
) -> Result<VocabularyEntry, OrigaError> {
    let mut entry = VocabularyEntry {
        russian_translation: None,
        english_translation: None,
        found_in_sets: None,
    };

    if !to_russian && !to_english {
        return Ok(entry);
    }

    let prompt = get_translation_prompt(word);
    let client = reqwest::Client::new();
    let request = create_chat_request(prompt);
    let response = send_chat_request(&client, api_base, api_key, &request).await?;

    let raw = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string());

    match raw {
        Some(content) => {
            let bilingual = parse_bilingual_translation(&content)?;
            if to_russian {
                entry.russian_translation = Some(bilingual.ru);
            }
            if to_english {
                entry.english_translation = Some(bilingual.en);
            }
        },
        None => {
            tracing::warn!("API returned empty translation for '{}'", word);
        },
    }

    Ok(entry)
}

pub async fn send_generic_chat_with_model(
    api_base: &str,
    api_key: &str,
    prompt: String,
    max_tokens: u32,
    model: String,
    reasoning: Option<ReasoningConfig>,
) -> Result<String, OrigaError> {
    let client = reqwest::Client::new();
    let request = ChatRequest {
        model,
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens,
        temperature: 0.3,
        top_p: 0.9,
        presence_penalty: 0.0,
        reasoning,
    };

    let response = send_chat_request_with_retry(&client, api_base, api_key, &request, 3).await?;

    response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .ok_or_else(|| OrigaError::LlmError {
            reason: "Empty response from API".to_string(),
        })
}

pub async fn validate_translation(
    api_base: &str,
    api_key: &str,
    model: &str,
    word: &str,
    russian_translation: &str,
    english_translation: &str,
) -> Result<(bool, String), OrigaError> {
    let client = reqwest::Client::new();
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a translation validator. Respond with exactly one character: Y if the translation is correct, N if it is not. Never output anything else.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Japanese word: {}\nRussian translation: {}\nEnglish translation: {}\nIs the translation correct? Y/N:",
                    word, russian_translation, english_translation
                ),
            },
        ],
        max_tokens: 1,
        temperature: 0.0,
        top_p: 1.0,
        presence_penalty: 0.0,
        reasoning: Some(ReasoningConfig::disabled()),
    };

    let max_parse_retries = 3;
    for attempt in 0..=max_parse_retries {
        let response =
            send_chat_request_with_retry(&client, api_base, api_key, &request, 3).await?;

        let raw = response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .unwrap_or_default();

        match parse_validation_response(&raw) {
            Some(valid) => return Ok((valid, raw)),
            None => {
                if attempt < max_parse_retries {
                    tracing::warn!(
                        "Ambiguous LLM response for '{}': '{}', retrying ({}/{})",
                        word,
                        raw,
                        attempt + 1,
                        max_parse_retries
                    );
                    continue;
                }
                tracing::warn!(
                    "Could not parse LLM response for '{}' after {} retries: '{}', assuming valid",
                    word,
                    max_parse_retries,
                    raw
                );
                return Ok((true, raw));
            },
        }
    }
    Ok((true, "No response".to_string()))
}

fn parse_validation_response(raw: &str) -> Option<bool> {
    let trimmed = raw.trim().to_uppercase();
    let first_char = trimmed.chars().next()?;
    match first_char {
        'Y' => Some(true),
        'N' => Some(false),
        _ => None,
    }
}

/// Generates grammar description in both English and Russian
pub async fn generate_grammar_description(
    api_base: &str,
    api_key: &str,
    model: &str,
    title: &str,
    level: &str,
    rule_name_from_index: Option<&str>,
    reasoning: Option<ReasoningConfig>,
) -> Result<BilingualGrammarContent, OrigaError> {
    let input = crate::api::prompts::GrammarPromptInput {
        title,
        level,
        rule_name_from_index,
    };

    let prompt = get_grammar_prompt(&input);
    let raw = send_generic_chat_with_model(
        api_base,
        api_key,
        prompt,
        6000,
        model.to_string(),
        reasoning,
    )
    .await?;
    parse_bilingual_grammar_response(&raw)
}
