use crate::application::LlmService;
use crate::domain::OrigaError;
use crate::domain::VOCABULARY_DICTIONARY;
use crate::domain::{Answer, JapaneseLevel, NativeLanguage};
use serde::Deserialize;

const MAX_RETRIES: usize = 3;

#[derive(Clone)]
pub struct GenerateCardContentUseCase<'a, L: LlmService> {
    llm_service: &'a L,
}

#[derive(Debug, Deserialize)]
struct LlmResponse {
    translation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardContent {
    pub answer: Answer,
}

impl<'a, L: LlmService> GenerateCardContentUseCase<'a, L> {
    pub fn new(llm_service: &'a L) -> Self {
        Self { llm_service }
    }

    pub async fn generate_content(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
        japanese_level: &JapaneseLevel,
    ) -> Result<CardContent, OrigaError> {
        if let Some(result) = self.try_get_from_dictionary(question_text, native_language)? {
            return Ok(result);
        }

        self.generate_with_llm(question_text, native_language, japanese_level)
            .await
    }

    fn try_get_from_dictionary(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
    ) -> Result<Option<CardContent>, OrigaError> {
        if let Some(translation) =
            VOCABULARY_DICTIONARY.get_translation(question_text, native_language)
        {
            let answer = Answer::new(translation)?;
            return Ok(Some(CardContent { answer }));
        }

        Ok(None)
    }

    async fn generate_with_llm(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
        japanese_level: &JapaneseLevel,
    ) -> Result<CardContent, OrigaError> {
        let prompt = build_prompt(question_text, native_language, japanese_level);
        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            match self.llm_service.generate_text(&prompt).await {
                Ok(response) => match self.process_llm_response(&response, attempt) {
                    Ok(result) => {
                        return Ok(result);
                    }
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < MAX_RETRIES {
                            continue;
                        }
                    }
                },
                Err(e) => {
                    last_error = Some(create_generation_error(attempt, &e));
                    if attempt < MAX_RETRIES {
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| OrigaError::LlmError {
            reason: "Failed to generate content after all retries".to_string(),
        }))
    }

    fn process_llm_response(
        &self,
        response: &str,
        attempt: usize,
    ) -> Result<CardContent, OrigaError> {
        let response_cleaned = clean_json_response(response);
        let llm_response = parse_json_response(&response_cleaned, attempt)?;
        let answer = validate_and_create_answer(&llm_response.translation, attempt)?;
        Ok(CardContent { answer })
    }
}

fn build_prompt(
    question_text: &str,
    native_language: &NativeLanguage,
    japanese_level: &JapaneseLevel,
) -> String {
    format!(
        r#"Ты — помощник для изучения языков.
Твоя задача: Создай перевод слова: '{word}' для {native_language} говорящего студента уровня {japanese_level}.

Требования к переводу:
1. Ответь 1 предложением.
2. Не повторяй слово в ответе, потому что твой ответ будет использоваться как обратная сторона карточки и нужно иметь возможность их переворачивать и прогонять в обратном направлении.
3. Не указывай в ответе чтение или транскрипцию, студент умеет читать.
4. Выдай просто ответ без вводных или объяснений зачем и для кого это.
5. Если слово состоит из 1 кандзи, то объясни его значение как слово, а не как кандзи.

Ответ должен быть СТРОГО валидным JSON, без markdown разметки (без ```json):
{{
  "translation": "перевод слова"
}}

Например (для уровня N5, русскоязычного студента):
{{
  "translation": "изучать"
}}"#,
        word = question_text
    )
}

fn clean_response_text(response: &str) -> String {
    response.trim_matches(['\n', '\r', '.', ' ']).to_string()
}

fn parse_json_response(response: &str, attempt: usize) -> Result<LlmResponse, OrigaError> {
    serde_json::from_str::<LlmResponse>(response).map_err(|e| OrigaError::LlmError {
        reason: format!(
            "Failed to parse JSON (attempt {}/{}): {}. Response: {}",
            attempt, MAX_RETRIES, e, response
        ),
    })
}

fn validate_and_create_answer(translation: &str, attempt: usize) -> Result<Answer, OrigaError> {
    let answer_text = clean_response_text(translation);
    Answer::new(answer_text).map_err(|e| OrigaError::LlmError {
        reason: format!(
            "Invalid answer format (attempt {}/{}): {}",
            attempt, MAX_RETRIES, e
        ),
    })
}

fn create_generation_error(attempt: usize, error: &dyn std::fmt::Display) -> OrigaError {
    OrigaError::LlmError {
        reason: format!(
            "Failed to generate content (attempt {}/{}): {}",
            attempt, MAX_RETRIES, error
        ),
    }
}

fn clean_json_response(response: &str) -> String {
    response.trim().replace("```json", "").replace("```", "")
}
