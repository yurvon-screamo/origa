use std::future::Future;

use crate::domain::OrigaError;

pub trait CdnProvider {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>>;
    fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>>;
}
