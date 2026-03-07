use crate::domain::OrigaError;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub base_url: String,
    pub model_name: String,
    pub cache_dir: String,
}

impl ModelConfig {
    pub fn new(
        base_url: impl Into<String>,
        model_name: impl Into<String>,
        cache_dir: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            model_name: model_name.into(),
            cache_dir: cache_dir.into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Result<Self, OrigaError> {
        let url = base_url.into();
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(OrigaError::OcrError {
                reason: "base_url must start with http:// or https://".to_string(),
            });
        }
        self.base_url = url;
        Ok(self)
    }

    pub fn with_model_name(mut self, model_name: impl Into<String>) -> Result<Self, OrigaError> {
        let name = model_name.into();
        if name.contains("..") || name.contains('\\') {
            return Err(OrigaError::OcrError {
                reason: "model_name cannot contain '..' or '\\'".to_string(),
            });
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/')
        {
            return Err(OrigaError::OcrError {
                reason: "model_name can only contain alphanumeric characters, '-', '_', and '/'"
                    .to_string(),
            });
        }
        self.model_name = name;
        Ok(self)
    }

    pub fn with_cache_dir(mut self, cache_dir: impl Into<String>) -> Self {
        self.cache_dir = cache_dir.into();
        self
    }

    pub fn model_file_url(&self, filename: &str) -> String {
        format!(
            "{}/{}/resolve/main/{}",
            self.base_url.trim_end_matches('/'),
            self.model_name
                .trim_start_matches('/')
                .trim_end_matches('/'),
            filename.trim_start_matches('/')
        )
    }

    pub fn file_names() -> &'static [&'static str] {
        &[
            "encoder_model.onnx",
            "decoder_model.onnx",
            "tokenizer.json",
            "config.json",
            "preprocessor_config.json",
            "special_tokens_map.json",
            "tokenizer_config.json",
            "vocab.txt",
            "generation_config.json",
        ]
    }
}
