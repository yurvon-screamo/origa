#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub ndlocr_base_url: String,
    pub ndlocr_cache_dir: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            ndlocr_base_url: "/ndlocr".to_string(),
            ndlocr_cache_dir: "ndlocr-model-".to_string(),
        }
    }
}

impl ModelConfig {
    pub fn new(ndlocr_base_url: impl Into<String>, ndlocr_cache_dir: impl Into<String>) -> Self {
        Self {
            ndlocr_base_url: ndlocr_base_url.into(),
            ndlocr_cache_dir: ndlocr_cache_dir.into(),
        }
    }

    pub fn ndlocr_file_names() -> &'static [&'static str] {
        &[
            "deim.onnx",
            "parseq-30.onnx",
            "parseq-50.onnx",
            "parseq-100.onnx",
            "vocab.txt",
        ]
    }

    pub fn model_url(&self, filename: &str) -> String {
        format!(
            "{}/{}",
            self.ndlocr_base_url.trim_end_matches('/'),
            filename.trim_start_matches('/')
        )
    }
}
