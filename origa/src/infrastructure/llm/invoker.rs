use async_trait::async_trait;

use crate::application::LlmService;
use crate::domain::OrigaError;

use super::OpenAiLlm;

pub enum LlmServiceInvoker {
    None,
    OpenAi(OpenAiLlm),
}

#[async_trait]
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
