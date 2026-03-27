use crate::api::prompts::{get_english_translation_prompt, get_russian_translation_prompt};
use crate::api::types::{ChatMessage, ChatRequest, ChatResponse, VocabularyEntry};
use origa::domain::OrigaError;

/// Creates a chat request with the given prompt
fn create_chat_request(prompt: String) -> ChatRequest {
    ChatRequest {
        model: "llm".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens: 500,
        temperature: 0.3,
        top_p: 0.9,
        presence_penalty: 0.0,
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

/// Extracts translation from chat response
fn extract_translation(response: ChatResponse) -> Option<String> {
    response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
}

/// Translates a word to a single language using the API
async fn translate_to_language(
    word: &str,
    api_base: &str,
    api_key: &str,
    language: &str,
) -> Result<Option<String>, OrigaError> {
    let client = reqwest::Client::new();

    let prompt = match language {
        "russian" => get_russian_translation_prompt(word),
        "english" => get_english_translation_prompt(word),
        _ => return Ok(None),
    };

    let request = create_chat_request(prompt);
    let response = send_chat_request(&client, api_base, api_key, &request).await?;

    match extract_translation(response) {
        Some(translation) => Ok(Some(translation)),
        None => {
            tracing::warn!(
                "API returned empty translation for '{}' to {}",
                word,
                language
            );
            Ok(None)
        },
    }
}

/// Translates a word to Russian and/or English using the API
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

    if to_russian {
        entry.russian_translation =
            translate_to_language(word, api_base, api_key, "russian").await?;
    }

    if to_english {
        entry.english_translation =
            translate_to_language(word, api_base, api_key, "english").await?;
    }

    Ok(entry)
}
