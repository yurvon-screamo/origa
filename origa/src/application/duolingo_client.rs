use crate::domain::OrigaError;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct DuolingoWord {
    pub text: String,
    pub translations: Vec<String>,
}

pub trait DuolingoClient {
    fn get_words(
        &self,
        jwt_token: &str,
    ) -> impl Future<Output = Result<Vec<DuolingoWord>, OrigaError>>;
}
