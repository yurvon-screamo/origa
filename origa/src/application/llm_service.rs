use crate::domain::OrigaError;
use std::future::Future;

pub trait LlmService {
    fn generate_text(&self, question: &str) -> impl Future<Output = Result<String, OrigaError>>;
}
