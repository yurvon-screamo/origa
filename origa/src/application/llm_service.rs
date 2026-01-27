use async_trait::async_trait;

use crate::domain::OrigaError;

#[async_trait(?Send)]
pub trait LlmService: Send + Sync {
    async fn generate_text(&self, question: &str) -> Result<String, OrigaError>;
}
