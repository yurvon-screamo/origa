#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub ocr_base_url: String,
    pub ocr_model_name: String,
    pub ocr_cache_dir: String,
}

impl ModelConfig {
    pub fn default() -> Self {
        Self {
            ocr_base_url: "https://huggingface.co".to_string(),
            ocr_model_name: "l0wgear/manga-ocr-2025-onnx".to_string(),
            ocr_cache_dir: "manga-ocr-model-".to_string(),
        }
    }

    pub fn new(
        ocr_base_url: impl Into<String>,
        ocr_model_name: impl Into<String>,
        ocr_cache_dir: impl Into<String>,
    ) -> Self {
        Self {
            ocr_base_url: ocr_base_url.into(),
            ocr_model_name: ocr_model_name.into(),
            ocr_cache_dir: ocr_cache_dir.into(),
        }
    }

    pub fn ocr_model_file_url(&self, filename: &str) -> String {
        format!(
            "{}/{}/resolve/main/{}",
            self.ocr_base_url.trim_end_matches('/'),
            self.ocr_model_name
                .trim_start_matches('/')
                .trim_end_matches('/'),
            filename.trim_start_matches('/')
        )
    }

    pub fn ocr_file_names() -> &'static [&'static str] {
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
