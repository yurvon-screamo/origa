use std::collections::HashMap;
use std::future::Future;

use crate::domain::OrigaError;
use crate::traits::CdnProvider;

pub struct MockCdnProvider {
    texts: HashMap<String, String>,
    bytes: HashMap<String, Vec<u8>>,
}

impl MockCdnProvider {
    pub fn new() -> Self {
        Self {
            texts: HashMap::new(),
            bytes: HashMap::new(),
        }
    }

    pub fn with_text(mut self, path: &str, content: &str) -> Self {
        self.texts.insert(path.to_string(), content.to_string());
        self
    }

    pub fn with_bytes(mut self, path: &str, data: Vec<u8>) -> Self {
        self.bytes.insert(path.to_string(), data);
        self
    }
}

impl Default for MockCdnProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CdnProvider for MockCdnProvider {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        let result = self
            .texts
            .get(path)
            .cloned()
            .ok_or_else(|| OrigaError::NetworkError {
                url: path.to_string(),
                reason: "Not found in mock".to_string(),
            });
        async move { result }
    }

    fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
        let result = self
            .bytes
            .get(path)
            .cloned()
            .ok_or_else(|| OrigaError::NetworkError {
                url: path.to_string(),
                reason: "Not found in mock".to_string(),
            });
        async move { result }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_cdn_returns_configured_text() {
        let cdn = MockCdnProvider::new().with_text("test.json", r#"{"ok":true}"#);
        let result: Result<String, OrigaError> = cdn.fetch_text("test.json").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"ok":true}"#);
    }

    #[tokio::test]
    async fn mock_cdn_returns_error_for_missing_text() {
        let cdn = MockCdnProvider::new();
        let result: Result<String, OrigaError> = cdn.fetch_text("missing.json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn mock_cdn_returns_configured_bytes() {
        let cdn = MockCdnProvider::new().with_bytes("model.bin", vec![1, 2, 3]);
        let result: Result<Vec<u8>, OrigaError> = cdn.fetch_bytes("model.bin").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }
}
