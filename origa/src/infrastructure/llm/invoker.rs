use crate::application::LlmService;
use crate::domain::OrigaError;

use super::OpenAiLlm;

#[derive(Clone)]
pub enum LlmServiceInvoker {
    None,
    OpenAi(OpenAiLlm),
}

impl LlmService for LlmServiceInvoker {
    async fn generate_text(&self, question: &str) -> Result<String, OrigaError> {
        match self {
            LlmServiceInvoker::None => Err(OrigaError::InvalidValues {
                reason: "Please set LLM settings in your profile".to_string(),
            }),
            LlmServiceInvoker::OpenAi(service) => service.generate_text(question).await,
        }
    }
}
